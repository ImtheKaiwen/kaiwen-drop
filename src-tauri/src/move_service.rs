//! Guvenli dosya tasima servisi.
//!
//! Garantiler:
//! * Hedefte ayni isim varsa ASLA uzerine yazmaz / birlestirmez.
//! * Ayni disk: `MoveFileExW` bayraksiz (REPLACE_EXISTING yok) -> atomik, var olani ezmez.
//! * Farkli disk: dogrulanmis kopyala -> kaynagi yalniz dogrulama basariliysa sil.
//! * Klasoru kendi icine/altina tasima engellenir.
//! * Silme/geri donusumsuz temizlik yok; kopya basarisizsa yalniz BIZIM olusturdugumuz kismi geri alinir.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Policy {
    /// Cakisan ogeleri atla.
    Skip,
    /// Cakisan ogeleri "ad (2)" gibi yeni adla tasi.
    Rename,
    /// Cakisma varsa hicbir sey yapma.
    Cancel,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanItem {
    pub source: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub conflict: bool,
    /// Kaynak zaten hedef klasorde.
    pub already_there: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Plan {
    pub dest: PathBuf,
    pub items: Vec<PlanItem>,
}

impl Plan {
    pub fn conflicts(&self) -> usize {
        self.items.iter().filter(|i| i.conflict).count()
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrKind {
    DestMissing,
    IntoSelf,
    Protected,
    SourceMissing,
    AccessDenied,
    InUse,
    CloudOnly,
    Unsupported,
    Other,
}

#[derive(Debug, Clone, Serialize)]
pub struct MoveError {
    pub kind: ErrKind,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ItemOutcome {
    Moved {
        name: String,
        to: PathBuf,
    },
    Skipped {
        name: String,
    },
    Failed {
        name: String,
        kind: ErrKind,
        detail: String,
    },
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct MoveResult {
    pub total: usize,
    pub moved: usize,
    pub skipped: usize,
    pub failed: usize,
    pub cancelled: bool,
    pub items: Vec<ItemOutcome>,
}

fn lower(p: &Path) -> String {
    let s = p.to_string_lossy().replace('/', "\\");
    let s = s.strip_prefix("\\\\?\\").unwrap_or(&s).to_string();
    s.trim_end_matches('\\').to_ascii_lowercase()
}

fn file_name_of(p: &Path) -> Option<String> {
    p.file_name().map(|n| n.to_string_lossy().to_string())
}

/// Kaynaklari ve hedefi salt okunur dogrular, plan uretir. Dosya sistemini DEGISTIRMEZ.
pub fn plan(sources: &[PathBuf], dest: &Path) -> Result<Plan, MoveError> {
    let dest_meta = fs::metadata(dest).map_err(|e| MoveError {
        kind: ErrKind::DestMissing,
        detail: e.to_string(),
    })?;
    if !dest_meta.is_dir() {
        return Err(MoveError {
            kind: ErrKind::DestMissing,
            detail: "hedef klasor degil".into(),
        });
    }
    let dest_c = fs::canonicalize(dest).map_err(|e| MoveError {
        kind: ErrKind::DestMissing,
        detail: e.to_string(),
    })?;
    let dest_l = lower(&dest_c);

    let mut items = Vec::new();
    for src in sources {
        if crate::selection::is_protected(src) {
            return Err(MoveError {
                kind: ErrKind::Protected,
                detail: src.display().to_string(),
            });
        }
        let meta = fs::symlink_metadata(src).map_err(|e| MoveError {
            kind: ErrKind::SourceMissing,
            detail: format!("{}: {e}", src.display()),
        })?;
        let name = file_name_of(src).ok_or_else(|| MoveError {
            kind: ErrKind::SourceMissing,
            detail: src.display().to_string(),
        })?;
        // Baglanti ise baglantinin kendisi tasinir; ana dizini canonicalize edip adi ekle.
        let parent = src.parent().ok_or_else(|| MoveError {
            kind: ErrKind::Protected,
            detail: src.display().to_string(),
        })?;
        let parent_c = fs::canonicalize(parent).map_err(|e| MoveError {
            kind: ErrKind::SourceMissing,
            detail: e.to_string(),
        })?;
        let src_c = parent_c.join(&name);
        let src_l = lower(&src_c);

        let is_dir = meta.is_dir();
        // Kendisi veya altina tasima.
        if dest_l == src_l || dest_l.starts_with(&format!("{src_l}\\")) {
            return Err(MoveError {
                kind: ErrKind::IntoSelf,
                detail: format!("{} -> {}", src.display(), dest.display()),
            });
        }
        let already_there = lower(&parent_c) == dest_l;
        let conflict = !already_there && fs::symlink_metadata(dest_c.join(&name)).is_ok();
        items.push(PlanItem {
            source: src.clone(),
            name,
            is_dir,
            conflict,
            already_there,
        });
    }
    Ok(Plan {
        dest: dest_c,
        items,
    })
}

/// "ad (2).ext" biciminde cakismayan bir ad bulur.
pub fn unique_name(dest: &Path, name: &str) -> String {
    let p = Path::new(name);
    let (stem, ext) = match (p.file_stem(), p.extension()) {
        (Some(s), Some(e)) if !name.starts_with('.') || name.matches('.').count() > 1 => (
            s.to_string_lossy().to_string(),
            format!(".{}", e.to_string_lossy()),
        ),
        _ => (name.to_string(), String::new()),
    };
    for n in 2..10_000 {
        let cand = format!("{stem} ({n}){ext}");
        if fs::symlink_metadata(dest.join(&cand)).is_err() {
            return cand;
        }
    }
    format!("{stem} (kopya){ext}")
}

/// Plani uygular. Plan alindiktan sonra dosya sisteminin degismis olabilecegi varsayilir: her ogede yeniden kontrol.
pub fn execute(plan: &Plan, policy: Policy) -> MoveResult {
    let mut res = MoveResult {
        total: plan.items.len(),
        ..Default::default()
    };
    if policy == Policy::Cancel && plan.conflicts() > 0 {
        res.cancelled = true;
        return res;
    }
    for item in &plan.items {
        if item.already_there {
            res.skipped += 1;
            res.items.push(ItemOutcome::Skipped {
                name: item.name.clone(),
            });
            continue;
        }
        let mut target_name = item.name.clone();
        let exists_now = fs::symlink_metadata(plan.dest.join(&target_name)).is_ok();
        if exists_now {
            match policy {
                Policy::Rename => target_name = unique_name(&plan.dest, &item.name),
                _ => {
                    res.skipped += 1;
                    res.items.push(ItemOutcome::Skipped {
                        name: item.name.clone(),
                    });
                    continue;
                }
            }
        }
        let target = plan.dest.join(&target_name);
        match move_one(&item.source, &target) {
            Ok(()) => {
                res.moved += 1;
                res.items.push(ItemOutcome::Moved {
                    name: item.name.clone(),
                    to: target,
                });
            }
            Err(e) => {
                res.failed += 1;
                res.items.push(ItemOutcome::Failed {
                    name: item.name.clone(),
                    kind: e.kind,
                    detail: e.detail,
                });
            }
        }
    }
    res
}

fn classify_io(e: &io::Error) -> ErrKind {
    match e.raw_os_error() {
        Some(5) => ErrKind::AccessDenied,
        Some(32) | Some(33) => ErrKind::InUse,
        Some(2) | Some(3) => ErrKind::SourceMissing,
        _ => match e.kind() {
            io::ErrorKind::PermissionDenied => ErrKind::AccessDenied,
            io::ErrorKind::NotFound => ErrKind::SourceMissing,
            _ => ErrKind::Other,
        },
    }
}

fn ioerr(e: io::Error) -> MoveError {
    MoveError {
        kind: classify_io(&e),
        detail: e.to_string(),
    }
}

/// Tek ogeyi hedefe tasir; hedef ZATEN VARSA basarisiz olur (ezmez).
pub fn move_one(src: &Path, target: &Path) -> Result<(), MoveError> {
    match rename_no_replace(src, target) {
        Ok(()) => Ok(()),
        Err(e) if e.raw_os_error() == Some(17) => copy_then_remove(src, target), // ERROR_NOT_SAME_DEVICE
        Err(e) if matches!(e.raw_os_error(), Some(80) | Some(183)) => Err(MoveError {
            kind: ErrKind::Other,
            detail: "hedefte ayni adli oge var".into(),
        }),
        Err(e) => Err(ioerr(e)),
    }
}

#[cfg(windows)]
fn rename_no_replace(src: &Path, dst: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::{MoveFileExW, MOVE_FILE_FLAGS};
    let w = |p: &Path| -> Vec<u16> {
        p.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    };
    let (s, d) = (w(src), w(dst));
    // Bayrak 0: REPLACE_EXISTING yok, COPY_ALLOWED yok.
    unsafe { MoveFileExW(PCWSTR(s.as_ptr()), PCWSTR(d.as_ptr()), MOVE_FILE_FLAGS(0)) }
        .map_err(|e| io::Error::from_raw_os_error(e.code().0 & 0xFFFF))
}

#[cfg(not(windows))]
fn rename_no_replace(src: &Path, dst: &Path) -> io::Result<()> {
    if fs::symlink_metadata(dst).is_ok() {
        return Err(io::Error::from_raw_os_error(183));
    }
    fs::rename(src, dst)
}

/// Farkli disk senaryosu: dogrulanmis kopya, sonra kaynagi sil.
pub fn copy_then_remove(src: &Path, target: &Path) -> Result<(), MoveError> {
    let meta = fs::symlink_metadata(src).map_err(ioerr)?;
    if is_special(&meta) {
        return Err(MoveError {
            kind: if is_recall(&meta) {
                ErrKind::CloudOnly
            } else {
                ErrKind::Unsupported
            },
            detail: "baglanti veya bulut yer tutucusu diskler arasi tasinamaz".into(),
        });
    }
    let before = if meta.is_dir() {
        copy_dir_new(src, target).and_then(|_| tree_summary(src))
    } else {
        copy_file_new(src, target).map(|n| (1u64, n))
    };
    let before = match before {
        Ok(b) => b,
        Err(e) => {
            // Hedef ZATEN vardiysa (AlreadyExists) o bizim degil: asla silinmez.
            // Aksi halde yalniz bizim olusturdugumuz kismi geri al; kaynaga dokunulmaz.
            let pre_existing =
                matches!(&e, CopyErr::Io(io) if io.kind() == io::ErrorKind::AlreadyExists);
            if !pre_existing {
                let _ = if meta.is_dir() {
                    fs::remove_dir_all(target)
                } else {
                    fs::remove_file(target)
                };
            }
            return Err(match e {
                CopyErr::Io(e) => ioerr(e),
                CopyErr::Unsupported => MoveError {
                    kind: ErrKind::Unsupported,
                    detail: "icerikte baglanti/bulut yer tutucusu var".into(),
                },
            });
        }
    };
    let after = if meta.is_dir() {
        tree_summary(target).map_err(|_| MoveError {
            kind: ErrKind::Other,
            detail: "dogrulama".into(),
        })?
    } else {
        (1, fs::metadata(target).map(|m| m.len()).unwrap_or(u64::MAX))
    };
    if before != after {
        let _ = if meta.is_dir() {
            fs::remove_dir_all(target)
        } else {
            fs::remove_file(target)
        };
        return Err(MoveError {
            kind: ErrKind::Other,
            detail: "kopya dogrulanamadi".into(),
        });
    }
    let removed = if meta.is_dir() {
        fs::remove_dir_all(src)
    } else {
        fs::remove_file(src)
    };
    removed.map_err(|e| MoveError {
        kind: classify_io(&e),
        detail: format!("kopyalandi ancak kaynak silinemedi: {e}"),
    })
}

enum CopyErr {
    Io(io::Error),
    Unsupported,
}
impl From<io::Error> for CopyErr {
    fn from(e: io::Error) -> Self {
        CopyErr::Io(e)
    }
}

fn is_special(m: &fs::Metadata) -> bool {
    m.file_type().is_symlink() || is_reparse(m) || is_recall(m)
}

#[cfg(windows)]
fn attrs(m: &fs::Metadata) -> u32 {
    use std::os::windows::fs::MetadataExt;
    m.file_attributes()
}
#[cfg(not(windows))]
fn attrs(_m: &fs::Metadata) -> u32 {
    0
}
fn is_reparse(m: &fs::Metadata) -> bool {
    attrs(m) & 0x400 != 0
}
fn is_recall(m: &fs::Metadata) -> bool {
    attrs(m) & (0x40_0000 | 0x4_0000 | 0x1000) != 0 // RECALL_ON_DATA_ACCESS | RECALL_ON_OPEN | OFFLINE
}

fn copy_file_new(src: &Path, dst: &Path) -> Result<u64, CopyErr> {
    let mut r = fs::File::open(src)?;
    let mut w = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dst)?;
    let n = io::copy(&mut r, &mut w)?;
    w.sync_all()?;
    Ok(n)
}

fn copy_dir_new(src: &Path, dst: &Path) -> Result<(), CopyErr> {
    fs::create_dir(dst)?; // varsa hata: birlestirme yok
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let m = fs::symlink_metadata(entry.path())?;
        if is_special(&m) {
            return Err(CopyErr::Unsupported);
        }
        let to = dst.join(entry.file_name());
        if m.is_dir() {
            copy_dir_new(&entry.path(), &to)?;
        } else {
            copy_file_new(&entry.path(), &to)?;
        }
    }
    Ok(())
}

fn tree_summary(p: &Path) -> Result<(u64, u64), CopyErr> {
    let mut files = 0u64;
    let mut bytes = 0u64;
    for entry in fs::read_dir(p)? {
        let entry = entry?;
        let m = fs::symlink_metadata(entry.path())?;
        if m.is_dir() {
            let (f, b) = tree_summary(&entry.path())?;
            files += f;
            bytes += b;
        } else {
            files += 1;
            bytes += m.len();
        }
    }
    Ok((files, bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(p: &Path, s: &str) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, s).unwrap();
    }

    #[test]
    fn moves_single_and_multiple() {
        let t = tempfile::tempdir().unwrap();
        let (a, b, dest) = (
            t.path().join("a.txt"),
            t.path().join("klasör/b.txt"),
            t.path().join("hedef"),
        );
        write(&a, "A");
        write(&b, "B");
        fs::create_dir(&dest).unwrap();
        let p = plan(&[a.clone(), t.path().join("klasör")], &dest).unwrap();
        let r = execute(&p, Policy::Cancel);
        assert_eq!((r.total, r.moved, r.failed), (2, 2, 0));
        assert!(!a.exists());
        assert!(dest.join("a.txt").exists());
        assert!(dest.join("klasör/b.txt").exists());
    }

    #[test]
    fn unicode_names() {
        let t = tempfile::tempdir().unwrap();
        let f = t.path().join("Çalışma İğne Öğün şeker.txt");
        write(&f, "ü");
        let dest = t.path().join("Fotoğraflar");
        fs::create_dir(&dest).unwrap();
        let r = execute(
            &plan(std::slice::from_ref(&f), &dest).unwrap(),
            Policy::Cancel,
        );
        assert_eq!(r.moved, 1);
        assert_eq!(
            fs::read_to_string(dest.join("Çalışma İğne Öğün şeker.txt")).unwrap(),
            "ü"
        );
    }

    #[test]
    fn never_overwrites_skip() {
        let t = tempfile::tempdir().unwrap();
        let f = t.path().join("x.txt");
        let dest = t.path().join("d");
        write(&f, "YENI");
        write(&dest.join("x.txt"), "ESKI");
        let p = plan(std::slice::from_ref(&f), &dest).unwrap();
        assert_eq!(p.conflicts(), 1);
        let r = execute(&p, Policy::Skip);
        assert_eq!((r.moved, r.skipped), (0, 1));
        assert_eq!(fs::read_to_string(dest.join("x.txt")).unwrap(), "ESKI");
        assert!(f.exists());
    }

    #[test]
    fn cancel_policy_moves_nothing() {
        let t = tempfile::tempdir().unwrap();
        let (f, g, dest) = (
            t.path().join("x.txt"),
            t.path().join("y.txt"),
            t.path().join("d"),
        );
        write(&f, "1");
        write(&g, "2");
        write(&dest.join("x.txt"), "ESKI");
        let r = execute(
            &plan(&[g.clone(), f.clone()], &dest).unwrap(),
            Policy::Cancel,
        );
        assert!(r.cancelled);
        assert_eq!(r.moved, 0);
        assert!(g.exists() && f.exists());
    }

    #[test]
    fn rename_policy_keeps_both() {
        let t = tempfile::tempdir().unwrap();
        let f = t.path().join("x.txt");
        let dest = t.path().join("d");
        write(&f, "YENI");
        write(&dest.join("x.txt"), "ESKI");
        let r = execute(
            &plan(std::slice::from_ref(&f), &dest).unwrap(),
            Policy::Rename,
        );
        assert_eq!(r.moved, 1);
        assert_eq!(fs::read_to_string(dest.join("x.txt")).unwrap(), "ESKI");
        assert_eq!(fs::read_to_string(dest.join("x (2).txt")).unwrap(), "YENI");
    }

    #[test]
    fn blocks_move_into_self_and_below() {
        let t = tempfile::tempdir().unwrap();
        let dir = t.path().join("proje");
        fs::create_dir_all(dir.join("alt/derin")).unwrap();
        for d in [dir.clone(), dir.join("alt"), dir.join("alt/derin")] {
            let e = plan(std::slice::from_ref(&dir), &d).unwrap_err();
            assert_eq!(e.kind, ErrKind::IntoSelf);
        }
    }

    #[test]
    fn same_parent_is_noop() {
        let t = tempfile::tempdir().unwrap();
        let f = t.path().join("x.txt");
        write(&f, "1");
        let p = plan(std::slice::from_ref(&f), t.path()).unwrap();
        assert!(p.items[0].already_there);
        let r = execute(&p, Policy::Cancel);
        assert_eq!((r.moved, r.skipped), (0, 1));
        assert!(f.exists());
    }

    #[test]
    fn partial_failure_reported() {
        let t = tempfile::tempdir().unwrap();
        let (a, b, c, dest) = (
            t.path().join("a.txt"),
            t.path().join("b.txt"),
            t.path().join("c.txt"),
            t.path().join("d"),
        );
        for f in [&a, &b, &c] {
            write(f, "x");
        }
        fs::create_dir(&dest).unwrap();
        let p = plan(&[a, b.clone(), c], &dest).unwrap();
        fs::remove_file(&b).unwrap(); // plan sonrasi kayboldu
        let r = execute(&p, Policy::Skip);
        assert_eq!((r.total, r.moved, r.failed), (3, 2, 1));
    }

    #[test]
    fn missing_dest_is_error() {
        let t = tempfile::tempdir().unwrap();
        let f = t.path().join("x.txt");
        write(&f, "1");
        let e = plan(&[f], &t.path().join("yok")).unwrap_err();
        assert_eq!(e.kind, ErrKind::DestMissing);
    }

    #[test]
    fn cross_volume_fallback_file_and_dir() {
        let t = tempfile::tempdir().unwrap();
        let f = t.path().join("büyük.bin");
        write(&f, &"z".repeat(100_000));
        copy_then_remove(&f, &t.path().join("kopya.bin")).unwrap();
        assert!(!f.exists());
        assert_eq!(
            fs::metadata(t.path().join("kopya.bin")).unwrap().len(),
            100_000
        );

        write(&t.path().join("k/a/b.txt"), "1");
        write(&t.path().join("k/c.txt"), "22");
        copy_then_remove(&t.path().join("k"), &t.path().join("k2")).unwrap();
        assert!(!t.path().join("k").exists());
        assert!(t.path().join("k2/a/b.txt").exists());
    }

    #[test]
    fn fallback_never_overwrites() {
        let t = tempfile::tempdir().unwrap();
        let f = t.path().join("x.txt");
        write(&f, "YENI");
        write(&t.path().join("hedef.txt"), "ESKI");
        assert!(copy_then_remove(&f, &t.path().join("hedef.txt")).is_err());
        assert_eq!(
            fs::read_to_string(t.path().join("hedef.txt")).unwrap(),
            "ESKI"
        );
        assert!(f.exists());

        // klasor: var olan hedef klasor ve icerigi dokunulmaz kalir
        write(&t.path().join("kaynak/a.txt"), "K");
        write(&t.path().join("var/onemli.txt"), "ONEMLI");
        assert!(copy_then_remove(&t.path().join("kaynak"), &t.path().join("var")).is_err());
        assert_eq!(
            fs::read_to_string(t.path().join("var/onemli.txt")).unwrap(),
            "ONEMLI"
        );
        assert!(t.path().join("kaynak/a.txt").exists());
    }

    #[test]
    fn move_one_refuses_existing_target() {
        let t = tempfile::tempdir().unwrap();
        let f = t.path().join("x.txt");
        write(&f, "YENI");
        write(&t.path().join("y.txt"), "ESKI");
        assert!(move_one(&f, &t.path().join("y.txt")).is_err());
        assert_eq!(fs::read_to_string(t.path().join("y.txt")).unwrap(), "ESKI");
    }

    #[test]
    fn unique_name_handles_ext_and_dirs() {
        let t = tempfile::tempdir().unwrap();
        write(&t.path().join("a.tar.gz"), "");
        assert_eq!(unique_name(t.path(), "a.tar.gz"), "a.tar (2).gz");
        fs::create_dir(t.path().join("klasör")).unwrap();
        assert_eq!(unique_name(t.path(), "klasör"), "klasör (2)");
    }
}
