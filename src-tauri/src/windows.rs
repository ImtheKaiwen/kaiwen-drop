//! Win32 yardimcilari, pencere olusturma ve (saf, test edilebilir) popup yerlesim hesabi.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Seffaf kenar boslugu (golge icin), mantiksal px. Icerik dikdortgeni = pencere - MARGIN.
pub const MARGIN: i32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub l: i32,
    pub t: i32,
    pub r: i32,
    pub b: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            l: x,
            t: y,
            r: x + w,
            b: y + h,
        }
    }
    pub fn w(&self) -> i32 {
        self.r - self.l
    }
    pub fn h(&self) -> i32 {
        self.b - self.t
    }
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.l && x < self.r && y >= self.t && y < self.b
    }
    pub fn inflate(&self, d: i32) -> Rect {
        Rect {
            l: self.l - d,
            t: self.t - d,
            r: self.r + d,
            b: self.b + d,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MonitorInfo {
    pub work: Rect,
    pub scale: f64,
}

// ---------------------------------------------------------------- saf yerlesim

/// Kok menu: imlecin yaninda; sigmazsa sola/yukari doner. Donus: (x, y, sola_acildi_mi). Fiziksel px, icerik koordinati.
pub fn place_root(cursor: (i32, i32), w: i32, h: i32, work: Rect) -> (i32, i32, bool) {
    let mut left = false;
    let mut x = cursor.0 + 2;
    if x + w > work.r {
        x = cursor.0 - w - 2;
        left = true;
    }
    x = x.clamp(work.l, (work.r - w).max(work.l));
    let mut y = cursor.1 + 2;
    if y + h > work.b {
        y = cursor.1 - h - 2;
        if y < work.t {
            y = work.b - h;
        }
    }
    y = y.clamp(work.t, (work.b - h).max(work.t));
    (x, y, left)
}

#[allow(clippy::too_many_arguments)]
/// Alt menu: ana menunun yaninda, ilk satir hover edilen satirla hizali. Sigmazsa ters yone acilir.
pub fn place_child(
    parent: Rect,
    row_top: i32,
    w: i32,
    h: i32,
    work: Rect,
    prefer_left: bool,
    gap: i32,
    pad: i32,
) -> (i32, i32, bool) {
    let right_x = parent.r + gap;
    let left_x = parent.l - gap - w;
    let fits_right = right_x + w <= work.r;
    let fits_left = left_x >= work.l;
    let go_left = if prefer_left {
        fits_left || !fits_right && (parent.l - work.l) > (work.r - parent.r)
    } else {
        !fits_right && (fits_left || (parent.l - work.l) > (work.r - parent.r))
    };
    let x = if go_left { left_x } else { right_x };
    let x = x.clamp(work.l, (work.r - w).max(work.l));
    let mut y = row_top - pad;
    if y + h > work.b {
        y = work.b - h;
    }
    y = y.max(work.t);
    (x, y, go_left)
}

// ---------------------------------------------------------------- Win32

#[cfg(windows)]
mod sys {
    use super::*;
    use windows::Win32::Foundation::{HWND, POINT, RECT};
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
    };
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows::Win32::System::Threading::GetCurrentProcessId;
    use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
    use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetCursorPos, GetForegroundWindow, GetWindowLongPtrW, GetWindowThreadProcessId,
        SetForegroundWindow, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST,
        SWP_NOACTIVATE, SWP_SHOWWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    };

    pub fn cursor_pos() -> (i32, i32) {
        let mut p = POINT::default();
        unsafe {
            let _ = GetCursorPos(&mut p);
        }
        (p.x, p.y)
    }

    pub fn monitor_at(x: i32, y: i32) -> MonitorInfo {
        unsafe {
            let hm = MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST);
            let mut mi = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            let work = if GetMonitorInfoW(hm, &mut mi).as_bool() {
                let r: RECT = mi.rcWork;
                Rect {
                    l: r.left,
                    t: r.top,
                    r: r.right,
                    b: r.bottom,
                }
            } else {
                Rect::new(0, 0, 1920, 1040)
            };
            let (mut dx, mut dy) = (96u32, 96u32);
            let _ = GetDpiForMonitor(hm, MDT_EFFECTIVE_DPI, &mut dx, &mut dy);
            MonitorInfo {
                work,
                scale: dx as f64 / 96.0,
            }
        }
    }

    pub fn is_down(vk: u16) -> bool {
        (unsafe { GetAsyncKeyState(vk as i32) } as u16) & 0x8000 != 0
    }

    /// On plandaki pencere bu surece mi ait?
    pub fn foreground_is_ours() -> bool {
        unsafe {
            let fg = GetForegroundWindow();
            if fg.0.is_null() {
                return false;
            }
            let mut pid = 0u32;
            GetWindowThreadProcessId(fg, Some(&mut pid));
            pid == GetCurrentProcessId()
        }
    }

    /// Popup stili: arac penceresi (gorev cubugunda yok); istenirse aktive OLMAZ.
    pub fn apply_popup_style(hwnd: HWND, activatable: bool) {
        unsafe {
            let mut ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
            ex |= WS_EX_TOOLWINDOW.0;
            if activatable {
                ex &= !WS_EX_NOACTIVATE.0;
            } else {
                ex |= WS_EX_NOACTIVATE.0;
            }
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex as isize);
        }
    }

    /// Fiziksel px, tek atomik cagri. Iki kez uygulanir: farkli DPI'li monitore tasinirken WM_DPICHANGED duzeltmesine karsi.
    pub fn place_window(hwnd: HWND, r: Rect, show: bool) {
        let flags = if show {
            SWP_NOACTIVATE | SWP_SHOWWINDOW
        } else {
            SWP_NOACTIVATE
        };
        unsafe {
            for _ in 0..2 {
                let _ = SetWindowPos(hwnd, Some(HWND_TOPMOST), r.l, r.t, r.w(), r.h(), flags);
            }
        }
    }

    pub fn activate(hwnd: HWND) {
        unsafe {
            let _ = SetForegroundWindow(hwnd);
        }
    }

    /// Win11: yuvarlak kose + ince kenarlik (Settings gibi cercevesiz pencereler icin).
    pub fn round_corners(hwnd: HWND, border_rgb: u32) {
        unsafe {
            let pref = DWMWCP_ROUND;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &pref as *const _ as *const _,
                std::mem::size_of_val(&pref) as u32,
            );
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_BORDER_COLOR,
                &border_rgb as *const _ as *const _,
                4,
            );
        }
    }
}

