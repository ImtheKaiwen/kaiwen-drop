//! Birakma alani (Akis C) ve istege bagli kose dock'u.
//! Gercek Windows yollari Rust tarafinda yerel surukle-birak olayindan alinir (File API / RAM kopyasi yok).

use crate::windows::{self as win, Rect};
use crate::{menu_session, selection};
use serde_json::json;
use tauri::{
    AppHandle, DragDropEvent, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};

const SURFACE: (f64, f64) = (200.0, 120.0);
const DOCK: (f64, f64) = (44.0, 44.0);

pub fn toggle_surface(app: &AppHandle) {
    if app.get_webview_window("drop").is_some() {
        close(app, "drop");
    } else {
        open(app, false);
    }
    let lang = app
        .state::<crate::AppState>()
        .settings
        .lock()
        .unwrap()
        .language
        .clone();
    crate::tray::refresh(app, &lang);
}

pub fn close(app: &AppHandle, label: &str) {
    if let Some(w) = app.get_webview_window(label) {
        let _ = w.destroy();
    }
}

pub fn open(app: &AppHandle, dock: bool) {
    let label = if dock { "dock" } else { "drop" };
    if app.get_webview_window(label).is_some() {
        return;
    }
    let (w, h) = if dock { DOCK } else { SURFACE };
    let url = format!("drop.html?mode={}", if dock { "dock" } else { "surface" });
    let theme = app
        .state::<crate::AppState>()
        .settings
        .lock()
        .unwrap()
        .theme
        .clone();
    let dark = crate::settings::is_dark(&theme);
    let bg = if dark {
        tauri::window::Color(27, 27, 27, 255)
    } else {
        tauri::window::Color(255, 255, 255, 255)
    };
    let built = WebviewWindowBuilder::new(app, label, WebviewUrl::App(url.into()))
        .title("Kaiwen Drop")
        .inner_size(w, h)
        .decorations(false)
        .transparent(false) // seffaf pencerelerde OLE birakma hedefi calismiyor; opak + DWM yuvarlak kose/golge
        .shadow(true)
        .background_color(bg)
        .resizable(false)
        .visible(false)
        .always_on_top(true)
        .maximizable(false)
        .minimizable(false)
        .build();
    let Ok(win_) = built else { return };

    // Konum: calisma alani sag alt (yuzey) veya sag kenar ortasi (dock).
    let cur = win::cursor_pos();
    let mon = win::monitor_at(cur.0, cur.1);
    let (wp, hp) = (
        (w * mon.scale).round() as i32,
        (h * mon.scale).round() as i32,
    );
    let m = 0; // opak pencere: seffaf kenar boslugu yok
    let edge = (24.0 * mon.scale).round() as i32;
    let content = if dock {
        Rect::new(
            mon.work.r - wp,
            mon.work.t + (mon.work.h() - hp) / 2,
            wp,
            hp,
        )
    } else {
        Rect::new(mon.work.r - wp - edge, mon.work.b - hp - edge, wp, hp)
    };
    let outer = content.inflate(m);
    #[cfg(windows)]
    if let Ok(h) = win_.hwnd() {
        win::round_corners(h, if dark { 0x0033_3333 } else { 0x00E7_E9E9 });
        win::apply_popup_style(h, true); // aktive olabilir (NOACTIVATE yok); yalniz arac penceresi stili
        win::place_window(h, outer, false); // gorunmez yerlestir; sayfa `window_ready` diyince gosterilir
    }
    crate::PENDING_SHOW_ONCE
        .lock()
        .unwrap()
        .insert(label.to_string(), outer);

    // Gercek yollar yerel surukle-birak olayindan gelir (Tauri 2: pencere olayi).
    let app2 = app.clone();
    let label2 = label.to_string();
    win_.on_window_event(move |ev| {
        if let WindowEvent::DragDrop(d) = ev {
            #[cfg(debug_assertions)]
            eprintln!("[drag] {d:?}");
            match d {
                DragDropEvent::Enter { .. } | DragDropEvent::Over { .. } => {
                    let _ = app2.emit_to(label2.as_str(), "drop-hover", true);
                }
                DragDropEvent::Leave => {
                    let _ = app2.emit_to(label2.as_str(), "drop-hover", false);
                }
                DragDropEvent::Drop { paths, .. } => {
                    let _ = app2.emit_to(label2.as_str(), "drop-hover", false);
                    let (app3, paths) = (app2.clone(), paths.clone());
                    std::thread::spawn(move || {
                        let sources = selection::validate_sources(paths);
                        if sources.is_empty() {
                            menu_session::toast(
                                &app3,
                                json!({"type":"info","key":"unsupported"}),
                                3000,
                            );
                        } else {
                            let _ = menu_session::open_session(&app3, sources, win::cursor_pos());
                        }
                    });
                }
                _ => {}
            }
        }
    });
}
/// Ayara gore dock'u acar/kapatir.
pub fn sync_dock(app: &AppHandle, enabled: bool) {
    if enabled {
        open(app, true);
    } else {
        close(app, "dock");
    }
}
