use windows::core::{s, w};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW,
    CreatePen, CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, EndPaint,
    FillRect, InvalidateRect, RoundRect, SelectObject, SetBkMode,
    SetTextColor, DT_CENTER, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE,
    DT_VCENTER, FW_BOLD, FW_NORMAL, FW_SEMIBOLD, HDC, PAINTSTRUCT, PS_SOLID,
    SRCCOPY, TRANSPARENT,
};
use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryA};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GHND};
use windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect,
    GetMessageW, LoadCursorW, PostQuitMessage, RegisterClassW, SendMessageW,
    ShowWindow, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
    IDC_ARROW, IDC_HAND, MSG, SW_MINIMIZE, SW_SHOW,
    WINDOW_EX_STYLE, WM_DESTROY, WM_LBUTTONDOWN,
    WM_MOUSEMOVE, WM_NCCALCSIZE, WM_PAINT, WM_SYSCOMMAND, WNDCLASSW, WS_CLIPCHILDREN,
    WS_MINIMIZEBOX, WS_POPUP, WS_THICKFRAME, WS_VISIBLE,
};

#[repr(C)]
#[allow(non_snake_case)]
struct TRACKMOUSEEVENT {
    cbSize: u32,
    dwFlags: u32,
    hwndTrack: HWND,
    dwHoverTime: u32,
}
const TME_LEAVE: u32 = 0x00000002;
const WM_MOUSELEAVE: u32 = 0x02A3;

extern "system" {
    fn TrackMouseEvent(lpEventTrack: *mut TRACKMOUSEEVENT) -> i32;
}

use crate::state::SharedState;

static GLOBAL_STATE: std::sync::OnceLock<SharedState> = std::sync::OnceLock::new();
static mut HOVERED_BTN: u32 = 0; // 0: None, 1: Close, 2: Min, 3: Copy, 4: Refresh
static mut COPIED_FLAG: bool = false;

const BTN_NONE: u32 = 0;
const BTN_CLOSE: u32 = 1;
const BTN_MIN: u32 = 2;
const BTN_COPY: u32 = 3;
const BTN_REFRESH: u32 = 4;

const WIN_WIDTH: i32 = 440;
const WIN_HEIGHT: i32 = 620;

// Button Rectangles (relative to client area)
const RECT_CLOSE: RECT = RECT { left: 394, top: 0, right: 440, bottom: 42 };
const RECT_MIN: RECT = RECT { left: 348, top: 0, right: 394, bottom: 42 };
const RECT_COPY: RECT = RECT { left: 55, top: 218, right: 385, bottom: 262 };
const RECT_REFRESH: RECT = RECT { left: 55, top: 388, right: 385, bottom: 432 };

pub struct NativeQuickSupportGui;

impl NativeQuickSupportGui {
    pub fn run(state: SharedState) {
        unsafe {
            let _ = GLOBAL_STATE.set(state);

            let instance = GetModuleHandleW(None).unwrap_or_default();
            let class_name = w!("OxiDeskQuickSupportFluentNativeClass");

            let wnd_class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(qs_wnd_proc),
                hInstance: instance.into(),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                lpszClassName: class_name,
                ..Default::default()
            };

            let _ = RegisterClassW(&wnd_class);

            // Frameless Fluent Window: WS_POPUP with WS_THICKFRAME (enables native DWM shadow & aero snap)
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class_name,
                w!("OxiDesk Hızlı Destek"),
                WS_POPUP | WS_THICKFRAME | WS_MINIMIZEBOX | WS_VISIBLE | WS_CLIPCHILDREN,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                WIN_WIDTH,
                WIN_HEIGHT,
                None,
                None,
                instance,
                None,
            );

            if let Ok(hwnd) = hwnd {
                apply_fluent_window_effects(hwnd);
                let _ = ShowWindow(hwnd, SW_SHOW);

                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                    let _ = windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
    }
}

