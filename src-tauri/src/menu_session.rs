//! Tek merkezi menu oturumu yoneticisi.
//!
//! Her tetikleme = en fazla bir `Session`. Oturumun tum gecici pencereleri (kok menu, alt menuler,
//! onay diyalogu) bu yapida izlenir; Esc / disari tiklama / islem sonu / tekrar kisayol hepsini kapatir.
//! Pencereler yalnizca `Idle` disinda vardir; bos secimde hic pencere olusturulmaz.

use crate::folders::{self, Filter, FolderItem};
use crate::move_service::{self, ItemOutcome, MoveResult, Plan, Policy};
use crate::settings::{self, Settings};
use crate::windows::{self as win, Rect, MARGIN};
use crate::AppState;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

const GAP: i32 = 2; // alt menu ile ana menu arasi (fiziksel degil, mantiksal; olcekle carpilir)
const PAD: f64 = 6.0;
/// Sicak (onceden yuklenmis, gizli) pencere son kullanimdan sonra bu sure korunur.
const WARM_TTL: Duration = Duration::from_secs(90);

pub struct Popup {
    pub label: String,
    pub depth: usize,
    /// Fiziksel px, kenar boslugu HARIC (gorunen yuzey).
    pub content: Rect,
    pub left: bool,
    pub path: Option<PathBuf>,
    pub seq: u64,
    pub scale: f64,
}

pub struct Session {
    pub id: u64,
    pub sources: Vec<PathBuf>,
    pub popups: Vec<Popup>,
    pub plan: Option<Plan>,
    pub stop: Arc<AtomicBool>,
}

// ------------------------------------------------------------------ boyut hesabi (saf)

pub fn menu_size(
    rows: usize,
    move_here: bool,
    density: &str,
    longest: usize,
    work_h_logical: f64,
) -> (f64, f64) {
    let row = if density == "compact" { 30.0 } else { 34.0 };
    let n = rows + usize::from(move_here);
    let sep = if move_here && rows > 0 { 9.0 } else { 0.0 };
    let h =
        (2.0 * PAD + n as f64 * row + sep + 2.0).min((work_h_logical - 32.0).clamp(80.0, 560.0));
    let w = (longest as f64 * 7.0 + 76.0).clamp(212.0, 300.0);
    (w, h)
}

// ------------------------------------------------------------------ pencere havuzu

fn new_label(app: &AppHandle) -> String {
    let st = app.state::<AppState>();
    format!("pw-{}", st.next_id.fetch_add(1, Ordering::SeqCst))
}

/// Havuzdaki sicak pencereyi verir (yoksa uretir) ve yenisini arka planda hazirlar.
pub fn acquire(app: &AppHandle) -> Result<WebviewWindow, String> {
    let st = app.state::<AppState>();
    touch(app);
    let warm = st.warm.lock().unwrap().take();
    let w = warm.and_then(|l| app.get_webview_window(&l));
    let w = match w {
        Some(w) => w,
        None => win::build_popup(app, &new_label(app)).map_err(|e| e.to_string())?,
    };
    ensure_warm(app);
    Ok(w)
}

pub fn ensure_warm(app: &AppHandle) {
    let st = app.state::<AppState>();
    if st.warm.lock().unwrap().is_some() || st.warming.swap(true, Ordering::SeqCst) {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        let label = new_label(&app);
        let ok = win::build_popup(&app, &label).is_ok();
        let st = app.state::<AppState>();
        if ok {
            let mut g = st.warm.lock().unwrap();
            if g.is_none() {
                *g = Some(label);
            } else if let Some(w) = app.get_webview_window(&label) {
                let _ = w.destroy();
            }
        }
        st.warming.store(false, Ordering::SeqCst);
    });
}

/// Son etkinlik damgasi (sicak havuzun ne zaman bosaltilacagini belirler).
pub fn touch(app: &AppHandle) {
    *app.state::<AppState>().last_activity.lock().unwrap() = Instant::now();
}