#[cfg(windows)]
pub use sys::*;

#[cfg(not(windows))]
mod stub {
    use super::*;
    pub fn cursor_pos() -> (i32, i32) {
        (0, 0)
    }
    pub fn monitor_at(_x: i32, _y: i32) -> MonitorInfo {
        MonitorInfo {
            work: Rect::new(0, 0, 1920, 1040),
            scale: 1.0,
        }
    }
    pub fn is_down(_vk: u16) -> bool {
        false
    }
    pub fn foreground_is_ours() -> bool {
        true
    }
}
#[cfg(not(windows))]
pub use stub::*;

pub const VK_LBUTTON: u16 = 0x01;
pub const VK_RBUTTON: u16 = 0x02;
pub const VK_MBUTTON: u16 = 0x04;
pub const VK_ESCAPE: u16 = 0x1B;

// ---------------------------------------------------------------- pencere uretimi

pub fn build_popup(app: &AppHandle, label: &str) -> tauri::Result<WebviewWindow> {
    let w = WebviewWindowBuilder::new(app, label, WebviewUrl::App("popup.html".into()))
        .title("Kaiwen Drop")
        .inner_size(220.0, 120.0)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .resizable(false)
        .visible(false)
        .focused(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .maximizable(false)
        .minimizable(false)
        .accept_first_mouse(true)
        .disable_drag_drop_handler()
        .build()?;
    #[cfg(windows)]
    if let Ok(h) = w.hwnd() {
        apply_popup_style(h, false);
    }
    Ok(w)
}

/// Fiziksel `Rect` ile (kenar boslugu dahil pencere dikdortgeni) yerlestirir ve gosterir.
pub fn show_at(w: &WebviewWindow, outer: Rect, activate_it: bool) {
    #[cfg(windows)]
    if let Ok(h) = w.hwnd() {
        apply_popup_style(h, activate_it);
        place_window(h, outer, true);
        if activate_it {
            activate(h);
            let _ = w.set_focus();
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (w, outer, activate_it);
    }
}

pub fn open_settings(app: &AppHandle, dark: bool) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }
    let bg = if dark {
        tauri::window::Color(18, 18, 18, 255)
    } else {
        tauri::window::Color(250, 250, 249, 255)
    };
    let app2 = app.clone();
    let built = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
        .title("Kaiwen Drop")
        .inner_size(440.0, 560.0)
        .min_inner_size(400.0, 420.0)
        .decorations(false)
        .shadow(true)
        .resizable(false)
        .maximizable(false)
        .center()
        .visible(false)
        .background_color(bg)
        .build();
    match built {
        Ok(w) => {
            #[cfg(windows)]
            if let Ok(h) = w.hwnd() {
                round_corners(h, if dark { 0x0033_3333 } else { 0x00E7_E9E9 });
            }
            // Kisayol kaydi sirasinda pencere kapanirsa global kisayol geri yuklensin.
            w.on_window_event(move |ev| {
                if matches!(ev, tauri::WindowEvent::Destroyed) {
                    crate::pause_shortcut(app2.clone(), false);
                }
            });
            // gosterme isini sayfa `window_ready` ile yapar (beyaz flash yok)
        }
        Err(e) => eprintln!("settings penceresi acilamadi: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORK: Rect = Rect {
        l: 0,
        t: 0,
        r: 1920,
        b: 1040,
    };

    #[test]
    fn root_fits_normally() {
        let (x, y, left) = place_root((500, 400), 232, 200, WORK);
        assert_eq!((x, y, left), (502, 402, false));
    }

    #[test]
    fn root_flips_left_at_right_edge() {
        let (x, _, left) = place_root((1900, 400), 232, 200, WORK);
        assert!(left);
        assert!(x + 232 <= 1920);
        assert_eq!(x, 1900 - 232 - 2);
    }

    #[test]
    fn root_flips_up_at_bottom() {
        let (_, y, _) = place_root((500, 1030), 232, 200, WORK);
        assert_eq!(y, 1030 - 200 - 2);
        assert!(y >= 0);
    }

    #[test]
    fn root_taller_than_screen_is_clamped() {
        let (_, y, _) = place_root((500, 500), 232, 2000, WORK);
        assert_eq!(y, 0);
    }

    #[test]
    fn child_opens_right_aligned_to_row() {
        let parent = Rect::new(500, 400, 232, 100);
        let (x, y, left) = place_child(parent, 440, 232, 150, WORK, false, 2, 6);
        assert_eq!((x, y, left), (734, 434, false));
    }

    #[test]
    fn child_flips_left_when_no_room() {
        let parent = Rect::new(1600, 400, 232, 100);
        let (x, _, left) = place_child(parent, 440, 232, 150, WORK, false, 2, 6);
        assert!(left);
        assert_eq!(x, 1600 - 2 - 232);
    }

    #[test]
    fn child_prefers_left_when_parent_went_left() {
        let parent = Rect::new(900, 400, 232, 100);
        let (_, _, left) = place_child(parent, 440, 232, 150, WORK, true, 2, 6);
        assert!(left);
    }

    #[test]
    fn child_shifts_up_at_bottom_and_stays_in_work_area() {
        let parent = Rect::new(500, 900, 232, 100);
        let (_, y, _) = place_child(parent, 1000, 232, 300, WORK, false, 2, 6);
        assert_eq!(y, 1040 - 300);
        let (_, y2, _) = place_child(parent, 10, 232, 300, WORK, false, 2, 6);
        assert_eq!(y2, 4);
    }

    #[test]
    fn multi_monitor_negative_origin() {
        let work = Rect {
            l: -1920,
            t: 0,
            r: 0,
            b: 1040,
        };
        let (x, _, left) = place_root((-10, 100), 232, 200, work);
        assert!(left);
        assert!(x >= -1920 && x + 232 <= 0);
    }
}
