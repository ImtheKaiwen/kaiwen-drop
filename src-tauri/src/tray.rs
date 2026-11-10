//! Sistem tepsisi: Ayarlar, Birakma alanini ac/kapat, Cikis.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

fn labels(lang: &str) -> (&'static str, &'static str, &'static str, &'static str) {
    if lang == "en-US" {
        ("Settings", "Open drop zone", "Close drop zone", "Quit")
    } else {
        (
            "Ayarlar",
            "Bırakma alanını aç",
            "Bırakma alanını kapat",
            "Çıkış",
        )
    }
}

fn build_menu(app: &AppHandle, lang: &str) -> tauri::Result<Menu<tauri::Wry>> {
    let (settings, open_drop, close_drop, quit) = labels(lang);
    let drop_open = app.get_webview_window("drop").is_some();
    let m = Menu::new(app)?;
    m.append(&MenuItem::with_id(
        app,
        "settings",
        settings,
        true,
        None::<&str>,
    )?)?;
    m.append(&MenuItem::with_id(
        app,
        "drop",
        if drop_open { close_drop } else { open_drop },
        true,
        None::<&str>,
    )?)?;
    m.append(&PredefinedMenuItem::separator(app)?)?;
    m.append(&MenuItem::with_id(app, "quit", quit, true, None::<&str>)?)?;
    Ok(m)
}

pub fn init(app: &AppHandle, lang: &str) -> tauri::Result<()> {
    let menu = build_menu(app, lang)?;
    let mut b = TrayIconBuilder::with_id("main")
        .tooltip("Kaiwen Drop")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, ev| match ev.id().as_ref() {
            "settings" => crate::show_settings(app),
            "drop" => crate::drop_surface::toggle_surface(app),
            "quit" => {
                crate::menu_session::close_session(app);
                app.exit(0);
            }
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        b = b.icon(icon.clone());
    }
    b.build(app)?;
    Ok(())
}

/// Dil veya birakma alani durumu degisince menuyu yeniler.
pub fn refresh(app: &AppHandle, lang: &str) {
    if let (Some(tray), Ok(menu)) = (app.tray_by_id("main"), build_menu(app, lang)) {
        let _ = tray.set_menu(Some(menu));
    }
}
