//! Otomatik guncelleme: GitHub Releases'teki imzali `latest.json` denetlenir.
//! Acilistan 30 sn sonra ve her 6 saatte bir bakilir; yeni surum varsa indirilir, imza dogrulanir ve
//! kurulur (NSIS "passive": ilerleme cubugu gosterir, uygulama kapanip yeniden acilir).
//! Hata durumunda sessizce bir sonraki denetime kalinir; kullanicinin verisine dokunulmaz.

use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

const FIRST_DELAY: Duration = Duration::from_secs(30);
const INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

pub fn start(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio_sleep(FIRST_DELAY).await;
        loop {
            if let Err(e) = check_and_install(&app).await {
                eprintln!("[updater] {e}");
            }
            tokio_sleep(INTERVAL).await;
        }
    });
}

async fn tokio_sleep(d: Duration) {
    // Ek bagimlilik eklememek icin engelleyen bekleme ayri iste calisir.
    let _ = tauri::async_runtime::spawn_blocking(move || std::thread::sleep(d)).await;
}

async fn check_and_install(app: &AppHandle) -> Result<(), tauri_plugin_updater::Error> {
    let Some(update) = app.updater()?.check().await? else {
        return Ok(());
    };
    update.download_and_install(|_, _| {}, || {}).await
}
