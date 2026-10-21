use std::sync::{Arc, Mutex};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint,
    FillRect, InvalidateRect, SelectObject, SetBkMode,
    SetTextColor, SetDIBitsToDevice, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    DRAW_TEXT_FORMAT, DT_CENTER, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE,
    FW_BOLD, FW_SEMIBOLD, HDC, PAINTSTRUCT, TRANSPARENT,
};
use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GHND};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
    GetWindowTextLengthW, GetWindowTextW, PostQuitMessage, RegisterClassW, SetWindowTextW,
    ShowWindow, BS_PUSHBUTTON, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, ES_AUTOHSCROLL,
    HMENU, MSG, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_COMMAND, WM_CREATE,
    WM_DESTROY, WM_PAINT, WNDCLASSW, WS_BORDER,
    WS_CHILD, WS_CLIPCHILDREN, WS_OVERLAPPED, WS_OVERLAPPEDWINDOW, WS_SYSMENU,
    WS_TABSTOP, WS_VISIBLE,
};

use crate::state::SharedState;

static GLOBAL_ADMIN_STATE: std::sync::OnceLock<SharedState> = std::sync::OnceLock::new();
static mut TARGET_EDIT_HWND: HWND = HWND(0 as _);
static mut PIN_EDIT_HWND: HWND = HWND(0 as _);
static mut MAIN_ADMIN_HWND: HWND = HWND(0 as _);

const ID_BTN_COPY_ID: usize = 201;
const ID_BTN_REFRESH_PIN: usize = 202;
const ID_BTN_CONNECT: usize = 203;
const ID_BTN_SCAN: usize = 204;
const ID_BTN_FILL_DISCOVERED: usize = 205;

pub struct NativeAdminGui;

impl NativeAdminGui {
    pub fn run(state: SharedState) {
        unsafe {
            let _ = GLOBAL_ADMIN_STATE.set(state);

            let instance = GetModuleHandleW(None).unwrap_or_default();
            let class_name = w!("OxiDeskAdminNativeClass");

            let bg_brush = CreateSolidBrush(COLORREF(0x001A120F)); // Dark background #0F121A

            let wnd_class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(admin_wnd_proc),
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
                w!("OxiDesk - IT Uzak Masaüstü Yönetim Konsolu v5.0.0 (Saf Win32 AnyDesk Modeli)"),
                WS_OVERLAPPED | WS_SYSMENU | WS_CLIPCHILDREN,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                860,
                620,
                None,
                None,
                instance,
                None,
            );

            if hwnd.is_err() {
                return;
            }

            let hwnd = hwnd.unwrap();
            MAIN_ADMIN_HWND = hwnd;

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

unsafe extern "system" fn admin_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let instance = GetModuleHandleW(None).unwrap_or_default();
            let btn_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_PUSHBUTTON as u32);
            let edit_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_BORDER | WINDOW_STYLE(ES_AUTOHSCROLL as u32);