/// Bosta kalinca sicak pencereyi yok eder: bosta RAM = yalniz tray sureci.
/// Her 30 sn uyanir (ihmal edilebilir CPU); oturum/bildirim varken dokunmaz.
pub fn start_janitor(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(30));
        let st = app.state::<AppState>();
        let idle = st.last_activity.lock().unwrap().elapsed() > WARM_TTL;
        if !idle || has_session(&app) || st.toast_label.lock().unwrap().is_some() {
            continue;
        }
        let label = st.warm.lock().unwrap().take();
        if let Some(l) = label {
            st.loaded.lock().unwrap().remove(&l);
            st.pending.lock().unwrap().remove(&l);
            if let Some(w) = app.get_webview_window(&l) {
                let _ = w.destroy();
            }
        }
    });
}

/// Pencereyi emekli eder: havuz bossa sifirlayip saklar, degilse yok eder.
pub fn retire(app: &AppHandle, label: &str) {
    let st = app.state::<AppState>();
    st.pending.lock().unwrap().remove(label);
    st.pending_show.lock().unwrap().remove(label);
    let Some(w) = app.get_webview_window(label) else {
        return;
    };
    let mut warm = st.warm.lock().unwrap();
    if warm.is_none() && st.loaded.lock().unwrap().contains(label) {
        let _ = w.hide();
        let _ = w.emit("popup-reset", ());
        *warm = Some(label.to_string());
    } else {
        drop(warm);
        st.loaded.lock().unwrap().remove(label);
        let _ = w.destroy();
    }
}

/// Konfigurasyonu pencereye iletir (sayfa hazirsa olay, degilse `popup_hello` bekler).
fn deliver(app: &AppHandle, label: &str, cfg: Value) {
    let st = app.state::<AppState>();
    let loaded = st.loaded.lock().unwrap().contains(label);
    if loaded {
        let _ = app.emit_to(label, "popup-config", cfg);
    } else {
        st.pending.lock().unwrap().insert(label.to_string(), cfg);
    }
}

/// `popup_ready` geldiginde (veya zaman asiminda) gosterilecek yerlesim.
fn schedule_show(app: &AppHandle, label: &str, outer: Rect, activate: bool) {
    let st = app.state::<AppState>();
    st.pending_show
        .lock()
        .unwrap()
        .insert(label.to_string(), (outer, activate));
    let (app2, label2) = (app.clone(), label.to_string());
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(1500));
        show_now(&app2, &label2); // sayfa hic hazir demediyse
    });
}

fn show_now(app: &AppHandle, label: &str) {
    let st = app.state::<AppState>();
    let entry = st.pending_show.lock().unwrap().remove(label);
    if let (Some((outer, act)), Some(w)) = (entry, app.get_webview_window(label)) {
        win::show_at(&w, outer, act);
    }
}

fn settings_snapshot(app: &AppHandle) -> Settings {
    app.state::<AppState>().settings.lock().unwrap().clone()
}

fn base_cfg(s: &Settings) -> Value {
    json!({ "density": s.density, "theme": s.theme, "lang": s.language })
}

fn merge(mut a: Value, b: Value) -> Value {
    if let (Some(am), Some(bm)) = (a.as_object_mut(), b.as_object()) {
        for (k, v) in bm {
            am.insert(k.clone(), v.clone());
        }
    }
    a
}

fn filter_of(s: &Settings) -> Filter {
    Filter {
        show_hidden: s.show_hidden,
        hide_dev: s.hide_dev_folders,
    }
}

fn resolved_roots(s: &Settings) -> Vec<(String, PathBuf)> {
    s.roots
        .iter()
        .filter_map(|r| settings::resolve_root(r).map(|p| (r.name.clone(), p)))
        .collect()
}

// ------------------------------------------------------------------ oturum

