//! Lazy klasor listeleme (yalniz alt klasorler), filtreleme, dogal siralama.

use serde::Serialize;
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub struct Filter {
    pub show_hidden: bool,
    pub hide_dev: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FolderItem {
    pub name: String,
    pub path: String,
    pub has_children: bool,
}

const DEV_FOLDERS: &[&str] = &[
    ".git",
    "node_modules",
    ".venv",
    "venv",
    "__pycache__",
    "target",
    "dist",
    "build",
    ".mypy_cache",
    ".pytest_cache",
    ".idea",
    ".vscode",
    ".next",
    ".cache",
    ".gradle",
];

/// has_children taramasinin ust siniri (cok buyuk klasorlerde varsayim: var).
const SCAN_CAP: usize = 600;

#[cfg(windows)]
fn attrs(m: &fs::Metadata) -> u32 {
    use std::os::windows::fs::MetadataExt;
    m.file_attributes()
}
#[cfg(not(windows))]
fn attrs(_m: &fs::Metadata) -> u32 {
    0
}

fn is_hidden_or_system(m: &fs::Metadata) -> bool {
    attrs(m) & (0x2 | 0x4) != 0
}

fn passes(name: &str, m: &fs::Metadata, f: &Filter) -> bool {
    if !f.show_hidden && (is_hidden_or_system(m) || name.starts_with('$')) {
        return false;
    }
    if f.hide_dev && DEV_FOLDERS.iter().any(|d| d.eq_ignore_ascii_case(name)) {
        return false;
    }
    true
}

/// Reparse point (junction/symlink) hedefi, listelenen dizinin kendisi veya ustu ise donguye girer.
fn creates_loop(child: &Path, parent_canon: &Path) -> bool {
    match fs::canonicalize(child) {
        Ok(c) => parent_canon.starts_with(&c),
        Err(_) => true, // cozulemeyen baglanti: gosterme
    }
}

pub fn list_subfolders(dir: &Path, f: &Filter) -> std::io::Result<Vec<FolderItem>> {
    let parent_canon = fs::canonicalize(dir)?;
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        let Ok(sm) = fs::symlink_metadata(&path) else {
            continue;
        };
        let is_link = sm.file_type().is_symlink() || attrs(&sm) & 0x400 != 0;
        let Ok(m) = (if is_link {
            fs::metadata(&path)
        } else {
            Ok(sm.clone())
        }) else {
            continue;
        };
        if !m.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !passes(&name, &sm, f) {
            continue;
        }
        if is_link && creates_loop(&path, &parent_canon) {
            continue;
        }
        let has_children = has_subdir(&path, f);
        out.push(FolderItem {
            name,
            path: path.to_string_lossy().to_string(),
            has_children,
        });
    }
    out.sort_by(|a, b| natural_cmp(&a.name, &b.name));
    Ok(out)
}

pub fn has_subdir(dir: &Path, f: &Filter) -> bool {
    let Ok(rd) = fs::read_dir(dir) else {
        return false;
    };
    for (i, e) in rd.enumerate() {
        if i >= SCAN_CAP {
            return true;
        }
        let Ok(e) = e else { continue };
        let Ok(sm) = fs::symlink_metadata(e.path()) else {
            continue;
        };
        let link = sm.file_type().is_symlink() || attrs(&sm) & 0x400 != 0;
        let is_dir = if link {
            fs::metadata(e.path()).map(|m| m.is_dir()).unwrap_or(false)
        } else {
            sm.is_dir()
        };
        if is_dir && passes(&e.file_name().to_string_lossy(), &sm, f) {
            return true;
        }
    }
    false
}

