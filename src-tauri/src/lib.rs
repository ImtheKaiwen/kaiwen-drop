//! Kaiwen Drop - cekirdek: durum, kisayol, ayar komutlari, yasam dongusu.

pub mod drop_surface;
pub mod folders;
pub mod menu_session;
pub mod move_service;
pub mod selection;
pub mod settings;
pub mod tray;
pub mod updater;
pub mod windows;

use serde::Serialize;
use serde_json::{json, Value};
use settings::Settings;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, RunEvent};
use tauri_plugin_autostart::ManagerExt as _;
use tauri_plugin_dialog::DialogExt as _;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use windows::Rect;

/// `window_ready` ile gosterilecek, sabit yerlestirilmis pencereler (birakma alani, dock).
pub static PENDING_SHOW_ONCE: LazyLock<Mutex<HashMap<String, Rect>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct AppState {
    pub settings: Mutex<Settings>,
    pub settings_path: PathBuf,
    pub session: Mutex<Option<menu_session::Session>>,
    pub next_id: AtomicU64,
    pub warm: Mutex<Option<String>>,
    pub warming: AtomicBool,
    pub loaded: Mutex<HashSet<String>>,
    pub pending: Mutex<HashMap<String, Value>>,
    pub pending_show: Mutex<HashMap<String, (Rect, bool)>>,
    pub capturing: AtomicBool,
    pub hotkey_error: Mutex<Option<String>>,
    pub toast_gen: AtomicU64,
    pub toast_label: Mutex<Option<String>>,
    pub last_hotkey: Mutex<Instant>,
    pub last_activity: Mutex<Instant>,
}

// ------------------------------------------------------------------ kisayol

fn on_hotkey(app: &AppHandle) {
    let st = app.state::<AppState>();
    {
        let mut last = st.last_hotkey.lock().unwrap();
        if last.elapsed() < Duration::from_millis(220) {
            return; // hizli tekrar: duplicate yok
        }
        *last = Instant::now();
    }
    if menu_session::has_session(app) {
        menu_session::close_session(app); // tekrar kisayol: onceki oturumu guvenle kapat
        return;
    }
    if st.capturing.swap(true, Ordering::SeqCst) {
        return;
    }
    menu_session::touch(app);
    menu_session::ensure_warm(app); // WebView olusturma, secim yakalama ile paralel ilerler
    let app = app.clone();
    std::thread::spawn(move || {
        // Secim, HICBIR pencere acilmadan ve odak degismeden once yakalanir.
        let sources = selection::capture_foreground();
        let anchor = windows::cursor_pos();
        app.state::<AppState>()
            .capturing
            .store(false, Ordering::SeqCst);
        if sources.is_empty() {
            return; // Akis B: sifir pencere, sifir bildirim
        }
        let _ = menu_session::open_session(&app, sources, anchor);
    });
}

fn register_shortcut(app: &AppHandle, accel: &str) -> Result<(), String> {
    settings::validate_shortcut(accel)?;
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    gs.on_shortcut(accel, |app, _sc, ev| {
        if ev.state() == ShortcutState::Pressed {
            on_hotkey(app);
        }
    })
    .map_err(|e| e.to_string())
}

// ------------------------------------------------------------------ ayar komutlari

#[derive(Serialize)]
struct RootInfo {
    id: String,
    path: String,
    exists: bool,
}

#[derive(Serialize)]
struct StateDto {
    settings: Settings,
    roots: Vec<RootInfo>,
    hotkey_error: Option<String>,
    autostart: bool,
    system_dark: bool,
}

fn state_dto(app: &AppHandle) -> StateDto {
    let st = app.state::<AppState>();
    let s = st.settings.lock().unwrap().clone();
    let roots = s
        .roots
        .iter()
        .map(|r| {
            let p = settings::resolve_root(r);
            RootInfo {
                id: r.id.clone(),
                exists: p.as_ref().map(|p| p.is_dir()).unwrap_or(false),
                path: p
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
            }
        })
        .collect();
    let hotkey_error = st.hotkey_error.lock().unwrap().clone();
    let autostart = app.autolaunch().is_enabled().unwrap_or(false);
    StateDto {
        settings: s,
        roots,
        hotkey_error,
        autostart,
        system_dark: settings::system_is_dark(),
    }
}