            // Left Card Buttons: [Kimliği Kopyala] and [🔄 PIN Yenile]
            let _ = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("Kimliği Kopyala"),
                btn_style,
                45,
                245,
                140,
                34,
                hwnd,
                HMENU(ID_BTN_COPY_ID as _),
                instance,
                None,
            );

            let _ = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("Yeni PIN"),
                btn_style,
                200,
                245,
                120,
                34,
                hwnd,
                HMENU(ID_BTN_REFRESH_PIN as _),
                instance,
                None,
            );

            // Right Card Inputs: Target OxiDesk ID and PIN
            let target_edit = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("EDIT"),
                w!(""),
                edit_style,
                460,
                160,
                330,
                30,
                hwnd,
                HMENU(301 as _),
                instance,
                None,
            );
            if let Ok(e) = target_edit {
                TARGET_EDIT_HWND = e;
            }

            let pin_edit = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("EDIT"),
                w!(""),
                edit_style,
                460,
                225,
                160,
                30,
                hwnd,
                HMENU(302 as _),
                instance,
                None,
            );
            if let Ok(e) = pin_edit {
                PIN_EDIT_HWND = e;
            }

            // [ 🚀 Uzak Oturumu Başlat ] Button
            let _ = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("🚀 Uzak Oturumu Başlat"),
                btn_style,
                460,
                265,
                330,
                42,
                hwnd,
                HMENU(ID_BTN_CONNECT as _),
                instance,
                None,
            );

            // Bottom Section: [ 🔍 Ağı Tara ] Button
            let _ = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("🔍 Ağ Cihazlarını Tara (Scan)"),
                btn_style,
                45,
                520,
                240,
                36,
                hwnd,
                HMENU(ID_BTN_SCAN as _),
                instance,
                None,
            );

            let _ = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("BUTTON"),
                w!("Bulunan Cihazı Seç"),
                btn_style,
                300,
                520,
                180,
                36,
                hwnd,
                HMENU(ID_BTN_FILL_DISCOVERED as _),
                instance,
                None,
            );

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
                        w!("OxiDesk Uyarı"),
                        windows::Win32::UI::WindowsAndMessaging::MB_OK | windows::Win32::UI::WindowsAndMessaging::MB_ICONWARNING,
                    );
                } else {
                    // Launch Native Remote Desktop Viewer Window
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

            paint_admin_gui(hdc, hwnd);

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

unsafe fn paint_admin_gui(hdc: HDC, hwnd: HWND) {
    let mut rect = RECT::default();
    let _ = GetClientRect(hwnd, &mut rect);

    // Dark Background
    let bg_brush = CreateSolidBrush(COLORREF(0x001A120F)); // #0F121A
    FillRect(hdc, &rect, bg_brush);
    let _ = DeleteObject(bg_brush);

    let _ = SetBkMode(hdc, TRANSPARENT);

    // Fonts
    let title_font = CreateFontW(
        22, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"),
    );
    let heading_font = CreateFontW(
        15, 0, 0, 0, FW_SEMIBOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"),
    );
    let label_font = CreateFontW(
        12, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"),
    );
    let id_font = CreateFontW(
        30, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"),
    );
    let pin_font = CreateFontW(
        22, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, 1, 0, 0, 2, 0, w!("Segoe UI"),
    );

    // Header Title
    SelectObject(hdc, title_font);
    SetTextColor(hdc, COLORREF(0x00F3F3F3));
    let mut title_rect = RECT { left: 35, top: 20, right: rect.right, bottom: 55 };
    draw_text_helper(
        hdc,
        "OxiDesk IT Yönetim Konsolu (Native Fluent GUI)",
        &mut title_rect,
        DT_LEFT | DT_SINGLELINE | DT_NOPREFIX,
    );

    // Card 1: Bu Cihazı Yönetime Aç (Left Box: x: 30..400)
    let card1_brush = CreateSolidBrush(COLORREF(0x002B211A));
    let card1_rect = RECT { left: 30, top: 70, right: 410, bottom: 330 };
    FillRect(hdc, &card1_rect, card1_brush);
    let _ = DeleteObject(card1_brush);

    SelectObject(hdc, heading_font);
    SetTextColor(hdc, COLORREF(0x00E0E6ED));
    let mut c1_head = RECT { left: 45, top: 85, right: 395, bottom: 105 };
    draw_text_helper(hdc, "BU CİHAZI YÖNETİME AÇ", &mut c1_head, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);

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
    SetTextColor(hdc, COLORREF(0x009BA0A6));
    let mut c1_lbl = RECT { left: 45, top: 115, right: 395, bottom: 135 };
    draw_text_helper(hdc, "Bu Bilgisayarın OxiDesk ID'si:", &mut c1_lbl, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);

    SelectObject(hdc, id_font);
    SetTextColor(hdc, COLORREF(0x00FFCD60)); // Cyan
    let mut id_rect = RECT { left: 45, top: 135, right: 395, bottom: 175 };
    draw_text_helper(hdc, &formatted_id, &mut id_rect, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);

    SelectObject(hdc, label_font);
    SetTextColor(hdc, COLORREF(0x009BA0A6));
    let mut pin_lbl = RECT { left: 45, top: 180, right: 200, bottom: 200 };
    draw_text_helper(hdc, "Geçici PIN Kodu:", &mut pin_lbl, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);

    SelectObject(hdc, pin_font);
    SetTextColor(hdc, COLORREF(0x0024BFFB)); // Gold
    let mut pin_rect = RECT { left: 45, top: 200, right: 200, bottom: 235 };
    draw_text_helper(hdc, &pin_str, &mut pin_rect, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);

    SelectObject(hdc, label_font);
    SetTextColor(hdc, COLORREF(0x005ECC34)); // Green
    let status_str = format!("🟢 Çevrimiçi | Port: {} | Web Portal: http://{}:{}", port, local_ip, port);
    let mut st_rect = RECT { left: 45, top: 295, right: 395, bottom: 315 };
    draw_text_helper(hdc, &status_str, &mut st_rect, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);

    // Card 2: Uzak Cihaza Bağlan (Right Box: x: 440..820)
    let card2_brush = CreateSolidBrush(COLORREF(0x002B211A));
    let card2_rect = RECT { left: 440, top: 70, right: 820, bottom: 330 };
    FillRect(hdc, &card2_rect, card2_brush);
    let _ = DeleteObject(card2_brush);

    SelectObject(hdc, heading_font);
    SetTextColor(hdc, COLORREF(0x00E0E6ED));
    let mut c2_head = RECT { left: 460, top: 85, right: 800, bottom: 105 };
    draw_text_helper(hdc, "UZAK CİHAZA BAĞLAN", &mut c2_head, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);

    SelectObject(hdc, label_font);
    SetTextColor(hdc, COLORREF(0x009BA0A6));
    let mut t_lbl = RECT { left: 460, top: 135, right: 800, bottom: 155 };
    draw_text_helper(hdc, "Hedef OxiDesk ID veya Yerel IP:", &mut t_lbl, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);

    let mut p_lbl = RECT { left: 460, top: 200, right: 800, bottom: 220 };
    draw_text_helper(hdc, "Bağlantı PIN Kodu:", &mut p_lbl, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);

    // Card 3: Ağdaki Cihazlar & Keşif (Bottom Box: x: 30..820)
    let card3_brush = CreateSolidBrush(COLORREF(0x002B211A));
    let card3_rect = RECT { left: 30, top: 350, right: 820, bottom: 575 };
    FillRect(hdc, &card3_rect, card3_brush);
    let _ = DeleteObject(card3_brush);

    SelectObject(hdc, heading_font);
    SetTextColor(hdc, COLORREF(0x00E0E6ED));
    let mut c3_head = RECT { left: 45, top: 365, right: 800, bottom: 385 };
    draw_text_helper(hdc, "AĞDAKİ CİHAZLAR & HIZLI DESTEK TALEPLERİ (LAN / SOS)", &mut c3_head, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);

    // Discovered Devices Listing
    SelectObject(hdc, label_font);
    let mut y_offset = 395;
    if let Some(state) = GLOBAL_ADMIN_STATE.get() {
        let disc = state.discovered.lock().unwrap();
        if disc.is_empty() {
            SetTextColor(hdc, COLORREF(0x0078808A));
            let mut empty_rect = RECT { left: 45, top: y_offset, right: 800, bottom: y_offset + 25 };
            draw_text_helper(hdc, "Henüz keşfedilen cihaz yok. 'Ağ Cihazlarını Tara' butonuna basarak arayabilirsiniz.", &mut empty_rect, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);
        } else {
            for (_, dev) in disc.iter().take(4) {
                let id_text = dev.device_id.clone().unwrap_or_else(|| "-".to_string());
                let sos_tag = if dev.is_sos { "🚨 SOS YARDIM BEKLİYOR" } else { "Hazır" };
                let line_str = format!("• {} ({}) | OxiDesk ID: {} | IP: {}:{} | [{}]", dev.hostname, dev.user_name.clone().unwrap_or_default(), id_text, dev.ip, dev.port, sos_tag);
                
                SetTextColor(hdc, if dev.is_sos { COLORREF(0x0024BFFB) } else { COLORREF(0x005ECC34) });
                let mut line_rect = RECT { left: 45, top: y_offset, right: 800, bottom: y_offset + 22 };
                draw_text_helper(hdc, &line_str, &mut line_rect, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);
                y_offset += 24;
            }
        }
    }

    // Cleanup
    let _ = DeleteObject(title_font);
    let _ = DeleteObject(heading_font);
    let _ = DeleteObject(label_font);
    let _ = DeleteObject(id_font);
    let _ = DeleteObject(pin_font);
}

// =========================================================================
// Native Remote Desktop Viewer Window (Pure Win32 GDI & WebSocket Streaming)
// =========================================================================

struct ViewerState {
    frame_pixels: Arc<Mutex<Option<(Vec<u8>, u32, u32)>>>, // (BGRX pixels, width, height)
    target_addr: String,
    pin: String,
    status: Arc<Mutex<String>>,
}

static VIEWER_STATE: std::sync::RwLock<Option<Arc<ViewerState>>> = std::sync::RwLock::new(None);

fn launch_native_viewer(target: &str, pin: &str) {
    let clean_target = target.replace(" ", "");
    let clean_pin = pin.to_string();

    // Resolve target IP/port if it is an OxiDesk ID
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

    // Spawn native viewer thread
    std::thread::spawn(move || {
        unsafe {
            let instance = GetModuleHandleW(None).unwrap_or_default();
            let class_name = w!("OxiDeskNativeViewerWindowClass");

            let bg_brush = CreateSolidBrush(COLORREF(0x00000000)); // Black

            let wnd_class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(viewer_wnd_proc),
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
                let _ = ShowWindow(hwnd, SW_SHOW);

                // Start WebSocket receiver task
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
                        // Decode JPEG directly into BGRX pixel buffer for blitting
                        let mut decoder = jpeg_decoder::Decoder::new(&bytes[..]);
                        if let Ok(pixels) = decoder.decode() {
                            if let Some(info) = decoder.info() {
                                let w = info.width as u32;
                                let h = info.height as u32;
                                
                                // Convert RGB to 32-bit BGRX for fast GDI blitting
                                let mut bgrx = Vec::with_capacity((w * h * 4) as usize);
                                for chunk in pixels.chunks_exact(3) {
                                    bgrx.push(chunk[2]); // B
                                    bgrx.push(chunk[1]); // G
                                    bgrx.push(chunk[0]); // R
                                    bgrx.push(0);        // X
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
                            biHeight: -(*img_h as i32), // Top-down DIB
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
                    // Draw status text while connecting
                    let status = vs.status.lock().unwrap().clone();
                    let _ = SetBkMode(hdc, TRANSPARENT);
                    SetTextColor(hdc, COLORREF(0x00FFCD60));
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
