use windows::core::w;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint,
    FillRect, InvalidateRect, SelectObject, SetBkMode,
    SetTextColor, DRAW_TEXT_FORMAT, DT_CENTER, DT_NOPREFIX, DT_SINGLELINE,
    FW_BOLD, FW_SEMIBOLD, HDC, PAINTSTRUCT, TRANSPARENT,
};
use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GHND};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
    PostQuitMessage, RegisterClassW, SetWindowTextW, ShowWindow,
    BS_PUSHBUTTON, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, HMENU, MSG, SW_SHOW,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_PAINT,
    WNDCLASSW, WS_CHILD, WS_CLIPCHILDREN, WS_OVERLAPPED, WS_SYSMENU, WS_TABSTOP,
    WS_VISIBLE,
};

use crate::state::SharedState;

static GLOBAL_STATE: std::sync::OnceLock<SharedState> = std::sync::OnceLock::new();
static mut COPY_BTN_HWND: HWND = HWND(0 as _);
static mut MAIN_HWND: HWND = HWND(0 as _);

const ID_BTN_COPY: usize = 101;
const ID_BTN_REFRESH: usize = 102;

pub struct NativeQuickSupportGui;

impl NativeQuickSupportGui {
    pub fn run(state: SharedState) {
        unsafe {
            let _ = GLOBAL_STATE.set(state);

            let instance = GetModuleHandleW(None).unwrap_or_default();
            let class_name = w!("OxiDeskQuickSupportNativeClass");

            let bg_brush = CreateSolidBrush(COLORREF(0x001A120F)); // Dark background #0F121A

            let wnd_class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                hInstance: instance.into(),
                hCursor: windows::Win32::UI::WindowsAndMessaging::LoadCursorW(
                    None,
                    windows::Win32::UI::WindowsAndMessaging::IDC_ARROW,
                )
                .unwrap_or_default(),
                hbrBackground: bg_brush,
                lpszClassName: class_name,
                ..Default::default()
            };

            let _ = RegisterClassW(&wnd_class);

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class_name,
                w!("OxiDesk Hızlı Destek v5.0.0 (Saf Win32 AnyDesk Modeli)"),
                WS_OVERLAPPED | WS_SYSMENU | WS_CLIPCHILDREN,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                420,
                580,
                None,
                None,
                instance,
                None,
            );

            if hwnd.is_err() {
                return;
            }

            let hwnd = hwnd.unwrap();
            MAIN_HWND = hwnd;

            let _ = ShowWindow(hwnd, SW_SHOW);

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
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

unsafe fn draw_text_helper(hdc: HDC, text: &str, rect: &mut RECT, format: DRAW_TEXT_FORMAT) {
    let mut utf16: Vec<u16> = text.encode_utf16().collect();
    let _ = DrawTextW(hdc, &mut utf16, rect, format);
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let instance = GetModuleHandleW(None).unwrap_or_default();
            let btn_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_PUSHBUTTON as u32);

