//! Kalici ayarlar: surumlu JSON sema, bozuk dosya kurtarma, atomik yazma.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_SHORTCUT: &str = "Ctrl+Alt+M";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Root {
    pub id: String,
    pub name: String,
    /// Sabit yol. `known` doluysa yok sayilir.
    #[serde(default)]
    pub path: String,
    /// "documents": Windows Known Folder (OneDrive yonlendirmesine dayanikli).
    #[serde(default)]
    pub known: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Settings {
    pub version: u32,
    pub roots: Vec<Root>,
    pub language: String, // tr-TR | en-US
    pub theme: String,    // system | light | dark
    pub shortcut: String, // "Ctrl+Alt+M"
    pub density: String,  // normal | compact
    pub show_hidden: bool,
    pub hide_dev_folders: bool,
    pub confirm_move: bool,
    pub corner_dock: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let mut roots = Vec::new();
        if Path::new("C:\\Dev").is_dir() {
            roots.push(Root {
                id: "dev".into(),
                name: "Dev".into(),
                path: "C:\\Dev".into(),
                known: None,
            });
        }
        roots.push(Root {
            id: "documents".into(),
            name: "Documents".into(),
            path: String::new(),
            known: Some("documents".into()),
        });
        Settings {
            version: SCHEMA_VERSION,
            roots,
            language: "tr-TR".into(),
            theme: "system".into(),
            shortcut: DEFAULT_SHORTCUT.into(),
            density: "normal".into(),
            show_hidden: false,
            hide_dev_folders: true,
            confirm_move: true,
            corner_dock: false,
        }
    }
}

impl Settings {
    /// Gelen (UI'dan) degerleri guvenli araliga ceker.
    pub fn sanitize(mut self) -> Self {
        if !["tr-TR", "en-US"].contains(&self.language.as_str()) {
            self.language = "tr-TR".into();
        }
        if !["system", "light", "dark"].contains(&self.theme.as_str()) {
            self.theme = "system".into();
        }
        if !["normal", "compact"].contains(&self.density.as_str()) {
            self.density = "normal".into();
        }
        if validate_shortcut(&self.shortcut).is_err() {
            self.shortcut = DEFAULT_SHORTCUT.into();
        }
        self.version = SCHEMA_VERSION;
        self
    }
}

/// En az bir degistirici + tek bir ana tus.
pub fn validate_shortcut(s: &str) -> Result<(), String> {
    let parts: Vec<&str> = s
        .split('+')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() < 2 {
        return Err("en az bir degistirici tus gerekli".into());
    }
    let is_mod = |p: &str| {
        matches!(
            p.to_ascii_lowercase().as_str(),
            "ctrl" | "control" | "alt" | "shift" | "win" | "super" | "meta"
        )
    };
    let (mods, keys): (Vec<&str>, Vec<&str>) = parts.iter().partition(|p| is_mod(p));
    if mods.is_empty() || keys.len() != 1 {
        return Err("gecersiz kisayol".into());
    }
    Ok(())
}

/// Dizi yolundan yukler. Bozuksa yedekler ve varsayilanlara doner.
pub fn load_from(path: &Path) -> (Settings, Option<String>) {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return (Settings::default(), None),
        Err(e) => return (Settings::default(), Some(format!("okunamadi: {e}"))),
    };
    let text = text.trim_start_matches('\u{feff}');
    let parsed: Result<Settings, _> = serde_json::from_str(text);
    match parsed {
        Ok(s) if s.version <= SCHEMA_VERSION => (migrate(s).sanitize(), None),
        _ => {
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let bak = path.with_extension(format!("json.bak-{stamp}"));
            let _ = fs::rename(path, &bak);
            (
                Settings::default(),
                Some(format!("ayar dosyasi bozuktu, yedek: {}", bak.display())),
            )
        }
    }
}

fn migrate(mut s: Settings) -> Settings {
    // Su an tek sema surumu var; ileride surum adimlari burada zincirlenir.
    if s.version == 0 {
        s.version = 1;
    }
    s
}

/// Atomik kaydet (gecici dosya + rename).
pub fn save_to(path: &Path, s: &Settings) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
    fs::write(&tmp, json.as_bytes()).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())
}

/// Kokun gercek yolu (Known Folder dahil).
pub fn resolve_root(r: &Root) -> Option<PathBuf> {
    match r.known.as_deref() {
        Some("documents") => known_documents(),
        _ if !r.path.trim().is_empty() => Some(PathBuf::from(&r.path)),
        _ => None,
    }
}