unsafe fn apply_fluent_window_effects(hwnd: HWND) {
    if let Ok(dwm) = LoadLibraryA(s!("dwmapi.dll")) {
        type FnDwmSetWindowAttribute = unsafe extern "system" fn(HWND, u32, *const std::ffi::c_void, u32) -> i32;
        if let Some(set_attr) = GetProcAddress(dwm, s!("DwmSetWindowAttribute")) {
            let set_attr: FnDwmSetWindowAttribute = std::mem::transmute(set_attr);
            // DWMWA_USE_IMMERSIVE_DARK_MODE = 20
            let dark_mode: i32 = 1;
            set_attr(hwnd, 20, &dark_mode as *const _ as _, 4);
            // DWMWA_WINDOW_CORNER_PREFERENCE = 33 -> DWMWCP_ROUND = 2
            let corner_round: i32 = 2;
            set_attr(hwnd, 33, &corner_round as *const _ as _, 4);
        }
    }
}

unsafe fn copy_to_clipboard(text: &str) {
    if OpenClipboard(None).is_ok() {
        let _ = EmptyClipboard();
        let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let byte_len = utf16.len() * 2;

        if let Ok(handle) = GlobalAlloc(GHND, byte_len) {
            let ptr = GlobalLock(handle);
            if !ptr.is_null() {
                std::ptr::copy_nonoverlapping(utf16.as_ptr() as *const u8, ptr as *mut u8, byte_len);
                let _ = GlobalUnlock(handle);
                // CF_UNICODETEXT = 13
                let _ = SetClipboardData(13, windows::Win32::Foundation::HANDLE(handle.0));
            }
        }
        let _ = CloseClipboard();
    }
}

unsafe fn is_inside(x: i32, y: i32, r: &RECT) -> bool {
    x >= r.left && x <= r.right && y >= r.top && y <= r.bottom
}

