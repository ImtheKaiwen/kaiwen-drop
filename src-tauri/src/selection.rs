//! Windows Explorer / Masaustu secim yakalama.
//!
//! Kurallar (brief Bolum 5):
//! * Yalniz *on plandaki* Explorer penceresinin (sekmesinin) gorunur secimi okunur.
//! * Secim `IFolderView2::GetSelection(FALSE)` ile okunur; bos secim bos listedir.
//! * Clipboard, sahte klavye/fare olayi ve "son klasor" fallback'i YOKTUR.

use std::path::{Path, PathBuf};

/// On plandaki pencerenin turu (sinif adindan).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ForegroundKind {
    Explorer,
    DesktopHost,
    Other,
}

/// Saf fonksiyon: pencere sinif adindan tur cikarir.
pub fn classify_class(class: &str) -> ForegroundKind {
    match class {
        "CabinetWClass" | "ExploreWClass" => ForegroundKind::Explorer,
        "Progman" | "WorkerW" => ForegroundKind::DesktopHost,
        _ => ForegroundKind::Other,
    }
}

/// Dosya sistemi yolu tasimaya uygun mu? (mutlak, mevcut, korumasiz, sanal degil)
pub fn is_valid_source(p: &Path) -> bool {
    if !p.is_absolute() {
        return false;
    }
    let s = p.to_string_lossy().to_ascii_lowercase();
    // Sanal / ozel shell yollari ($Recycle.Bin gibi) ve sistem dizinleri.
    if s.contains("\\$recycle.bin") {
        return false;
    }
    if is_protected(p) {
        return false;
    }
    std::fs::symlink_metadata(p).is_ok()
}

/// Disk koku, Windows dizini, Program Files vb. korumali mi?
pub fn is_protected(p: &Path) -> bool {
    let comps = p.components().count();
    // "C:\" -> Prefix + RootDir = 2 bilesen
    if comps <= 2 {
        return true;
    }
    let lower = norm_lower(p);
    let mut protected: Vec<String> = Vec::new();
    for var in [
        "SystemRoot",
        "ProgramFiles",
        "ProgramFiles(x86)",
        "ProgramData",
    ] {
        if let Ok(v) = std::env::var(var) {
            protected.push(norm_lower(Path::new(&v)));
        }
    }
    protected
        .iter()
        .any(|pr| lower == *pr || lower.starts_with(&format!("{pr}\\")))
}

fn norm_lower(p: &Path) -> String {
    let s = p.to_string_lossy().replace('/', "\\");
    let s = s.strip_prefix("\\\\?\\").unwrap_or(&s).to_string();
    s.trim_end_matches('\\').to_ascii_lowercase()
}

/// Gecersiz ogeleri eler, tekrarlari kaldirir. Sonuc bossa UI hic acilmaz.
pub fn validate_sources(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    for p in paths {
        if is_valid_source(&p) && !out.iter().any(|o| norm_lower(o) == norm_lower(&p)) {
            out.push(p);
        }
    }
    out
}

#[cfg(windows)]
pub use win::capture_foreground;

#[cfg(not(windows))]
pub fn capture_foreground() -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(windows)]
mod win {
    use super::*;
    use windows::core::{Interface, PCWSTR, PWSTR};
    use windows::Win32::Foundation::{HWND, RPC_E_CHANGED_MODE};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, IServiceProvider,
        CLSCTX_ALL, COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::System::Variant::VARIANT;
    use windows::Win32::UI::Shell::{
        IFolderView2, IShellBrowser, IShellItemArray, IShellView, IShellWindows, IWebBrowserApp,
        SID_STopLevelBrowser, ShellWindows, SIGDN_FILESYSPATH, SWC_DESKTOP, SWFO_NEEDDISPATCH,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        FindWindowExW, GetAncestor, GetClassNameW, GetForegroundWindow, GetParent, GetWindow,
        GA_ROOT, GW_CHILD, GW_HWNDNEXT,
    };

    struct ComGuard(bool);
    impl Drop for ComGuard {
        fn drop(&mut self) {
            if self.0 {
                unsafe { CoUninitialize() };
            }
        }
    }

    fn class_of(hwnd: HWND) -> String {
        let mut buf = [0u16; 256];
        let n = unsafe { GetClassNameW(hwnd, &mut buf) };
        String::from_utf16_lossy(&buf[..n.max(0) as usize])
    }