            // 1. "Kimliği Kopyala" Button
            let btn_copy = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("Kimliği Kopyala"),
                btn_style,
                110,
                245,
                180,
                38,
                hwnd,
                HMENU(ID_BTN_COPY as _),
                instance,
                None,
            );
            if let Ok(b) = btn_copy {
                COPY_BTN_HWND = b;
            }

            // 2. "Yeni PIN Üret" Button
            let _ = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("🔄 Yeni PIN Üret"),
                btn_style,
                110,
                410,
                180,
                36,
                hwnd,
                HMENU(ID_BTN_REFRESH as _),
                instance,
                None,
            );

            LRESULT(0)
        }
        WM_COMMAND => {
            let id = wparam.0 as usize & 0xFFFF;
            if id == ID_BTN_COPY {
                if let Some(state) = GLOBAL_STATE.get() {
                    let id_str = state.info.lock().unwrap().device_id.clone();
                    copy_to_clipboard(&id_str);
                    let _ = SetWindowTextW(COPY_BTN_HWND, w!("Kopyalandı! ✓"));
                }
            } else if id == ID_BTN_REFRESH {
                if let Some(state) = GLOBAL_STATE.get() {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let new_pin: u16 = rng.gen_range(1000..9999);
                    state.info.lock().unwrap().pin = new_pin.to_string();
                    let _ = InvalidateRect(hwnd, None, true);
                    let _ = SetWindowTextW(COPY_BTN_HWND, w!("Kimliği Kopyala"));
                }
            }
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            paint_native_gui(hdc, hwnd);

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

unsafe fn paint_native_gui(hdc: HDC, hwnd: HWND) {
    let mut rect = RECT::default();
    let _ = GetClientRect(hwnd, &mut rect);

    // Dark Background Fill
    let bg_brush = CreateSolidBrush(COLORREF(0x001A120F)); // #0F121A
    FillRect(hdc, &rect, bg_brush);
    let _ = DeleteObject(bg_brush);

    let _ = SetBkMode(hdc, TRANSPARENT);

    // Modern Segoe UI Fonts
    let title_font = CreateFontW(
        22, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"),
    );
    let badge_font = CreateFontW(
        15, 0, 0, 0, FW_SEMIBOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"),
    );
    let label_font = CreateFontW(
        13, 0, 0, 0, FW_SEMIBOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"),
    );
    let id_font = CreateFontW(
        34, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"),
    );
    let pin_font = CreateFontW(
        28, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"),
    );
    let footer_font = CreateFontW(
        12, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"),
    );

    // 1. Title
    SelectObject(hdc, title_font);
    SetTextColor(hdc, COLORREF(0x00F3F3F3));
    let mut title_rect = RECT { left: 0, top: 20, right: rect.right, bottom: 50 };
    draw_text_helper(
        hdc,
        "OxiDesk Hızlı Destek (Native GUI)",
        &mut title_rect,
        DT_CENTER | DT_SINGLELINE | DT_NOPREFIX,
    );

    // 2. Status Badge ("🟢 Destek Ağına Bağlı")
    SelectObject(hdc, badge_font);
    SetTextColor(hdc, COLORREF(0x005ECC34)); // Green
    let mut badge_rect = RECT { left: 0, top: 52, right: rect.right, bottom: 74 };
    draw_text_helper(
        hdc,
        "🟢 Destek Ağına Bağlı (SOS Aktif)",
        &mut badge_rect,
        DT_CENTER | DT_SINGLELINE | DT_NOPREFIX,
    );

    SetTextColor(hdc, COLORREF(0x009BA0A6)); // Muted
    let mut note_rect = RECT { left: 0, top: 74, right: rect.right, bottom: 95 };
    draw_text_helper(
        hdc,
        "Bilgi İşlem Masası bu bilgisayarı görebilir",
        &mut note_rect,
        DT_CENTER | DT_SINGLELINE | DT_NOPREFIX,
    );

    // Draw Card 1: OxiDesk ID Box
    let card1_brush = CreateSolidBrush(COLORREF(0x002B211A)); // Card dark
    let card1_rect = RECT { left: 25, top: 110, right: rect.right - 25, bottom: 235 };
    FillRect(hdc, &card1_rect, card1_brush);
    let _ = DeleteObject(card1_brush);

    SelectObject(hdc, label_font);
    SetTextColor(hdc, COLORREF(0x00D0C6B8)); // Label
    let mut card1_lbl = RECT { left: 35, top: 122, right: rect.right - 35, bottom: 142 };
    draw_text_helper(
        hdc,
        "BU BİLGİSAYARIN OXİDESK KİMLİĞİ",
        &mut card1_lbl,
        DT_CENTER | DT_SINGLELINE | DT_NOPREFIX,
    );

    // ID Text
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

    SelectObject(hdc, id_font);
    SetTextColor(hdc, COLORREF(0x00FFCD60)); // Cyan #60CDFF
    let mut id_rect = RECT { left: 35, top: 155, right: rect.right - 35, bottom: 215 };
    draw_text_helper(
        hdc,
        &formatted_id,
        &mut id_rect,
        DT_CENTER | DT_SINGLELINE | DT_NOPREFIX,
    );

    // Draw Card 2: One-Time Random PIN Box
    let card2_brush = CreateSolidBrush(COLORREF(0x002B211A));
    let card2_rect = RECT { left: 25, top: 300, right: rect.right - 25, bottom: 400 };
    FillRect(hdc, &card2_rect, card2_brush);
    let _ = DeleteObject(card2_brush);

    SelectObject(hdc, label_font);
    SetTextColor(hdc, COLORREF(0x00D0C6B8));
    let mut card2_lbl = RECT { left: 35, top: 312, right: rect.right - 35, bottom: 332 };
    draw_text_helper(
        hdc,
        "TEK KULLANIMLIK GEÇİCİ PIN (OTP)",
        &mut card2_lbl,
        DT_CENTER | DT_SINGLELINE | DT_NOPREFIX,
    );

    SelectObject(hdc, pin_font);
    SetTextColor(hdc, COLORREF(0x0024BFFB)); // Gold/Yellow #FBBF24
    let mut pin_rect = RECT { left: 35, top: 340, right: rect.right - 35, bottom: 388 };
    draw_text_helper(
        hdc,
        &pin_str,
        &mut pin_rect,
        DT_CENTER | DT_SINGLELINE | DT_NOPREFIX,
    );

    // Footer Info
    SelectObject(hdc, footer_font);
    SetTextColor(hdc, COLORREF(0x0078808A));
    let info_text = format!("Cihaz: {} | Port: {}", hostname, port);
    let mut info_rect = RECT { left: 0, top: 465, right: rect.right, bottom: 485 };
    draw_text_helper(
        hdc,
        &info_text,
        &mut info_rect,
        DT_CENTER | DT_SINGLELINE | DT_NOPREFIX,
    );

    let mut notice_rect = RECT { left: 0, top: 490, right: rect.right, bottom: 520 };
    draw_text_helper(
        hdc,
        "Destek tamamlandığında pencereyi kapatmanız yeterlidir.",
        &mut notice_rect,
        DT_CENTER | DT_SINGLELINE | DT_NOPREFIX,
    );

    // Cleanup GDI objects
    let _ = DeleteObject(title_font);
    let _ = DeleteObject(badge_font);
    let _ = DeleteObject(label_font);
    let _ = DeleteObject(id_font);
    let _ = DeleteObject(pin_font);
    let _ = DeleteObject(footer_font);
}