unsafe extern "system" fn qs_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        // Strip legacy window frame and titlebar completely (Frameless WPF/Fluent UI)
        WM_NCCALCSIZE => {
            if wparam.0 != 0 {
                return LRESULT(0);
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_MOUSEMOVE => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            let prev_hover = HOVERED_BTN;
            let mut new_hover = BTN_NONE;

            if is_inside(x, y, &RECT_CLOSE) {
                new_hover = BTN_CLOSE;
            } else if is_inside(x, y, &RECT_MIN) {
                new_hover = BTN_MIN;
            } else if is_inside(x, y, &RECT_COPY) {
                new_hover = BTN_COPY;
            } else if is_inside(x, y, &RECT_REFRESH) {
                new_hover = BTN_REFRESH;
            }

            if new_hover != prev_hover {
                HOVERED_BTN = new_hover;
                let _ = InvalidateRect(hwnd, None, false);
            }

            // Set Hand cursor when hovering interactive buttons
            if new_hover != BTN_NONE {
                windows::Win32::UI::WindowsAndMessaging::SetCursor(LoadCursorW(None, IDC_HAND).unwrap_or_default());
            }

            // Track mouse leave so hover resets
            let mut tme = TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: hwnd,
                dwHoverTime: 0,
            };
            let _ = TrackMouseEvent(&mut tme);

            LRESULT(0)
        }

        WM_MOUSELEAVE => {
            if HOVERED_BTN != BTN_NONE {
                HOVERED_BTN = BTN_NONE;
                let _ = InvalidateRect(hwnd, None, false);
            }
            LRESULT(0)
        }

        WM_LBUTTONDOWN => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            if is_inside(x, y, &RECT_CLOSE) {
                PostQuitMessage(0);
                std::process::exit(0);
            } else if is_inside(x, y, &RECT_MIN) {
                let _ = ShowWindow(hwnd, SW_MINIMIZE);
            } else if is_inside(x, y, &RECT_COPY) {
                if let Some(state) = GLOBAL_STATE.get() {
                    let id_str = state.info.lock().unwrap().device_id.clone();
                    copy_to_clipboard(&id_str);
                    COPIED_FLAG = true;
                    let _ = InvalidateRect(hwnd, None, false);
                }
            } else if is_inside(x, y, &RECT_REFRESH) {
                if let Some(state) = GLOBAL_STATE.get() {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let new_pin: u16 = rng.gen_range(1000..9999);
                    state.info.lock().unwrap().pin = new_pin.to_string();
                    COPIED_FLAG = false;
                    let _ = InvalidateRect(hwnd, None, false);
                }
            } else if y < 44 && x < 348 {
                // Drag window natively by clicking anywhere on the custom Fluent title bar
                let _ = ReleaseCapture();
                let _ = SendMessageW(
                    hwnd,
                    WM_SYSCOMMAND,
                    WPARAM(0xF012), // SC_DRAGMOVE
                    LPARAM(0),
                );
            }

            LRESULT(0)
        }

        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut client_rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut client_rect);

            let w = client_rect.right - client_rect.left;
            let h = client_rect.bottom - client_rect.top;

            // Double buffering for ultra-smooth 60 FPS zero-flicker Fluent rendering
            let mem_dc = CreateCompatibleDC(hdc);
            let mem_bmp = CreateCompatibleBitmap(hdc, w, h);
            let old_bmp = SelectObject(mem_dc, mem_bmp);

            paint_fluent_qs_gui(mem_dc, w, h);

            let _ = BitBlt(hdc, 0, 0, w, h, mem_dc, 0, 0, SRCCOPY);

            SelectObject(mem_dc, old_bmp);
            let _ = DeleteObject(mem_bmp);
            let _ = DeleteDC(mem_dc);

            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }

        WM_DESTROY => {
            PostQuitMessage(0);
            std::process::exit(0);
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn paint_fluent_qs_gui(hdc: HDC, w: i32, h: i32) {
    let full_rect = RECT { left: 0, top: 0, right: w, bottom: h };

    // 1. Fluent Slate Background (#0B0F19)
    let bg_brush = CreateSolidBrush(COLORREF(0x00190F0B)); // BGR: 0x19, 0x0F, 0x0B
    FillRect(hdc, &full_rect, bg_brush);
    let _ = DeleteObject(bg_brush);

    // 2. Custom Fluent Title Bar (#0F172A)
    let tb_brush = CreateSolidBrush(COLORREF(0x002A170F));
    let tb_rect = RECT { left: 0, top: 0, right: w, bottom: 42 };
    FillRect(hdc, &tb_rect, tb_brush);
    let _ = DeleteObject(tb_brush);

    // Titlebar Border Line (#1E293B)
    let tb_pen = CreatePen(PS_SOLID, 1, COLORREF(0x003B291E));
    let old_pen = SelectObject(hdc, tb_pen);
    let _ = windows::Win32::Graphics::Gdi::MoveToEx(hdc, 0, 42, None);
    let _ = windows::Win32::Graphics::Gdi::LineTo(hdc, w, 42);
    let _ = DeleteObject(tb_pen);

    let _ = SetBkMode(hdc, TRANSPARENT);

    // Fonts (Modern Segoe UI)
    let f_title = CreateFontW(15, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"));
    let f_ver = CreateFontW(12, 0, 0, 0, FW_NORMAL.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"));
    let f_caption = CreateFontW(11, 0, 0, 0, FW_SEMIBOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"));
    let f_label = CreateFontW(10, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"));
    let f_id = CreateFontW(32, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"));
    let f_pin = CreateFontW(28, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"));
    let f_btn = CreateFontW(13, 0, 0, 0, FW_SEMIBOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"));
    let f_footer = CreateFontW(11, 0, 0, 0, FW_NORMAL.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"));

    // Title text: "🛡️ OxiDesk Hızlı Destek"
    SelectObject(hdc, f_title);
    SetTextColor(hdc, COLORREF(0x00F8BA38)); // Vibrant Sky Blue (#38BAF8)
    let mut title_r = RECT { left: 16, top: 0, right: 240, bottom: 42 };
    let mut text_title: Vec<u16> = "OxiDesk Hızlı Destek".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut text_title, &mut title_r, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    // Version Tag: "v5.0.0"
    SelectObject(hdc, f_ver);
    SetTextColor(hdc, COLORREF(0x0094A3B8)); // Slate 400
    let mut ver_r = RECT { left: 185, top: 0, right: 260, bottom: 42 };
    let mut text_ver: Vec<u16> = "v5.5.0".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut text_ver, &mut ver_r, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    // Title Bar Buttons (Minimize & Close)
    // 1. Minimize Button
    let min_hover = HOVERED_BTN == BTN_MIN;
    if min_hover {
        let b = CreateSolidBrush(COLORREF(0x0033291E)); // Light hover
        FillRect(hdc, &RECT_MIN, b);
        let _ = DeleteObject(b);
    }
    SelectObject(hdc, f_title);
    SetTextColor(hdc, COLORREF(0x00CBD5E1));
    let mut min_r = RECT_MIN;
    let mut t_min: Vec<u16> = "—".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_min, &mut min_r, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    // 2. Close Button
    let close_hover = HOVERED_BTN == BTN_CLOSE;
    if close_hover {
        let b = CreateSolidBrush(COLORREF(0x002311E8)); // Fluent Red (#E81123)
        FillRect(hdc, &RECT_CLOSE, b);
        let _ = DeleteObject(b);
        SetTextColor(hdc, COLORREF(0x00FFFFFF));
    } else {
        SetTextColor(hdc, COLORREF(0x00CBD5E1));
    }
    let mut close_r = RECT_CLOSE;
    let mut t_close: Vec<u16> = "✕".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_close, &mut close_r, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    // =========================================================================
    // Card 0: Status Ribbon (SOS Active Beacon)
    // =========================================================================
    draw_rounded_card(hdc, 20, 54, w - 20, 102, 0x001F1811, 0x0037291F, 12);
    SelectObject(hdc, f_caption);
    SetTextColor(hdc, COLORREF(0x0081B910)); // Emerald 500 (#10B981)
    let mut stat_r = RECT { left: 36, top: 62, right: w - 36, bottom: 82 };
    let mut t_stat: Vec<u16> = "●  DESTEK AĞINA BAĞLI & HAZIR (SOS)".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_stat, &mut stat_r, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    SelectObject(hdc, f_footer);
    SetTextColor(hdc, COLORREF(0x0094A3B8));
    let mut sub_stat_r = RECT { left: 36, top: 80, right: w - 36, bottom: 98 };
    let mut t_sub_stat: Vec<u16> = "Ağdaki Bilgi İşlem Masası bu cihaza doğrudan bağlanabilir.".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_sub_stat, &mut sub_stat_r, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    // =========================================================================
    // Card 1: OxiDesk ID Display
    // =========================================================================
    draw_rounded_card(hdc, 20, 114, w - 20, 276, 0x002E1E13, 0x004D3423, 16);

    SelectObject(hdc, f_label);
    SetTextColor(hdc, COLORREF(0x008B9AA6)); // Slate 400
    let mut c1_lbl = RECT { left: 40, top: 128, right: w - 40, bottom: 148 };
    let mut t_c1_lbl: Vec<u16> = "BU BİLGİSAYARIN OXİDESK KİMLİĞİ".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_c1_lbl, &mut c1_lbl, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    let (device_id_str, pin_str, hostname, port) = if let Some(state) = GLOBAL_STATE.get() {
        let info = state.info.lock().unwrap();
        (info.device_id.clone(), info.pin.clone(), info.hostname.clone(), info.port)
    } else {
        ("123456789".to_string(), "1234".to_string(), "PC".to_string(), 8080)
    };

    let formatted_id = if device_id_str.len() == 9 {
        format!("{} {} {}", &device_id_str[0..3], &device_id_str[3..6], &device_id_str[6..9])
    } else {
        device_id_str
    };

    SelectObject(hdc, f_id);
    SetTextColor(hdc, COLORREF(0x00F8BA38)); // Sky 400 (#38BAF8) Neon
    let mut id_r = RECT { left: 40, top: 152, right: w - 40, bottom: 204 };
    let mut t_id: Vec<u16> = formatted_id.encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_id, &mut id_r, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    // Button 1: Copy ID (Primary Vibrant Blue)
    let copy_hover = HOVERED_BTN == BTN_COPY;
    let copy_bg = if copy_hover { 0x00E9A50E } else { 0x00C78402 }; // Sky 500 / 600
    draw_rounded_card(hdc, RECT_COPY.left, RECT_COPY.top, RECT_COPY.right, RECT_COPY.bottom, copy_bg, copy_bg, 10);

    SelectObject(hdc, f_btn);
    SetTextColor(hdc, COLORREF(0x00FFFFFF));
    let mut copy_btn_r = RECT_COPY;
    let btn_label = if COPIED_FLAG { "✓  Kimlik Kopyalandı!" } else { "📋  Kimliği Kopyala" };
    let mut t_copy_btn: Vec<u16> = btn_label.encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_copy_btn, &mut copy_btn_r, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    // =========================================================================
    // Card 2: PIN Code Display
    // =========================================================================
    draw_rounded_card(hdc, 20, 288, w - 20, 446, 0x00261D15, 0x003D2E22, 16);

    SelectObject(hdc, f_label);
    SetTextColor(hdc, COLORREF(0x008B9AA6));
    let mut c2_lbl = RECT { left: 40, top: 302, right: w - 40, bottom: 322 };
    let mut t_c2_lbl: Vec<u16> = "GEÇİCİ BAĞLANTI PIN KODU".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_c2_lbl, &mut c2_lbl, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    SelectObject(hdc, f_pin);
    SetTextColor(hdc, COLORREF(0x0024BFFA)); // Amber/Yellow #FABF24
    let mut pin_r = RECT { left: 40, top: 326, right: w - 40, bottom: 374 };
    let mut t_pin: Vec<u16> = pin_str.encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_pin, &mut pin_r, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    // Button 2: Refresh PIN (Secondary Dark Slate)
    let ref_hover = HOVERED_BTN == BTN_REFRESH;
    let ref_bg = if ref_hover { 0x00473527 } else { 0x003B291E }; // Slate 700 / 800
    let ref_border = if ref_hover { 0x00644C38 } else { 0x004D3728 };
    draw_rounded_card(hdc, RECT_REFRESH.left, RECT_REFRESH.top, RECT_REFRESH.right, RECT_REFRESH.bottom, ref_bg, ref_border, 10);

    SelectObject(hdc, f_btn);
    SetTextColor(hdc, COLORREF(0x00F1F5F9));
    let mut ref_btn_r = RECT_REFRESH;
    let mut t_ref_btn: Vec<u16> = "🔄  Yeni PIN Üret".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_ref_btn, &mut ref_btn_r, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    // =========================================================================
    // Footer Information & Security Badge
    // =========================================================================
    SelectObject(hdc, f_caption);
    SetTextColor(hdc, COLORREF(0x00F8BA38));
    let mut sec_r = RECT { left: 20, top: 462, right: w - 20, bottom: 482 };
    let mut t_sec: Vec<u16> = "🔒  AES-256-GCM Uçtan Uca Şifreli Oturum".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_sec, &mut sec_r, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    SelectObject(hdc, f_footer);
    SetTextColor(hdc, COLORREF(0x0064748B));
    let dev_info = format!("Cihaz: {} | Yerel Port: {}", hostname, port);
    let mut dev_r = RECT { left: 20, top: 488, right: w - 20, bottom: 508 };
    let mut t_dev: Vec<u16> = dev_info.encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_dev, &mut dev_r, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    let mut foot_r = RECT { left: 20, top: 512, right: w - 20, bottom: 536 };
    let mut t_foot: Vec<u16> = "Destek oturumunu bitirmek için pencereyi kapatmanız yeterlidir.".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_foot, &mut foot_r, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    // Outer Modern 1px Border (#1E293B)
    let border_pen = CreatePen(PS_SOLID, 1, COLORREF(0x003B291E));
    SelectObject(hdc, border_pen);
    let null_brush = windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::GET_STOCK_OBJECT_FLAGS(5)); // NULL_BRUSH
    SelectObject(hdc, null_brush);
    let _ = RoundRect(hdc, 0, 0, w, h, 16, 16);
    let _ = DeleteObject(border_pen);

    // GDI Cleanup
    SelectObject(hdc, old_pen);
    let _ = DeleteObject(f_title);
    let _ = DeleteObject(f_ver);
    let _ = DeleteObject(f_caption);
    let _ = DeleteObject(f_label);
    let _ = DeleteObject(f_id);
    let _ = DeleteObject(f_pin);
    let _ = DeleteObject(f_btn);
    let _ = DeleteObject(f_footer);
}

unsafe fn draw_rounded_card(hdc: HDC, left: i32, top: i32, right: i32, bottom: i32, bg_bgr: u32, border_bgr: u32, radius: i32) {
    let brush = CreateSolidBrush(COLORREF(bg_bgr));
    let pen = CreatePen(PS_SOLID, 1, COLORREF(border_bgr));
    let old_brush = SelectObject(hdc, brush);
    let old_pen = SelectObject(hdc, pen);

    let _ = RoundRect(hdc, left, top, right, bottom, radius, radius);

    SelectObject(hdc, old_brush);
    SelectObject(hdc, old_pen);
    let _ = DeleteObject(brush);
    let _ = DeleteObject(pen);
}