/// Gecerli kaynaklarla oturum acar. Cagirmadan once `selection::validate_sources` uygulanmis olmali.
pub fn open_session(
    app: &AppHandle,
    sources: Vec<PathBuf>,
    anchor: (i32, i32),
) -> Result<(), String> {
    if sources.is_empty() {
        return Ok(());
    }
    close_session(app);
    let s = settings_snapshot(app);
    let filter = filter_of(&s);
    let items: Vec<FolderItem> = resolved_roots(&s)
        .into_iter()
        .filter(|(_, p)| p.is_dir())
        .map(|(name, p)| FolderItem {
            name,
            has_children: folders::has_subdir(&p, &filter),
            path: p.to_string_lossy().to_string(),
        })
        .collect();
    if items.is_empty() {
        toast(app, json!({"type":"info","key":"noRoots"}), 3500);
        return Ok(());
    }

    let mon = win::monitor_at(anchor.0, anchor.1);
    let longest = items
        .iter()
        .map(|i| i.name.chars().count())
        .max()
        .unwrap_or(0);
    let (w, h) = menu_size(
        items.len(),
        false,
        &s.density,
        longest,
        mon.work.h() as f64 / mon.scale,
    );
    let (wp, hp) = (
        (w * mon.scale).round() as i32,
        (h * mon.scale).round() as i32,
    );
    let (x, y, left) = win::place_root(anchor, wp, hp, mon.work);
    let content = Rect::new(x, y, wp, hp);

    let st = app.state::<AppState>();
    let id = st.next_id.fetch_add(1, Ordering::SeqCst);
    let stop = Arc::new(AtomicBool::new(false));

    let window = acquire(app)?;
    let label = window.label().to_string();
    let cfg = merge(
        base_cfg(&s),
        json!({"kind":"menu","session":id,"depth":0,"items":items,"moveHere":false,"size":[w,h],"keyboard":false,"label":label}),
    );
    {
        let mut g = st.session.lock().unwrap();
        *g = Some(Session {
            id,
            sources,
            popups: vec![Popup {
                label: label.clone(),
                depth: 0,
                content,
                left,
                path: None,
                seq: 0,
                scale: mon.scale,
            }],
            plan: None,
            stop: stop.clone(),
        });
    }
    deliver(app, &label, cfg);
    let m = (MARGIN as f64 * mon.scale).round() as i32;
    schedule_show(app, &label, content.inflate(m), true);
    watch(app.clone(), id, stop);
    Ok(())
}

pub fn close_session(app: &AppHandle) {
    let st = app.state::<AppState>();
    let sess = st.session.lock().unwrap().take();
    if let Some(s) = sess {
        s.stop.store(true, Ordering::SeqCst);
        for p in s.popups {
            retire(app, &p.label);
        }
    }
}

fn close_if(app: &AppHandle, id: u64) {
    let same = app
        .state::<AppState>()
        .session
        .lock()
        .unwrap()
        .as_ref()
        .map(|s| s.id)
        == Some(id);
    if same {
        close_session(app);
    }
}

pub fn has_session(app: &AppHandle) -> bool {
    app.state::<AppState>().session.lock().unwrap().is_some()
}

/// Oturum suresince calisan hafif izleyici: disari tiklama, Esc, odak kaybi. Oturum bitince sonlanir (bosta CPU yok).
fn watch(app: AppHandle, id: u64, stop: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let any_btn = || {
            win::is_down(win::VK_LBUTTON)
                || win::is_down(win::VK_RBUTTON)
                || win::is_down(win::VK_MBUTTON)
        };
        let mut prev_btn = any_btn();
        let mut prev_esc = win::is_down(win::VK_ESCAPE);
        let started = Instant::now();
        let mut seen_ours = false;
        let mut lost_since: Option<Instant> = None;
        loop {
            std::thread::sleep(Duration::from_millis(30));
            if stop.load(Ordering::SeqCst) {
                break;
            }
            let btn = any_btn();
            if btn && !prev_btn {
                let (cx, cy) = win::cursor_pos();
                let inside = {
                    let st = app.state::<AppState>();
                    let g = st.session.lock().unwrap();
                    match g.as_ref() {
                        Some(s) if s.id == id => s
                            .popups
                            .iter()
                            .any(|p| p.content.inflate(2).contains(cx, cy)),
                        _ => true,
                    }
                };
                if !inside {
                    close_if(&app, id);
                    break;
                }
            }
            prev_btn = btn;
            let esc = win::is_down(win::VK_ESCAPE);
            if esc && !prev_esc {
                close_if(&app, id);
                break;
            }
            prev_esc = esc;
            if win::foreground_is_ours() {
                seen_ours = true;
                lost_since = None;
            } else if seen_ours && started.elapsed() > Duration::from_millis(400) {
                // Gecici odak akislarina (diyalog gecisi) tolerans: kalici kayip =  oturum biter.
                let since = *lost_since.get_or_insert_with(Instant::now);
                if since.elapsed() > Duration::from_millis(600) {
                    close_if(&app, id);
                    break;
                }
            }
        }
    });
}