#[cfg(windows)]
pub fn known_documents() -> Option<PathBuf> {
    use windows::Win32::System::Com::CoTaskMemFree;
    use windows::Win32::UI::Shell::{FOLDERID_Documents, SHGetKnownFolderPath, KF_FLAG_DEFAULT};
    unsafe {
        let p = SHGetKnownFolderPath(&FOLDERID_Documents, KF_FLAG_DEFAULT, None).ok()?;
        let s = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as *const _));
        s.map(PathBuf::from)
    }
}

#[cfg(not(windows))]
pub fn known_documents() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Documents"))
}

/// Windows uygulama temasi koyu mu? (kayit defteri, yalniz okuma)
#[cfg(windows)]
pub fn system_is_dark() -> bool {
    use windows::core::w;
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
    let mut data: u32 = 1;
    let mut size = 4u32;
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut data as *mut u32 as *mut _),
            Some(&mut size),
        )
    };
    r.is_ok() && data == 0
}

#[cfg(not(windows))]
pub fn system_is_dark() -> bool {
    false
}

pub fn is_dark(theme: &str) -> bool {
    match theme {
        "dark" => true,
        "light" => false,
        _ => system_is_dark(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_with_turkish() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("settings.json");
        let mut s = Settings::default();
        s.roots.push(Root {
            id: "x".into(),
            name: "Fotoğraflar İş".into(),
            path: "D:\\Çalışma".into(),
            known: None,
        });
        save_to(&p, &s).unwrap();
        let (l, w) = load_from(&p);
        assert!(w.is_none());
        assert_eq!(l, s);
    }

    #[test]
    fn missing_file_gives_defaults() {
        let d = tempfile::tempdir().unwrap();
        let (s, w) = load_from(&d.path().join("yok.json"));
        assert!(w.is_none());
        assert_eq!(s.shortcut, "Ctrl+Alt+M");
        assert!(s
            .roots
            .iter()
            .any(|r| r.known.as_deref() == Some("documents")));
    }

    #[test]
    fn corrupt_file_backed_up() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("settings.json");
        fs::write(&p, "{ bozuk json").unwrap();
        let (s, w) = load_from(&p);
        assert!(w.is_some());
        assert_eq!(s, Settings::default());
        assert!(!p.exists());
        let baks = fs::read_dir(d.path())
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .contains(".bak-")
            })
            .count();
        assert_eq!(baks, 1);
    }

    #[test]
    fn partial_file_filled_with_defaults() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("settings.json");
        fs::write(&p, "{\"version\":1,\"language\":\"en-US\"}").unwrap();
        let (s, _) = load_from(&p);
        assert_eq!(s.language, "en-US");
        assert_eq!(s.theme, "system");
    }

    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn sanitize_rejects_bad_values() {
        let mut s = Settings::default();
        s.language = "xx".into();
        s.shortcut = "M".into();
        s.theme = "pink".into();
        let s = s.sanitize();
        assert_eq!(
            (s.language.as_str(), s.shortcut.as_str(), s.theme.as_str()),
            ("tr-TR", "Ctrl+Alt+M", "system")
        );
    }

    #[test]
    fn shortcut_validation() {
        assert!(validate_shortcut("Ctrl+Alt+M").is_ok());
        assert!(validate_shortcut("Ctrl+Shift+F5").is_ok());
        assert!(validate_shortcut("M").is_err());
        assert!(validate_shortcut("Ctrl+Alt").is_err());
        assert!(validate_shortcut("Ctrl+A+B").is_err());
    }

    #[test]
    fn root_add_remove_keeps_disk() {
        let d = tempfile::tempdir().unwrap();
        let real = d.path().join("gercek");
        fs::create_dir(&real).unwrap();
        fs::write(real.join("f.txt"), "1").unwrap();
        let mut s = Settings::default();
        s.roots.push(Root {
            id: "r".into(),
            name: "R".into(),
            path: real.to_string_lossy().into(),
            known: None,
        });
        s.roots.retain(|r| r.id != "r"); // yalniz kayit silinir
        assert!(real.join("f.txt").exists());
    }

    #[test]
    fn documents_resolves() {
        let r = Root {
            id: "d".into(),
            name: "D".into(),
            path: String::new(),
            known: Some("documents".into()),
        };
        assert!(resolve_root(&r).is_some());
    }
}