#[tauri::command]
async fn get_state(app: AppHandle) -> StateDto {
    state_dto(&app)
}

#[derive(Serialize)]
struct SaveResult {
    state: StateDto,
    error: Option<String>,
}

#[tauri::command]
async fn save_settings(app: AppHandle, new: Settings) -> Result<SaveResult, String> {
    let st = app.state::<AppState>();
    let mut new = new.sanitize();
    // Kok kimlikleri tekil, adlar bos degil.
    let mut seen = HashSet::new();
    new.roots.retain(|r| seen.insert(r.id.clone()));
    for r in &mut new.roots {
        if r.name.trim().is_empty() {
            r.name = std::path::Path::new(&r.path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Klasör".into());
        }
    }
    let old = st.settings.lock().unwrap().clone();
    let mut error = None;
    if new.shortcut != old.shortcut {
        match register_shortcut(&app, &new.shortcut) {
            Ok(()) => *st.hotkey_error.lock().unwrap() = None,
            Err(e) => {
                error = Some(e);
                new.shortcut = old.shortcut.clone();
                let _ = register_shortcut(&app, &old.shortcut);
            }
        }
    }
    settings::save_to(&st.settings_path, &new)?;
    *st.settings.lock().unwrap() = new.clone();

    if new.language != old.language {
        tray::refresh(&app, &new.language);
    }
    if new.corner_dock != old.corner_dock {
        drop_surface::sync_dock(&app, new.corner_dock);
    }
    let _ = tauri::Emitter::emit(&app, "settings-changed", json!({}));
    Ok(SaveResult {
        state: state_dto(&app),
        error,
    })
}

/// Kisayol kaydi sirasinda global kisayolu gecici birak: mevcut kombinasyon yakalanabilsin ve yanlislikla menu acmasin.
#[tauri::command]
fn pause_shortcut(app: AppHandle, paused: bool) {
    if paused {
        let _ = app.global_shortcut().unregister_all();
    } else {
        let cur = app
            .state::<AppState>()
            .settings
            .lock()
            .unwrap()
            .shortcut
            .clone();
        let _ = register_shortcut(&app, &cur);
    }
}

#[tauri::command]
async fn pick_folder(app: AppHandle) -> Option<String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut b = app.dialog().file();
    if let Some(w) = app.get_webview_window("settings") {
        b = b.set_parent(&w);
    }
    b.pick_folder(move |p| {
        let _ = tx.send(p);
    });
    tauri::async_runtime::spawn_blocking(move || rx.recv().ok().flatten())
        .await
        .ok()
        .flatten()
        .and_then(|p| p.into_path().ok())
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
async fn validate_path(path: String) -> bool {
    std::path::Path::new(&path).is_dir()
}

#[tauri::command]
async fn set_autostart(app: AppHandle, enabled: bool) -> Result<bool, String> {
    let m = app.autolaunch();
    if enabled {
        m.enable().map_err(|e| e.to_string())?;
    } else {
        m.disable().map_err(|e| e.to_string())?;
    }
    Ok(m.is_enabled().unwrap_or(false))
}

/// Sayfa ilk cizimini bitirdi: beyaz flash olmadan goster.
#[tauri::command]
async fn window_ready(app: AppHandle, label: String) {
    let Some(w) = app.get_webview_window(&label) else {
        return;
    };
    if label == "settings" {
        let _ = w.show();
        let _ = w.set_focus();
    } else if let Some(outer) = PENDING_SHOW_ONCE.lock().unwrap().remove(&label) {
        // Birakma alani/dock: stil degistirmeden yerlestir (WS_EX_NOACTIVATE, OLE birakma hedefini bozuyordu).
        #[cfg(windows)]
        if let Ok(h) = w.hwnd() {
            windows::place_window(h, outer, true);
        }
        let _ = w.show(); // wry/WebView2 kontrolcusunu gorunur isaretler (OLE birakma hedefi icin gerekli)
    }
}

/// Gercek pencere kapatma (CSS ile gizleme degil).
#[tauri::command]
async fn window_close(app: AppHandle, label: String) {
    if label == "drop" {
        drop_surface::close(&app, "drop");
        let lang = app
            .state::<AppState>()
            .settings
            .lock()
            .unwrap()
            .language
            .clone();
        tray::refresh(&app, &lang);
    } else if let Some(w) = app.get_webview_window(&label) {
        let _ = w.destroy();
    }
}

#[tauri::command]
async fn open_settings(app: AppHandle) {
    menu_session::close_session(&app); // popup zinciri once kapanir
    show_settings(&app);
}

pub fn show_settings(app: &AppHandle) {
    let theme = app
        .state::<AppState>()
        .settings
        .lock()
        .unwrap()
        .theme
        .clone();
    windows::open_settings(app, settings::is_dark(&theme));
}

// ------------------------------------------------------------------ calistirma

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            // Ikinci baslatma: ikinci surec yok; mevcut uygulama `--drop` ile birakma alanini, aksi halde Ayarlar'i acar.
            if args.iter().any(|a| a == "--quit") {
                // Tepsideki "Cikis" ile ayni yol (test ve guncelleyici icin).
                menu_session::close_session(app);
                app.exit(0);
            } else if args.iter().any(|a| a == "--drop") {
                drop_surface::open(app, false);
            } else {
                show_settings(app);
            }
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();
            let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
            let path = dir.join("settings.json");
            let (loaded, warn) = settings::load_from(&path);
            if let Some(w) = warn {
                eprintln!("[ayarlar] {w}");
            }
            let first_run = !path.exists();
            if first_run {
                let _ = settings::save_to(&path, &loaded);
            }
            let lang = loaded.language.clone();
            let shortcut = loaded.shortcut.clone();
            let dock = loaded.corner_dock;
            app.manage(AppState {
                settings: Mutex::new(loaded),
                settings_path: path,
                session: Mutex::new(None),
                next_id: AtomicU64::new(1),
                warm: Mutex::new(None),
                warming: AtomicBool::new(false),
                loaded: Mutex::new(HashSet::new()),
                pending: Mutex::new(HashMap::new()),
                pending_show: Mutex::new(HashMap::new()),
                capturing: AtomicBool::new(false),
                hotkey_error: Mutex::new(None),
                toast_gen: AtomicU64::new(0),
                toast_label: Mutex::new(None),
                last_hotkey: Mutex::new(Instant::now() - Duration::from_secs(5)),
                last_activity: Mutex::new(Instant::now()),
            });
            tray::init(&handle, &lang)?;
            if let Err(e) = register_shortcut(&handle, &shortcut) {
                *handle.state::<AppState>().hotkey_error.lock().unwrap() = Some(e);
                menu_session::toast(
                    &handle,
                    json!({"type":"info","key":"hotkeyFailed","shortcut":shortcut}),
                    6000,
                );
            }
            #[cfg(not(debug_assertions))]
            updater::start(&handle); // yalniz surum derlemesi; hata ayiklama/E2E derlemesi aga cikmaz
            menu_session::start_janitor(&handle); // sicak pencere yalniz ilk kisayolda hazirlanir; bosta yok edilir
            if std::env::args().any(|a| a == "--drop") {
                drop_surface::open(&handle, false);
            }
            if dock {
                drop_surface::sync_dock(&handle, true);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            menu_session::popup_hello,
            menu_session::popup_ready,
            menu_session::popup_open_child,
            menu_session::popup_close_below,
            menu_session::session_close,
            menu_session::menu_key,
            menu_session::move_here,
            menu_session::dialog_resolve,
            get_state,
            save_settings,
            pick_folder,
            validate_path,
            set_autostart,
            pause_shortcut,
            window_ready,
            window_close,
            open_settings,
        ])
        .build(tauri::generate_context!())
        .expect("Kaiwen Drop baslatilamadi")
        .run(|_app, event| {
            // Son pencere kapaninca uygulama CIKMAZ (tray sureci devam eder); yalniz acik `exit()` cikarir.
            if let RunEvent::ExitRequested { code, api, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
