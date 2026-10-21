use std::sync::{Arc, Mutex};
use windows::core::{s, w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW,
    CreatePen, CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, EndPaint,
    FillRect, InvalidateRect, RoundRect, SelectObject, SetBkColor, SetBkMode,
    SetDIBitsToDevice, SetTextColor, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    DIB_RGB_COLORS, DRAW_TEXT_FORMAT, DT_CENTER, DT_LEFT, DT_NOPREFIX,
    DT_SINGLELINE, DT_VCENTER, FW_BOLD, FW_NORMAL, FW_SEMIBOLD, HDC, PAINTSTRUCT,
    PS_SOLID, SRCCOPY, TRANSPARENT,
};
use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryA};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GHND};
use windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
    GetWindowTextLengthW, GetWindowTextW, LoadCursorW, PostQuitMessage, RegisterClassW,
    SendMessageW, SetWindowTextW, ShowWindow, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
    ES_AUTOHSCROLL, HMENU, IDC_ARROW, IDC_HAND, MSG, SW_MINIMIZE, SW_SHOW,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_COMMAND, WM_CREATE, WM_CTLCOLOREDIT,
    WM_CTLCOLORSTATIC, WM_DESTROY, WM_DRAWITEM, WM_LBUTTONDOWN, WM_MOUSEMOVE,
    WM_NCCALCSIZE, WM_PAINT, WM_SYSCOMMAND, WNDCLASSW, WS_BORDER, WS_CHILD,
    WS_CLIPCHILDREN, WS_MINIMIZEBOX, WS_OVERLAPPEDWINDOW, WS_POPUP, WS_TABSTOP,
    WS_THICKFRAME, WS_VISIBLE,
};

use crate::state::SharedState;

static GLOBAL_ADMIN_STATE: std::sync::OnceLock<SharedState> = std::sync::OnceLock::new();
static mut TARGET_EDIT_HWND: HWND = HWND(0 as _);
static mut PIN_EDIT_HWND: HWND = HWND(0 as _);
static mut MAIN_ADMIN_HWND: HWND = HWND(0 as _);
static mut HOVERED_ADMIN_BTN: u32 = 0; // 0: None, 1: Close, 2: Min

const ID_BTN_COPY_ID: usize = 201;
const ID_BTN_REFRESH_PIN: usize = 202;
const ID_BTN_CONNECT: usize = 203;
const ID_BTN_SCAN: usize = 204;
const ID_BTN_FILL_DISCOVERED: usize = 205;

const WIN_WIDTH: i32 = 880;
const WIN_HEIGHT: i32 = 640;

const RECT_ADMIN_CLOSE: RECT = RECT { left: 834, top: 0, right: 880, bottom: 42 };
const RECT_ADMIN_MIN: RECT = RECT { left: 788, top: 0, right: 834, bottom: 42 };

#[repr(C)]
#[allow(non_snake_case)]
struct DRAWITEMSTRUCT {
    ctlType: u32,
    ctlID: u32,
    itemID: u32,
    itemAction: u32,
    itemState: u32,
    hwndItem: HWND,
    hDC: HDC,
    rcItem: RECT,
    itemData: usize,
}

pub struct NativeAdminGui;

impl NativeAdminGui {
    pub fn run(state: SharedState) {
        unsafe {
            let _ = GLOBAL_ADMIN_STATE.set(state);

            let instance = GetModuleHandleW(None).unwrap_or_default();
            let class_name = w!("OxiDeskAdminFluentNativeClass");

            let wnd_class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(admin_wnd_proc),
                hInstance: instance.into(),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                lpszClassName: class_name,
                ..Default::default()
            };

            let _ = RegisterClassW(&wnd_class);

            // Frameless Fluent Window
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class_name,
                w!("OxiDesk IT Yönetim Masası"),
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
                MAIN_ADMIN_HWND = hwnd;
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
            let dark_mode: i32 = 1;
            set_attr(hwnd, 20, &dark_mode as *const _ as _, 4);
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
                let _ = SetClipboardData(13, windows::Win32::Foundation::HANDLE(handle.0));
            }
        }
        let _ = CloseClipboard();
    }
}

unsafe fn draw_text_helper(hdc: HDC, text: &str, rect: &mut RECT, format: DRAW_TEXT_FORMAT) {
    let mut utf16: Vec<u16> = text.encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut utf16, rect, format);
}

