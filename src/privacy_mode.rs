use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::CreateSolidBrush;
use windows::Win32::UI::Input::KeyboardAndMouse::BlockInput;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{w, PCWSTR};

static IS_PRIVACY_ACTIVE: AtomicBool = AtomicBool::new(false);
static PRIVACY_HWND: Mutex<Option<isize>> = Mutex::new(None);

pub fn is_privacy_mode_active() -> bool {
    IS_PRIVACY_ACTIVE.load(Ordering::SeqCst)
}

/// Enable privacy / curtain mode:
/// - Locks physical keyboard and mouse using BlockInput
/// - Creates a fullscreen black topmost window excluded from capture (WDA_EXCLUDEFROMCAPTURE)
pub fn enable_privacy_mode() -> bool {
    if IS_PRIVACY_ACTIVE.swap(true, Ordering::SeqCst) {
        return true; // Already active
    }

    // 1. Lock physical inputs
    unsafe {
        let _ = BlockInput(true);
    }

    // 2. Spawn thread for black overlay window
    std::thread::spawn(move || {
        unsafe {
            let class_name = w!("OxiDeskPrivacyOverlay");
            let hinstance = windows::Win32::System::LibraryLoader::GetModuleHandleW(PCWSTR::null())
                .unwrap_or_default();

            let black_brush = CreateSolidBrush(COLORREF(0x00000000));

            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(privacy_wndproc),
                hInstance: hinstance.into(),
                hbrBackground: black_brush,
                lpszClassName: class_name,
                ..Default::default()
            };

            let _ = RegisterClassW(&wc);

            let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
            let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
            let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
            let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);

            let (x, y, w, h) = if vw > 0 && vh > 0 {
                (vx, vy, vw, vh)
            } else {
                (0, 0, GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN))
            };

            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                class_name,
                w!("OxiDesk Privacy Mode"),
                WS_POPUP | WS_VISIBLE,
                x,
                y,
                w,
                h,
                HWND::default(),
                HMENU::default(),
                hinstance,
                None,
            );

            if let Ok(valid_hwnd) = hwnd {
                // WDA_EXCLUDEFROMCAPTURE = 0x00000011
                // This excludes the window from Desktop Duplication / GDI capture,
                // while completely covering the physical monitor!
                let _ = SetWindowDisplayAffinity(valid_hwnd, WINDOW_DISPLAY_AFFINITY(0x00000011));
                let _ = SetWindowPos(
                    valid_hwnd,
                    HWND_TOPMOST,
                    x,
                    y,
                    w,
                    h,
                    SWP_SHOWWINDOW,
                );

                if let Ok(mut lock) = PRIVACY_HWND.lock() {
                    *lock = Some(valid_hwnd.0 as isize);
                }

                let mut msg = MSG::default();
                while IS_PRIVACY_ACTIVE.load(Ordering::SeqCst) && GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                let _ = DestroyWindow(valid_hwnd);
            }

            let _ = windows::Win32::Graphics::Gdi::DeleteObject(black_brush);
        }
    });

    println!("[PrivacyMode] Aktif edildi: Fiziksel ekran karartildi ve yerel girisler kilitlendi.");
    true
}

/// Disable privacy / curtain mode and restore screen & input
pub fn disable_privacy_mode() {
    if !IS_PRIVACY_ACTIVE.swap(false, Ordering::SeqCst) {
        return; // Already inactive
    }

    unsafe {
        // Unlock physical inputs
        let _ = BlockInput(false);

        // Close overlay window
        if let Ok(mut lock) = PRIVACY_HWND.lock() {
            if let Some(raw_hwnd) = lock.take() {
                let hwnd = HWND(raw_hwnd as *mut _);
                let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
            }
        }
    }

    println!("[PrivacyMode] Devre disi birakildi: Fiziksel ekran ve yerel kontroller normale dondu.");
}

unsafe extern "system" fn privacy_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