    /// Guncel on plan seciminin dogrulanmis yollari. Hata veya bos secim => bos liste.
    pub fn capture_foreground() -> Vec<PathBuf> {
        match capture_inner() {
            Ok(v) => validate_sources(v),
            Err(_) => Vec::new(),
        }
    }

    fn capture_inner() -> Result<Vec<PathBuf>, String> {
        let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        let _guard = ComGuard(hr.is_ok());
        if hr.is_err() && hr != RPC_E_CHANGED_MODE {
            return Err(format!("CoInitializeEx: {hr:?}"));
        }

        let fg = unsafe { GetForegroundWindow() };
        if fg.0.is_null() {
            return Ok(Vec::new());
        }
        let root = unsafe { GetAncestor(fg, GA_ROOT) };
        let root = if root.0.is_null() { fg } else { root };

        let view = match classify_class(&class_of(root)) {
            ForegroundKind::Explorer => explorer_view(root)?,
            ForegroundKind::DesktopHost => {
                let has_defview = unsafe {
                    FindWindowExW(
                        Some(root),
                        None,
                        windows::core::w!("SHELLDLL_DefView"),
                        PCWSTR::null(),
                    )
                }
                .is_ok();
                if !has_defview {
                    return Ok(Vec::new());
                }
                desktop_view()?
            }
            ForegroundKind::Other => return Ok(Vec::new()),
        };
        selection_of(&view)
    }

    fn browser_of(disp: &windows::Win32::System::Com::IDispatch) -> Result<IShellBrowser, String> {
        let sp: IServiceProvider = disp.cast().map_err(|e| e.to_string())?;
        unsafe { sp.QueryService::<IShellBrowser>(&SID_STopLevelBrowser) }
            .map_err(|e| e.to_string())
    }

    fn desktop_view() -> Result<IShellView, String> {
        let shell: IShellWindows = unsafe { CoCreateInstance(&ShellWindows, None, CLSCTX_ALL) }
            .map_err(|e| e.to_string())?;
        let mut hwnd_out: i32 = 0;
        let loc = VARIANT::from(0i32); // CSIDL_DESKTOP
        let empty = VARIANT::default();
        let disp = unsafe {
            shell.FindWindowSW(&loc, &empty, SWC_DESKTOP, &mut hwnd_out, SWFO_NEEDDISPATCH)
        }
        .map_err(|e| e.to_string())?;
        let browser = browser_of(&disp)?;
        unsafe { browser.QueryActiveShellView() }.map_err(|e| e.to_string())
    }

    /// Foreground Explorer penceresine ait aktif sekmenin shell gorunumu.
    fn explorer_view(root: HWND) -> Result<IShellView, String> {
        let shell: IShellWindows = unsafe { CoCreateInstance(&ShellWindows, None, CLSCTX_ALL) }
            .map_err(|e| e.to_string())?;
        let count = unsafe { shell.Count() }.map_err(|e| e.to_string())?;
        // (gorunum, gorunumun pencere tutamaci): sekmeli Explorer'da pasif sekmelerin gorunum
        // penceresinin ust pencerelerinden biri gizlidir => IsWindowVisible yalniz AKTIF sekmede dogrudur.
        let mut candidates: Vec<(IShellView, HWND)> = Vec::new();
        for i in 0..count {
            let Ok(disp) = (unsafe { shell.Item(&VARIANT::from(i)) }) else {
                continue;
            };
            let Ok(app) = disp.cast::<IWebBrowserApp>() else {
                continue;
            };
            let Ok(h) = (unsafe { app.HWND() }) else {
                continue;
            };
            if h.0 != root.0 as isize {
                continue;
            }
            let Ok(browser) = browser_of(&disp) else {
                continue;
            };
            let Ok(view) = (unsafe { browser.QueryActiveShellView() }) else {
                continue;
            };
            let view_wnd = unsafe { view.GetWindow() }.unwrap_or_default();
            candidates.push((view, view_wnd));
        }
        // Tek aday: o. Birden cok aday (sekmeler): Explorer tum sekme pencerelerini ayni konumda
        // ve gorunur tutar; AKTIF sekmenin `ShellTabWindowClass` penceresi Z-sirasinda en ustte olandir.
        // Belirsizse HICBIRI (sessiz bos secim) - asla "ilk bulunan" secilmez.
        let chosen = if candidates.len() == 1 {
            candidates.into_iter().next()
        } else {
            let tabs: Vec<HWND> = candidates.iter().map(|(_, w)| tab_window_of(*w)).collect();
            topmost_tab(root, &tabs).and_then(|top| {
                candidates
                    .into_iter()
                    .zip(tabs)
                    .find(|(_, t)| t.0 == top.0)
                    .map(|(c, _)| c)
            })
        };
        let (view, _) = chosen.ok_or_else(|| "aktif Explorer sekmesi bulunamadi".to_string())?;
        Ok(view)
    }