unsafe fn get_edit_text(hwnd: HWND) -> String {
    let len = GetWindowTextLengthW(hwnd);
    if len <= 0 {
        return String::new();
    }
    let mut buf = vec![0u16; (len + 1) as usize];
    GetWindowTextW(hwnd, &mut buf);
    String::from_utf16_lossy(&buf[..len as usize])
}

unsafe fn is_inside(x: i32, y: i32, r: &RECT) -> bool {
    x >= r.left && x <= r.right && y >= r.top && y <= r.bottom
}

unsafe extern "system" fn admin_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_NCCALCSIZE => {
            if wparam.0 != 0 {
                return LRESULT(0); // Frameless WPF Fluent UI
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_CREATE => {
            let instance = GetModuleHandleW(None).unwrap_or_default();
            // BS_OWNERDRAW = 0x0000000B
            let btn_owner_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(0x0000000B);
            let edit_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_BORDER | WINDOW_STYLE(ES_AUTOHSCROLL as u32);

            // Left Card Buttons: [Kimliği Kopyala] and [🔄 PIN Yenile]
            let _ = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("📋 Kimliği Kopyala"),
                btn_owner_style,
                45, 260, 160, 38,
                hwnd, HMENU(ID_BTN_COPY_ID as _), instance, None,
            );

            let _ = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("🔄 Yeni PIN"),
                btn_owner_style,
                220, 260, 140, 38,
                hwnd, HMENU(ID_BTN_REFRESH_PIN as _), instance, None,
            );

            // Right Card Inputs: Target OxiDesk ID and PIN (Dark Slate Fluent Inputs)
            let target_edit = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("EDIT"),
                w!(""),
                edit_style,
                465, 155, 345, 34,
                hwnd, HMENU(301 as _), instance, None,
            );
            if let Ok(e) = target_edit {
                TARGET_EDIT_HWND = e;
            }

            let pin_edit = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("EDIT"),
                w!(""),
                edit_style,
                465, 218, 175, 34,
                hwnd, HMENU(302 as _), instance, None,
            );
            if let Ok(e) = pin_edit {
                PIN_EDIT_HWND = e;
            }

            // [ 🚀 Uzak Oturumu Başlat ] Button
            let _ = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("🚀 Uzak Oturumu Başlat"),
                btn_owner_style,
                465, 262, 345, 42,
                hwnd, HMENU(ID_BTN_CONNECT as _), instance, None,
            );

            // Bottom Section Buttons: [ 🔍 Ağı Tara ] and [ Seçileni Doldur ]
            let _ = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("🔍 Ağ Cihazlarını Tara (LAN Scan)"),
                btn_owner_style,
                45, 545, 260, 38,
                hwnd, HMENU(ID_BTN_SCAN as _), instance, None,
            );

            let _ = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("Seçili Cihazı Doldur"),
                btn_owner_style,
                325, 545, 180, 38,
                hwnd, HMENU(ID_BTN_FILL_DISCOVERED as _), instance, None,
            );

            LRESULT(0)
        }

        // Dark Fluent styling for EDIT controls (Zero white boxes!)
        WM_CTLCOLOREDIT | WM_CTLCOLORSTATIC => {
            let hdc = HDC(wparam.0 as _);
            SetTextColor(hdc, COLORREF(0x00FFFFFF)); // White text
            SetBkColor(hdc, COLORREF(0x002B1D14));   // Dark Slate #141D2B
            let brush = CreateSolidBrush(COLORREF(0x002B1D14));
            LRESULT(brush.0 as _)
        }

        // Custom Owner-Draw for Modern WPF / Fluent UI Buttons
        WM_DRAWITEM => {
            let dis = &*(lparam.0 as *const DRAWITEMSTRUCT);
            let hdc = dis.hDC;
            let r = dis.rcItem;
            let is_pressed = (dis.itemState & 0x0001) != 0; // ODS_SELECTED

            let (bg_bgr, text_color, label) = match dis.ctlID as usize {
                ID_BTN_CONNECT => {
                    let c = if is_pressed { 0x00699605 } else { 0x0081B910 }; // Emerald 500 / 600
                    (c, 0x00FFFFFF, "🚀  Uzak Oturumu Başlat")
                }
                ID_BTN_COPY_ID => {
                    let c = if is_pressed { 0x00A16903 } else { 0x00C78402 }; // Sky 500 / 600
                    (c, 0x00FFFFFF, "📋  Kimliği Kopyala")
                }
                ID_BTN_REFRESH_PIN => {
                    let c = if is_pressed { 0x00261E17 } else { 0x003B291E }; // Slate 700 / 800
                    (c, 0x00CBD5E1, "🔄  Yeni PIN")
                }
                ID_BTN_SCAN => {
                    let c = if is_pressed { 0x00A16903 } else { 0x00C78402 };
                    (c, 0x00FFFFFF, "🔍  Ağ Cihazlarını Tara")
                }
                ID_BTN_FILL_DISCOVERED => {
                    let c = if is_pressed { 0x00261E17 } else { 0x003B291E };
                    (c, 0x00CBD5E1, "Seçili Cihazı Doldur")
                }
                _ => (0x003B291E, 0x00FFFFFF, ""),
            };

            draw_rounded_card(hdc, r.left, r.top, r.right, r.bottom, bg_bgr, bg_bgr, 8);

            let font = CreateFontW(13, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"));
            let old_font = SelectObject(hdc, font);
            let _ = SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, COLORREF(text_color));

            let mut text_rect = r;
            let mut utf16: Vec<u16> = label.encode_utf16().collect();
            let _ = DrawTextW(hdc, &mut utf16, &mut text_rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

            SelectObject(hdc, old_font);
            let _ = DeleteObject(font);
            LRESULT(1)
        }

        WM_MOUSEMOVE => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            let prev_hover = HOVERED_ADMIN_BTN;
            let mut new_hover = 0;

            if is_inside(x, y, &RECT_ADMIN_CLOSE) {
                new_hover = 1;
            } else if is_inside(x, y, &RECT_ADMIN_MIN) {
                new_hover = 2;
            }

            if new_hover != prev_hover {
                HOVERED_ADMIN_BTN = new_hover;
                let _ = InvalidateRect(hwnd, None, false);
            }

            if new_hover != 0 {
                windows::Win32::UI::WindowsAndMessaging::SetCursor(LoadCursorW(None, IDC_HAND).unwrap_or_default());
            }

            LRESULT(0)
        }

        WM_LBUTTONDOWN => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            if is_inside(x, y, &RECT_ADMIN_CLOSE) {
                PostQuitMessage(0);
                std::process::exit(0);
            } else if is_inside(x, y, &RECT_ADMIN_MIN) {
                let _ = ShowWindow(hwnd, SW_MINIMIZE);
            } else if y < 44 && x < 788 {
                let _ = ReleaseCapture();
                let _ = SendMessageW(
                    hwnd,
                    WM_SYSCOMMAND,
                    WPARAM(0xF012),
                    LPARAM(0),
                );
            }
            LRESULT(0)
        }

        WM_COMMAND => {
            let id = wparam.0 as usize & 0xFFFF;
            if id == ID_BTN_COPY_ID {
                if let Some(state) = GLOBAL_ADMIN_STATE.get() {
                    let id_str = state.info.lock().unwrap().device_id.clone();
                    copy_to_clipboard(&id_str);
                }
            } else if id == ID_BTN_REFRESH_PIN {
                if let Some(state) = GLOBAL_ADMIN_STATE.get() {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let new_pin: u16 = rng.gen_range(1000..9999);
                    state.info.lock().unwrap().pin = new_pin.to_string();
                    let _ = InvalidateRect(hwnd, None, true);
                }
            } else if id == ID_BTN_CONNECT {
                let target_raw = get_edit_text(TARGET_EDIT_HWND);
                let pin_raw = get_edit_text(PIN_EDIT_HWND);
                let target = target_raw.trim().replace(" ", "");
                let pin = pin_raw.trim().to_string();

                if target.is_empty() {
                    windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
                        hwnd,
                        w!("Lütfen bir hedef OxiDesk ID veya IP adresi girin."),
                        w!("OxiDesk Bilgilendirme"),
                        windows::Win32::UI::WindowsAndMessaging::MB_OK | windows::Win32::UI::WindowsAndMessaging::MB_ICONWARNING,
                    );
                } else {
                    launch_native_viewer(&target, &pin);
                }
            } else if id == ID_BTN_SCAN {
                if let Some(state) = GLOBAL_ADMIN_STATE.get() {
                    let local_ip = state.info.lock().unwrap().local_ip.clone();
                    let st = state.clone();
                    tokio::spawn(async move {
                        let _ = crate::discovery::DiscoveryEngine::scan_subnet(&local_ip, st).await;
                        unsafe {
                            let _ = InvalidateRect(MAIN_ADMIN_HWND, None, true);
                        }
                    });
                }
            } else if id == ID_BTN_FILL_DISCOVERED {
                if let Some(state) = GLOBAL_ADMIN_STATE.get() {
                    let disc = state.discovered.lock().unwrap();
                    if let Some((_, dev)) = disc.iter().next() {
                        let target_str = dev.device_id.clone().unwrap_or_else(|| format!("{}:{}", dev.ip, dev.port));
                        let utf16: Vec<u16> = target_str.encode_utf16().chain(std::iter::once(0)).collect();
                        let _ = SetWindowTextW(TARGET_EDIT_HWND, PCWSTR::from_raw(utf16.as_ptr()));
                    }
                }
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

            // Zero-flicker double-buffered rendering
            let mem_dc = CreateCompatibleDC(hdc);
            let mem_bmp = CreateCompatibleBitmap(hdc, w, h);
            let old_bmp = SelectObject(mem_dc, mem_bmp);

            paint_fluent_admin_gui(mem_dc, w, h);

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

unsafe fn paint_fluent_admin_gui(hdc: HDC, w: i32, h: i32) {
    let full_rect = RECT { left: 0, top: 0, right: w, bottom: h };

    // 1. Fluent Dark Background (#0B0F19)
    let bg_brush = CreateSolidBrush(COLORREF(0x00190F0B));
    FillRect(hdc, &full_rect, bg_brush);
    let _ = DeleteObject(bg_brush);

    // 2. Custom Fluent Title Bar (#0F172A)
    let tb_brush = CreateSolidBrush(COLORREF(0x002A170F));
    let tb_rect = RECT { left: 0, top: 0, right: w, bottom: 42 };
    FillRect(hdc, &tb_rect, tb_brush);
    let _ = DeleteObject(tb_brush);

    let tb_pen = CreatePen(PS_SOLID, 1, COLORREF(0x003B291E));
    let old_pen = SelectObject(hdc, tb_pen);
    let _ = windows::Win32::Graphics::Gdi::MoveToEx(hdc, 0, 42, None);
    let _ = windows::Win32::Graphics::Gdi::LineTo(hdc, w, 42);
    let _ = DeleteObject(tb_pen);

    let _ = SetBkMode(hdc, TRANSPARENT);

    // Modern Segoe UI Fonts
    let title_font = CreateFontW(15, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"));
    let heading_font = CreateFontW(12, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"));
    let label_font = CreateFontW(11, 0, 0, 0, FW_SEMIBOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"));
    let id_font = CreateFontW(28, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"));
    let pin_font = CreateFontW(22, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"));
    let body_font = CreateFontW(11, 0, 0, 0, FW_NORMAL.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"));

    // Titlebar Branding
    SelectObject(hdc, title_font);
    SetTextColor(hdc, COLORREF(0x00F8BA38));
    let mut title_r = RECT { left: 20, top: 0, right: 340, bottom: 42 };
    let mut t_title: Vec<u16> = "OxiDesk IT Yönetim Konsolu  v5.5.0".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_title, &mut title_r, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    // Titlebar Buttons
    // Minimize
    if HOVERED_ADMIN_BTN == 2 {
        let b = CreateSolidBrush(COLORREF(0x0033291E));
        FillRect(hdc, &RECT_ADMIN_MIN, b);
        let _ = DeleteObject(b);
    }
    SetTextColor(hdc, COLORREF(0x00CBD5E1));
    let mut min_r = RECT_ADMIN_MIN;
    let mut t_min: Vec<u16> = "—".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_min, &mut min_r, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    // Close
    if HOVERED_ADMIN_BTN == 1 {
        let b = CreateSolidBrush(COLORREF(0x002311E8));
        FillRect(hdc, &RECT_ADMIN_CLOSE, b);
        let _ = DeleteObject(b);
        SetTextColor(hdc, COLORREF(0x00FFFFFF));
    } else {
        SetTextColor(hdc, COLORREF(0x00CBD5E1));
    }
    let mut close_r = RECT_ADMIN_CLOSE;
    let mut t_close: Vec<u16> = "✕".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_close, &mut close_r, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    // =========================================================================
    // Card 1: Bu Cihazı Yönetime Aç (Left Box: x: 25..415, y: 56..326)
    // =========================================================================
    draw_rounded_card(hdc, 25, 56, 415, 326, 0x00241A13, 0x003D2E22, 16);

    SelectObject(hdc, heading_font);
    SetTextColor(hdc, COLORREF(0x00F8BA38));
    let mut c1_head = RECT { left: 45, top: 72, right: 395, bottom: 92 };
    let mut t_c1_h: Vec<u16> = "BU CİHAZI YÖNETİME AÇ".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_c1_h, &mut c1_head, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    let (device_id_str, pin_str, local_ip, port) = if let Some(state) = GLOBAL_ADMIN_STATE.get() {
        let info = state.info.lock().unwrap();
        (info.device_id.clone(), info.pin.clone(), info.local_ip.clone(), info.port)
    } else {
        ("123456789".to_string(), "1234".to_string(), "127.0.0.1".to_string(), 8080)
    };

    let formatted_id = if device_id_str.len() == 9 {
        format!("{} {} {}", &device_id_str[0..3], &device_id_str[3..6], &device_id_str[6..9])
    } else {
        device_id_str
    };

    SelectObject(hdc, label_font);
    SetTextColor(hdc, COLORREF(0x008B9AA6));
    let mut c1_lbl = RECT { left: 45, top: 104, right: 395, bottom: 122 };
    let mut t_c1_l: Vec<u16> = "Bu Bilgisayarın OxiDesk ID'si:".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_c1_l, &mut c1_lbl, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    SelectObject(hdc, id_font);
    SetTextColor(hdc, COLORREF(0x00F8BA38)); // Neon Cyan
    let mut id_r = RECT { left: 45, top: 124, right: 395, bottom: 168 };
    let mut t_id: Vec<u16> = formatted_id.encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_id, &mut id_r, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    SelectObject(hdc, label_font);
    SetTextColor(hdc, COLORREF(0x008B9AA6));
    let mut pin_lbl = RECT { left: 45, top: 172, right: 200, bottom: 190 };
    let mut t_pin_lbl: Vec<u16> = "Geçici PIN Kodu:".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_pin_lbl, &mut pin_lbl, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    SelectObject(hdc, pin_font);
    SetTextColor(hdc, COLORREF(0x0024BFFA)); // Amber PIN
    let mut pin_r = RECT { left: 45, top: 192, right: 200, bottom: 228 };
    let mut t_pin: Vec<u16> = pin_str.encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_pin, &mut pin_r, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    SelectObject(hdc, body_font);
    SetTextColor(hdc, COLORREF(0x0064748B));
    let ip_info = format!("Yerel Ağ: {}:{}", local_ip, port);
    let mut ip_r = RECT { left: 220, top: 198, right: 395, bottom: 222 };
    let mut t_ip: Vec<u16> = ip_info.encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_ip, &mut ip_r, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    // =========================================================================
    // Card 2: Uzak Bilgisayara Bağlan (Right Box: x: 440..855, y: 56..326)
    // =========================================================================
    draw_rounded_card(hdc, 440, 56, 855, 326, 0x00241A13, 0x003D2E22, 16);

    SelectObject(hdc, heading_font);
    SetTextColor(hdc, COLORREF(0x0081B910)); // Green header
    let mut c2_head = RECT { left: 465, top: 72, right: 835, bottom: 92 };
    let mut t_c2_h: Vec<u16> = "UZAK BİLGİSAYARA BAĞLAN".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_c2_h, &mut c2_head, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    SelectObject(hdc, label_font);
    SetTextColor(hdc, COLORREF(0x00CBD5E1));
    let mut t_lbl = RECT { left: 465, top: 130, right: 835, bottom: 150 };
    let mut t_target_lbl: Vec<u16> = "Hedef OxiDesk ID veya IP:".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_target_lbl, &mut t_lbl, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    let mut p_lbl = RECT { left: 465, top: 196, right: 835, bottom: 216 };
    let mut t_pin_title: Vec<u16> = "Bağlantı PIN Kodu:".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_pin_title, &mut p_lbl, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    // =========================================================================
    // Card 3: Ağdaki Cihazlar & Canlı SOS Talepleri (Bottom: x: 25..855, y: 344..615)
    // =========================================================================
    draw_rounded_card(hdc, 25, 344, 855, 615, 0x001F1811, 0x0037291F, 16);

    SelectObject(hdc, heading_font);
    SetTextColor(hdc, COLORREF(0x00CBD5E1));
    let mut c3_head = RECT { left: 45, top: 360, right: 835, bottom: 382 };
    let mut t_c3_h: Vec<u16> = "AĞDAKİ CİHAZLAR & CANLI SOS HIZLI DESTEK TALEPLERİ (LAN RADAR)".encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut t_c3_h, &mut c3_head, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

    // Device Items Listing
    SelectObject(hdc, body_font);
    let mut y_offset = 396;
    if let Some(state) = GLOBAL_ADMIN_STATE.get() {
        let disc = state.discovered.lock().unwrap();
        if disc.is_empty() {
            SetTextColor(hdc, COLORREF(0x0064748B));
            let mut empty_rect = RECT { left: 45, top: y_offset, right: 835, bottom: y_offset + 25 };
            let mut t_empty: Vec<u16> = "Henüz keşfedilen cihaz yok. 'Ağ Cihazlarını Tara' butonuna basarak tarayabilirsiniz.".encode_utf16().collect();
            let _ = DrawTextW(hdc, &mut t_empty, &mut empty_rect, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
        } else {
            for (_, dev) in disc.iter().take(5) {
                let id_text = dev.device_id.clone().unwrap_or_else(|| "-".to_string());
                let sos_tag = if dev.is_sos { "🚨 SOS YARDIM BEKLİYOR" } else { "● Hazır" };
                let line_str = format!("{} ({})  |  ID: {}  |  IP: {}:{}  |  [{}]", dev.hostname, dev.user_name.clone().unwrap_or_default(), id_text, dev.ip, dev.port, sos_tag);

                SetTextColor(hdc, if dev.is_sos { COLORREF(0x004444EF) } else { COLORREF(0x0081B910) });
                let mut line_rect = RECT { left: 45, top: y_offset, right: 835, bottom: y_offset + 24 };
                let mut t_line: Vec<u16> = line_str.encode_utf16().collect();
                let _ = DrawTextW(hdc, &mut t_line, &mut line_rect, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
                y_offset += 26;
            }
        }
    }

    // Outer 1px Modern Slate Border
    let border_pen = CreatePen(PS_SOLID, 1, COLORREF(0x003B291E));
    SelectObject(hdc, border_pen);
    let null_brush = windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::GET_STOCK_OBJECT_FLAGS(5));
    SelectObject(hdc, null_brush);
    let _ = RoundRect(hdc, 0, 0, w, h, 16, 16);
    let _ = DeleteObject(border_pen);

    // Cleanup GDI
    SelectObject(hdc, old_pen);
    let _ = DeleteObject(title_font);
    let _ = DeleteObject(heading_font);
    let _ = DeleteObject(label_font);
    let _ = DeleteObject(id_font);
    let _ = DeleteObject(pin_font);
    let _ = DeleteObject(body_font);
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

// =========================================================================
// Native Remote Desktop Viewer Window (Pure Win32 GDI & WebSocket Streaming)
// =========================================================================

struct ViewerState {
    frame_pixels: Arc<Mutex<Option<(Vec<u8>, u32, u32)>>>,
    target_addr: String,
    pin: String,
    status: Arc<Mutex<String>>,
}

static VIEWER_STATE: std::sync::RwLock<Option<Arc<ViewerState>>> = std::sync::RwLock::new(None);

fn launch_native_viewer(target: &str, pin: &str) {
    let clean_target = target.replace(" ", "");
    let clean_pin = pin.to_string();

    let mut resolved_host = clean_target.clone();
    if let Some(state) = GLOBAL_ADMIN_STATE.get() {
        let disc = state.discovered.lock().unwrap();
        for (_, dev) in disc.iter() {
            if let Some(id) = &dev.device_id {
                if id.replace(" ", "") == clean_target {
                    resolved_host = format!("{}:{}", dev.ip, dev.port);
                    break;
                }
            }
            if dev.ip == clean_target {
                resolved_host = format!("{}:{}", dev.ip, dev.port);
                break;
            }
        }
    }

    if !resolved_host.contains(':') {
        resolved_host = format!("{}:8080", resolved_host);
    }

    let viewer_state = Arc::new(ViewerState {
        frame_pixels: Arc::new(Mutex::new(None)),
        target_addr: resolved_host.clone(),
        pin: clean_pin.clone(),
        status: Arc::new(Mutex::new(format!("Bağlanılıyor: {}", resolved_host))),
    });

    *VIEWER_STATE.write().unwrap() = Some(viewer_state.clone());

    std::thread::spawn(move || {
        unsafe {
            let instance = GetModuleHandleW(None).unwrap_or_default();
            let class_name = w!("OxiDeskNativeViewerWindowClass");

            let bg_brush = CreateSolidBrush(COLORREF(0x00000000));

            let wnd_class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(viewer_wnd_proc),
                hInstance: instance.into(),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                hbrBackground: bg_brush,
                lpszClassName: class_name,
                ..Default::default()
            };

            let _ = RegisterClassW(&wnd_class);

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class_name,
                w!("OxiDesk Uzak Masaüstü (Native Viewer)"),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                1280,
                760,
                None,
                None,
                instance,
                None,
            );

            if let Ok(hwnd) = hwnd {
                apply_fluent_window_effects(hwnd);
                let _ = ShowWindow(hwnd, SW_SHOW);

                start_native_ws_client(hwnd, viewer_state);

                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                    let _ = windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
    });
}

fn start_native_ws_client(hwnd: HWND, vs: Arc<ViewerState>) {
    let ws_url = format!("ws://{}/ws/client?pin={}", vs.target_addr, vs.pin);
    let hwnd_raw = hwnd.0 as usize;

    tokio::spawn(async move {
        use futures_util::StreamExt;
        use tokio_tungstenite::connect_async;

        match connect_async(&ws_url).await {
            Ok((ws_stream, _)) => {
                *vs.status.lock().unwrap() = format!("Bağlı: {}", vs.target_addr);
                unsafe { let _ = InvalidateRect(HWND(hwnd_raw as _), None, false); }

                let (_, mut read) = ws_stream.split();

                while let Some(Ok(msg)) = read.next().await {
                    if msg.is_binary() {
                        let bytes = msg.into_data();
                        let mut decoder = jpeg_decoder::Decoder::new(&bytes[..]);
                        if let Ok(pixels) = decoder.decode() {
                            if let Some(info) = decoder.info() {
                                let w = info.width as u32;
                                let h = info.height as u32;

                                let mut bgrx = Vec::with_capacity((w * h * 4) as usize);
                                for chunk in pixels.chunks_exact(3) {
                                    bgrx.push(chunk[2]);
                                    bgrx.push(chunk[1]);
                                    bgrx.push(chunk[0]);
                                    bgrx.push(0);
                                }

                                *vs.frame_pixels.lock().unwrap() = Some((bgrx, w, h));
                                unsafe {
                                    let _ = InvalidateRect(HWND(hwnd_raw as _), None, false);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                *vs.status.lock().unwrap() = format!("Bağlantı Hatası: {}", e);
                unsafe { let _ = InvalidateRect(HWND(hwnd_raw as _), None, true); }
            }
        }
    });
}

unsafe extern "system" fn viewer_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut client_rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut client_rect);

            let vs_opt = VIEWER_STATE.read().unwrap().clone();
            if let Some(vs) = &vs_opt {
                let lock = vs.frame_pixels.lock().unwrap();
                if let Some((bgrx, img_w, img_h)) = lock.as_ref() {
                    let mut bmi = BITMAPINFO {
                        bmiHeader: BITMAPINFOHEADER {
                            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                            biWidth: *img_w as i32,
                            biHeight: -(*img_h as i32),
                            biPlanes: 1,
                            biBitCount: 32,
                            biCompression: BI_RGB.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let _ = SetDIBitsToDevice(
                        hdc,
                        0,
                        0,
                        client_rect.right as u32,
                        client_rect.bottom as u32,
                        0,
                        0,
                        0,
                        *img_h,
                        bgrx.as_ptr() as *const _,
                        &mut bmi,
                        DIB_RGB_COLORS,
                    );
                } else {
                    let status = vs.status.lock().unwrap().clone();
                    let _ = SetBkMode(hdc, TRANSPARENT);
                    SetTextColor(hdc, COLORREF(0x00F8BA38));
                    draw_text_helper(hdc, &status, &mut client_rect, DT_CENTER | DT_SINGLELINE | DT_NOPREFIX);
                }
            }

            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_DESTROY => {
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
