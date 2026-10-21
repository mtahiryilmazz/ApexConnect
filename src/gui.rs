use std::sync::Arc;
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

pub struct GuiLauncher;

impl GuiLauncher {
    /// Run a native embedded frameless desktop window with zero folders next to executable
    pub fn run_native_window(url: &str, title: &str, width: u32, height: u32) {
        // Direct all WebView runtime caches to system %TEMP% so ZERO folders are created next to executable
        let runtime_cache_dir = std::env::temp_dir().join("OxiDesk_Runtime");
        let _ = std::fs::create_dir_all(&runtime_cache_dir);
        std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", &runtime_cache_dir);

        // Actively clean up any legacy EBWebView folder next to exe if present
        if let Ok(cwd) = std::env::current_dir() {
            let eb_dir = cwd.join("EBWebView");
            if eb_dir.exists() && eb_dir.is_dir() {
                let _ = std::fs::remove_dir_all(&eb_dir);
            }
        }
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let eb_dir = exe_dir.join("EBWebView");
                if eb_dir.exists() && eb_dir.is_dir() {
                    let _ = std::fs::remove_dir_all(&eb_dir);
                }
            }
        }

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(title)
            .with_decorations(false) // 100% Frameless modern Fluent window
            .with_inner_size(LogicalSize::new(width as f64, height as f64))
            .build(&event_loop)
            .expect("Pencere oluşturulamadı");

        let window = Arc::new(window);
        let win_for_ipc = window.clone();

        let _webview = WebViewBuilder::new()
            .with_url(url)
            .with_ipc_handler(move |req| {
                let msg = req.body();
                if msg == "close" {
                    cleanup_runtime_cache();
                    crate::force_terminate_process();
                } else if msg == "minimize" {
                    win_for_ipc.set_minimized(true);
                } else if msg == "maximize" {
                    let is_max = win_for_ipc.is_maximized();
                    win_for_ipc.set_maximized(!is_max);
                } else if msg == "drag" {
                    #[cfg(target_os = "windows")]
                    unsafe {
                        use windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
                        use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, SendMessageW, WM_SYSCOMMAND};
                        let _ = ReleaseCapture();
                        let hwnd = GetForegroundWindow();
                        let _ = SendMessageW(
                            hwnd,
                            WM_SYSCOMMAND,
                            windows::Win32::Foundation::WPARAM(0xF012),
                            windows::Win32::Foundation::LPARAM(0),
                        );
                    }
                    let _ = win_for_ipc.drag_window();
                }
            })
            .build(&*window)
            .expect("Masaüstü arayüzü başlatılamadı");

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested | WindowEvent::Destroyed,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                    cleanup_runtime_cache();
                    crate::force_terminate_process();
                }
                Event::LoopDestroyed => {
                    cleanup_runtime_cache();
                    crate::force_terminate_process();
                }
                _ => (),
            }
        });
    }
}

fn cleanup_runtime_cache() {
    if let Ok(cwd) = std::env::current_dir() {
        let eb_dir = cwd.join("EBWebView");
        if eb_dir.exists() {
            let _ = std::fs::remove_dir_all(&eb_dir);
        }
    }
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let eb_dir = exe_dir.join("EBWebView");
            if eb_dir.exists() {
                let _ = std::fs::remove_dir_all(&eb_dir);
            }
        }
    }
}