/// "Proje 2" < "Proje 10"; buyuk/kucuk harf duyarsiz.
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (mut ai, mut bi) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (ai.peek().copied(), bi.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, _) => return Ordering::Less,
            (_, None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let mut na = String::new();
                while let Some(&c) = ai.peek() {
                    if c.is_ascii_digit() {
                        na.push(c);
                        ai.next();
                    } else {
                        break;
                    }
                }
                let mut nb = String::new();
                while let Some(&c) = bi.peek() {
                    if c.is_ascii_digit() {
                        nb.push(c);
                        bi.next();
                    } else {
                        break;
                    }
                }
                let (ta, tb) = (na.trim_start_matches('0'), nb.trim_start_matches('0'));
                let o = ta.len().cmp(&tb.len()).then_with(|| ta.cmp(tb));
                if o != Ordering::Equal {
                    return o;
                }
            }
            (Some(x), Some(y)) => {
                let (lx, ly) = (
                    x.to_lowercase().next().unwrap_or(x),
                    y.to_lowercase().next().unwrap_or(y),
                );
                if lx != ly {
                    return lx.cmp(&ly);
                }
                ai.next();
                bi.next();
            }
        }
    }
}

/// Yolu kullanici dostu etikete cevirir (son bilesen).
pub fn display_name(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| p.display().to_string())
}

pub fn path_in_roots(candidate: &Path, roots: &[PathBuf]) -> bool {
    let Ok(c) = fs::canonicalize(candidate) else {
        return false;
    };
    roots
        .iter()
        .filter_map(|r| fs::canonicalize(r).ok())
        .any(|r| c.starts_with(&r))
}

#[cfg(test)]
mod tests {
    use super::*;

    const F: Filter = Filter {
        show_hidden: false,
        hide_dev: true,
    };

    #[test]
    fn natural_order() {
        let mut v = vec!["Proje 10", "proje 2", "Proje 1", "Çiçek", "Zeta", "alfa"];
        v.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(
            v,
            vec!["alfa", "Proje 1", "proje 2", "Proje 10", "Zeta", "Çiçek"]
        );
    }

    #[test]
    fn lists_only_dirs_sorted_and_filtered() {
        let t = tempfile::tempdir().unwrap();
        for d in ["b", "A", "node_modules", ".git", "Fotoğraflar/alt"] {
            fs::create_dir_all(t.path().join(d)).unwrap();
        }
        fs::write(t.path().join("dosya.txt"), "x").unwrap();
        let l = list_subfolders(t.path(), &F).unwrap();
        let names: Vec<_> = l.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, vec!["A", "b", "Fotoğraflar"]);
        assert!(
            l.iter()
                .find(|i| i.name == "Fotoğraflar")
                .unwrap()
                .has_children
        );
        assert!(!l.iter().find(|i| i.name == "A").unwrap().has_children);

        let all = list_subfolders(
            t.path(),
            &Filter {
                show_hidden: true,
                hide_dev: false,
            },
        )
        .unwrap();
        assert!(all.iter().any(|i| i.name == "node_modules"));
        assert!(all.iter().any(|i| i.name == ".git"));
    }

    #[cfg(windows)]
    #[test]
    fn hidden_attribute_respected() {
        let t = tempfile::tempdir().unwrap();
        let h = t.path().join("gizli");
        fs::create_dir(&h).unwrap();
        fs::create_dir(t.path().join("acik")).unwrap();
        std::process::Command::new("attrib")
            .arg("+h")
            .arg(&h)
            .status()
            .unwrap();
        let l = list_subfolders(t.path(), &F).unwrap();
        assert_eq!(l.len(), 1);
        let l2 = list_subfolders(
            t.path(),
            &Filter {
                show_hidden: true,
                hide_dev: true,
            },
        )
        .unwrap();
        assert_eq!(l2.len(), 2);
    }

    #[cfg(windows)]
    #[test]
    fn junction_loop_not_followed() {
        let t = tempfile::tempdir().unwrap();
        let a = t.path().join("a");
        fs::create_dir(&a).unwrap();
        let ok = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(a.join("dongu"))
            .arg(t.path())
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if ok {
            let l = list_subfolders(&a, &F).unwrap();
            assert!(l.iter().all(|i| i.name != "dongu"), "dongu uretti: {l:?}");
        }
    }

    #[test]
    fn path_in_roots_check() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("kok");
        fs::create_dir_all(root.join("a")).unwrap();
        fs::create_dir(t.path().join("disari")).unwrap();
        assert!(path_in_roots(&root.join("a"), std::slice::from_ref(&root)));
        assert!(!path_in_roots(
            &t.path().join("disari"),
            std::slice::from_ref(&root)
        ));
        assert!(!path_in_roots(&root.join("yok"), &[root]));
    }
}