// ------------------------------------------------------------------ komutlar (webview'dan)

#[tauri::command]
pub async fn popup_hello(app: AppHandle, label: String) -> Option<Value> {
    let st = app.state::<AppState>();
    st.loaded.lock().unwrap().insert(label.clone());
    // Bekleyen yapilandirma yoksa pencere sicak havuza girer (bir sonraki acilis icin hazir).
    let cfg = st.pending.lock().unwrap().remove(&label);
    cfg
}

#[tauri::command]
pub async fn popup_ready(app: AppHandle, label: String) {
    show_now(&app, &label);
}

#[tauri::command]
pub async fn popup_open_child(
    app: AppHandle,
    label: String,
    path: String,
    row_top: f64,
    keyboard: bool,
) -> Result<(), String> {
    let st = app.state::<AppState>();
    // 1) ana popup'i bul, ayni cocuk zaten aciksa cik, derin popup'lari kapat
    let (sid, seq, parent_rect, parent_left, scale, doomed) = {
        let mut g = st.session.lock().unwrap();
        let Some(s) = g.as_mut() else { return Ok(()) };
        let Some(idx) = s.popups.iter().position(|p| p.label == label) else {
            return Ok(());
        };
        if s.popups.get(idx + 1).and_then(|p| p.path.as_deref()) == Some(Path::new(&path)) {
            return Ok(());
        }
        s.popups[idx].seq += 1;
        let doomed: Vec<Popup> = s.popups.drain(idx + 1..).collect();
        let p = &s.popups[idx];
        (s.id, p.seq, p.content, p.left, p.scale, doomed)
    };
    for p in doomed {
        retire(&app, &p.label);
    }

    // 2) dizin listesi (UI thread'ini kilitlemez)
    let settings = settings_snapshot(&app);
    let filter = filter_of(&settings);
    let dir = PathBuf::from(&path);
    let items = tauri::async_runtime::spawn_blocking({
        let dir = dir.clone();
        move || folders::list_subfolders(&dir, &filter)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;
    if items.is_empty() {
        return Ok(()); // bos alt menu uretme
    }

    // 3) hala gecerli mi?
    let still = |app: &AppHandle| {
        let st = app.state::<AppState>();
        let g = st.session.lock().unwrap();
        g.as_ref()
            .map(|s| {
                s.id == sid
                    && s.popups.iter().any(|p| p.label == label && p.seq == seq)
                    && s.popups.last().map(|p| p.label == label).unwrap_or(false)
            })
            .unwrap_or(false)
    };
    if !still(&app) {
        return Ok(());
    }

    let mon = win::monitor_at(parent_rect.r, parent_rect.t + (row_top * scale) as i32);
    let longest = items
        .iter()
        .map(|i| i.name.chars().count())
        .max()
        .unwrap_or(0);
    let (w, h) = menu_size(
        items.len(),
        true,
        &settings.density,
        longest.max(14),
        mon.work.h() as f64 / mon.scale,
    );
    let (wp, hp) = (
        (w * mon.scale).round() as i32,
        (h * mon.scale).round() as i32,
    );
    let row_top_screen = parent_rect.t + (row_top * scale).round() as i32;
    let gap = (GAP as f64 * mon.scale).round() as i32;
    let pad = (PAD * mon.scale).round() as i32;
    let (x, y, left) = win::place_child(
        parent_rect,
        row_top_screen,
        wp,
        hp,
        mon.work,
        parent_left,
        gap,
        pad,
    );
    let content = Rect::new(x, y, wp, hp);

    let window = acquire(&app)?;
    let child_label = window.label().to_string();
    let depth;
    {
        let mut g = st.session.lock().unwrap();
        let ok = g
            .as_ref()
            .map(|s| {
                s.id == sid
                    && s.popups
                        .last()
                        .map(|p| p.label == label && p.seq == seq)
                        .unwrap_or(false)
            })
            .unwrap_or(false);
        if !ok {
            drop(g);
            retire(&app, &child_label);
            return Ok(());
        }
        let s = g.as_mut().unwrap();
        depth = s.popups.len();
        s.popups.push(Popup {
            label: child_label.clone(),
            depth,
            content,
            left,
            path: Some(dir),
            seq: 0,
            scale: mon.scale,
        });
    }
    let cfg = merge(
        base_cfg(&settings),
        json!({"kind":"menu","session":sid,"depth":depth,"items":items,"moveHere":true,"size":[w,h],"keyboard":keyboard,"label":child_label,"path":path}),
    );
    deliver(&app, &child_label, cfg);
    let m = (MARGIN as f64 * mon.scale).round() as i32;
    schedule_show(&app, &child_label, content.inflate(m), false);
    Ok(())
}

/// `label`'in altindaki tum popup'lari kapatir (farkli satira/dosyaya gelince).
#[tauri::command]
pub async fn popup_close_below(app: AppHandle, label: String) {
    let st = app.state::<AppState>();
    let doomed: Vec<Popup> = {
        let mut g = st.session.lock().unwrap();
        let Some(s) = g.as_mut() else { return };
        let Some(idx) = s.popups.iter().position(|p| p.label == label) else {
            return;
        };
        s.popups[idx].seq += 1;
        s.popups.drain(idx + 1..).collect()
    };
    for p in doomed {
        retire(&app, &p.label);
    }
}

#[tauri::command]
pub async fn session_close(app: AppHandle) {
    close_session(&app);
}

/// Klavye: yalniz kok popup odakta; tuslar en derin popup'a yonlendirilir.
#[tauri::command]
pub async fn menu_key(app: AppHandle, key: String) {
    if key == "Escape" {
        close_session(&app);
        return;
    }
    let st = app.state::<AppState>();
    let (deepest, depth) = {
        let g = st.session.lock().unwrap();
        match g.as_ref().and_then(|s| s.popups.last()) {
            Some(p) => (p.label.clone(), p.depth),
            None => return,
        }
    };
    if key == "ArrowLeft" && depth > 0 {
        let doomed = {
            let mut g = st.session.lock().unwrap();
            g.as_mut().and_then(|s| s.popups.pop())
        };
        if let Some(p) = doomed {
            retire(&app, &p.label);
        }
        return;
    }
    let _ = app.emit_to(deepest.as_str(), "popup-key", key);
}

// ------------------------------------------------------------------ tasima

fn dest_label(s: &Settings, dest: &Path) -> String {
    let Ok(c) = std::fs::canonicalize(dest) else {
        return folders::display_name(dest);
    };
    for (name, root) in resolved_roots(s) {
        if let Ok(rc) = std::fs::canonicalize(&root) {
            if let Ok(rel) = c.strip_prefix(&rc) {
                let mut parts = vec![name];
                parts.extend(
                    rel.components()
                        .map(|x| x.as_os_str().to_string_lossy().to_string()),
                );
                return parts.join(" / ");
            }
        }
    }
    folders::display_name(dest)
}

#[tauri::command]
pub async fn move_here(app: AppHandle, label: String, path: String) -> Result<(), String> {
    let st = app.state::<AppState>();
    let settings = settings_snapshot(&app);
    let dest = PathBuf::from(&path);

    let (sid, sources) = {
        let g = st.session.lock().unwrap();
        match g.as_ref() {
            Some(s) if s.popups.iter().any(|p| p.label == label) => (s.id, s.sources.clone()),
            _ => return Ok(()),
        }
    };
    // Hedef yalniz tanimli koklerin altinda olabilir.
    let roots: Vec<PathBuf> = resolved_roots(&settings)
        .into_iter()
        .map(|(_, p)| p)
        .collect();
    if !folders::path_in_roots(&dest, &roots) {
        close_session(&app);
        toast(&app, json!({"type":"error","code":"dest_missing"}), 4000);
        return Err("hedef kok klasorlerin disinda".into());
    }

    let (d2, s2) = (dest.clone(), sources.clone());
    let plan = tauri::async_runtime::spawn_blocking(move || move_service::plan(&s2, &d2))
        .await
        .map_err(|e| e.to_string())?;
    let plan = match plan {
        Ok(p) => p,
        Err(e) => {
            close_session(&app);
            toast(
                &app,
                json!({"type":"error","code":format!("{:?}", e.kind).to_lowercase()}),
                4000,
            );
            return Ok(());
        }
    };
    let dlabel = dest_label(&settings, &dest);

    if settings.confirm_move || plan.conflicts() > 0 {
        show_dialog(&app, sid, plan, &settings, &dlabel)?;
    } else {
        close_session(&app);
        run_move(
            &app,
            plan,
            Policy::Cancel,
            dlabel,
            settings.language.clone(),
        )
        .await;
    }
    Ok(())
}

fn show_dialog(
    app: &AppHandle,
    sid: u64,
    plan: Plan,
    s: &Settings,
    dest_label: &str,
) -> Result<(), String> {
    let st = app.state::<AppState>();
    // Kok popup (zaten odakli) diyalog penceresi olarak yeniden kullanilir: odak/on plan degismez.
    // Diger menu popup'lari kapatilir; oturum diyalog ile surer.
    let (root, doomed): (Option<String>, Vec<Popup>) = {
        let mut g = st.session.lock().unwrap();
        match g.as_mut() {
            Some(sess) if sess.id == sid => {
                let mut all: Vec<Popup> = std::mem::take(&mut sess.popups);
                let root = if all.is_empty() {
                    None
                } else {
                    Some(all.remove(0).label)
                };
                (root, all)
            }
            _ => return Ok(()),
        }
    };
    for p in doomed {
        retire(app, &p.label);
    }
    let Some(label) = root else { return Ok(()) };
    let anchor = win::cursor_pos();
    let mon = win::monitor_at(anchor.0, anchor.1);
    let (w, h) = (300.0, if plan.conflicts() > 0 { 176.0 } else { 132.0 });
    let (wp, hp) = (
        (w * mon.scale).round() as i32,
        (h * mon.scale).round() as i32,
    );
    let (x, y, left) = win::place_root(anchor, wp, hp, mon.work);
    let content = Rect::new(x, y, wp, hp);
    let names: Vec<String> = plan.items.iter().take(3).map(|i| i.name.clone()).collect();
    let cfg = merge(
        base_cfg(s),
        json!({
            "kind":"dialog","session":sid,"label":label,"size":[w,h],
            "count": plan.items.len(), "names": names, "conflicts": plan.conflicts(),
            "dest": dest_label,
        }),
    );
    {
        let mut g = st.session.lock().unwrap();
        let Some(sess) = g.as_mut() else {
            drop(g);
            retire(app, &label);
            return Ok(());
        };
        sess.plan = Some(plan);
        sess.popups.push(Popup {
            label: label.clone(),
            depth: 0,
            content,
            left,
            path: None,
            seq: 0,
            scale: mon.scale,
        });
    }
    deliver(app, &label, cfg);
    let m = (MARGIN as f64 * mon.scale).round() as i32;
    schedule_show(app, &label, content.inflate(m), true);
    Ok(())
}

#[tauri::command]
pub async fn dialog_resolve(app: AppHandle, choice: String) -> Result<(), String> {
    let settings = settings_snapshot(&app);
    let st = app.state::<AppState>();
    let plan = {
        let mut g = st.session.lock().unwrap();
        g.as_mut().and_then(|s| s.plan.take())
    };
    let Some(plan) = plan else {
        close_session(&app);
        return Ok(());
    };
    close_session(&app);
    let policy = match choice.as_str() {
        "move" => Policy::Cancel, // cakisma yoksa tum ogeler tasinir; cakisma cikarsa atlanir
        "skip" => Policy::Skip,
        "rename" => Policy::Rename,
        _ => return Ok(()),
    };
    let dlabel = dest_label(&settings, &plan.dest);
    run_move(&app, plan, policy, dlabel, settings.language.clone()).await;
    Ok(())
}

async fn run_move(app: &AppHandle, plan: Plan, policy: Policy, dest_label: String, _lang: String) {
    let res: MoveResult =
        tauri::async_runtime::spawn_blocking(move || move_service::execute(&plan, policy))
            .await
            .unwrap_or_default();
    let first_err = res.items.iter().find_map(|i| match i {
        ItemOutcome::Failed { kind, .. } => Some(format!("{kind:?}").to_lowercase()),
        _ => None,
    });
    let hold = if res.failed > 0 { 5500 } else { 2800 };
    toast(
        app,
        json!({"type":"result","moved":res.moved,"total":res.total,"skipped":res.skipped,"failed":res.failed,"dest":dest_label,"error":first_err}),
        hold,
    );
}

// ------------------------------------------------------------------ bildirim

/// Bildirim genisligi (mantiksal px): metin uzunlugunun kaba tahmini.
fn toast_width(data: &Value) -> f64 {
    let chars = match data.get("type").and_then(|t| t.as_str()) {
        Some("result") if data.get("failed").and_then(|f| f.as_u64()).unwrap_or(0) > 0 => 62,
        Some("result") => {
            data.get("dest")
                .and_then(|d| d.as_str())
                .map(|d| d.chars().count())
                .unwrap_or(0)
                + 22
        }
        _ => 58,
    };
    (chars as f64 * 7.0 + 60.0).clamp(240.0, 560.0)
}

/// Kisa omurlu sonuc bildirimi (calisma alani sag alt). Bos secimde ASLA cagrilmaz.
pub fn toast(app: &AppHandle, data: Value, hold_ms: u64) {
    let app = app.clone();
    std::thread::spawn(move || {
        let st = app.state::<AppState>();
        let gen = st.toast_gen.fetch_add(1, Ordering::SeqCst) + 1;
        if let Some(old) = st.toast_label.lock().unwrap().take() {
            retire(&app, &old);
        }
        let Ok(window) = acquire(&app) else { return };
        let label = window.label().to_string();
        *st.toast_label.lock().unwrap() = Some(label.clone());
        let s = settings_snapshot(&app);
        let cur = win::cursor_pos();
        let mon = win::monitor_at(cur.0, cur.1);
        let (w, h) = (toast_width(&data), 56.0);
        let (wp, hp) = (
            (w * mon.scale).round() as i32,
            (h * mon.scale).round() as i32,
        );
        let edge = (16.0 * mon.scale).round() as i32;
        let content = Rect::new(mon.work.r - wp - edge, mon.work.b - hp - edge, wp, hp);
        let cfg = merge(
            base_cfg(&s),
            json!({"kind":"toast","label":label,"size":[w,h],"data":data}),
        );
        deliver(&app, &label, cfg);
        let m = (MARGIN as f64 * mon.scale).round() as i32;
        schedule_show(&app, &label, content.inflate(m), false);
        std::thread::sleep(Duration::from_millis(hold_ms));
        if st.toast_gen.load(Ordering::SeqCst) == gen {
            *st.toast_label.lock().unwrap() = None;
            retire(&app, &label);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_are_compact_and_bounded() {
        let (w, h) = menu_size(2, false, "normal", 9, 1040.0);
        assert!((212.0..=300.0).contains(&w));
        assert_eq!(h, 12.0 + 2.0 * 34.0 + 2.0);
        let (_, hc) = menu_size(2, false, "compact", 9, 1040.0);
        assert!(hc < h);
    }

    #[test]
    fn many_rows_capped_for_scroll() {
        let (_, h) = menu_size(200, true, "normal", 20, 1040.0);
        assert_eq!(h, 560.0);
        let (_, h2) = menu_size(200, true, "normal", 20, 400.0);
        assert_eq!(h2, 368.0);
    }

    #[test]
    fn move_here_adds_row_and_separator() {
        let (_, a) = menu_size(3, false, "normal", 10, 1040.0);
        let (_, b) = menu_size(3, true, "normal", 10, 1040.0);
        assert_eq!(b - a, 34.0 + 9.0);
    }

    #[test]
    fn width_grows_with_long_names_up_to_cap() {
        let (w1, _) = menu_size(1, false, "normal", 5, 1040.0);
        let (w2, _) = menu_size(1, false, "normal", 60, 1040.0);
        assert_eq!(w1, 212.0);
        assert_eq!(w2, 300.0);
    }
}