    /// Gorunum penceresinden yukari dogru `ShellTabWindowClass` atasini bulur.
    fn tab_window_of(mut h: HWND) -> HWND {
        while !h.0.is_null() {
            if class_of(h) == "ShellTabWindowClass" {
                return h;
            }
            h = unsafe { GetParent(h) }.unwrap_or_default();
        }
        HWND::default()
    }

    /// `root` altindaki sekme pencereleri arasinda Z-sirasinda en ustte olan (aday listesinde bulunan).
    fn topmost_tab(root: HWND, wanted: &[HWND]) -> Option<HWND> {
        let mut cur = unsafe { GetWindow(root, GW_CHILD) }.ok()?;
        while !cur.0.is_null() {
            if class_of(cur) == "ShellTabWindowClass" && wanted.iter().any(|w| w.0 == cur.0) {
                return Some(cur);
            }
            cur = unsafe { GetWindow(cur, GW_HWNDNEXT) }.ok()?;
        }
        None
    }
    fn selection_of(view: &IShellView) -> Result<Vec<PathBuf>, String> {
        let fv2: IFolderView2 = view.cast().map_err(|e| e.to_string())?;
        // fnoneimplies_folder = FALSE: bos secim "klasorun kendisi" anlamina GELMEZ.
        let arr: IShellItemArray = match unsafe { fv2.GetSelection(false) } {
            Ok(a) => a,
            Err(_) => return Ok(Vec::new()),
        };
        let n = unsafe { arr.GetCount() }.map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for i in 0..n {
            let Ok(item) = (unsafe { arr.GetItemAt(i) }) else {
                continue;
            };
            let Ok(name) = (unsafe { item.GetDisplayName(SIGDN_FILESYSPATH) }) else {
                continue;
            };
            let s = pwstr_to_string(name);
            if !s.is_empty() {
                out.push(PathBuf::from(s));
            }
        }
        Ok(out)
    }

    fn pwstr_to_string(p: PWSTR) -> String {
        if p.is_null() {
            return String::new();
        }
        let s = unsafe { p.to_string() }.unwrap_or_default();
        unsafe { CoTaskMemFree(Some(p.0 as *const _)) };
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify() {
        assert_eq!(classify_class("CabinetWClass"), ForegroundKind::Explorer);
        assert_eq!(classify_class("Progman"), ForegroundKind::DesktopHost);
        assert_eq!(classify_class("WorkerW"), ForegroundKind::DesktopHost);
        assert_eq!(classify_class("Notepad"), ForegroundKind::Other);
    }

    #[test]
    fn rejects_drive_root_and_relative() {
        assert!(!is_valid_source(Path::new("C:\\")));
        assert!(!is_valid_source(Path::new("relative\\file.txt")));
    }

    #[test]
    fn rejects_missing_and_recycle() {
        assert!(!is_valid_source(Path::new("C:\\Kaiwen\\yok\\yok.txt")));
        assert!(!is_valid_source(Path::new("C:\\$Recycle.Bin\\S-1\\x")));
    }

    #[test]
    fn validate_dedups_and_filters() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("ğüş dosya.txt");
        std::fs::write(&f, "x").unwrap();
        let out = validate_sources(vec![f.clone(), f.clone(), d.path().join("yok")]);
        assert_eq!(out, vec![f]);
    }

    #[test]
    fn windows_dir_is_protected() {
        if let Ok(r) = std::env::var("SystemRoot") {
            assert!(is_protected(Path::new(&r)));
            assert!(is_protected(&Path::new(&r).join("System32")));
        }
    }
}
