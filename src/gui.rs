use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

pub struct GuiLauncher;

impl GuiLauncher {
    /// Run a native embedded frameless WebView2 desktop window directly inside this process
    pub fn run_native_window(url: &str, title: &str, width: u32, height: u32) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(title)
            .with_decorations(false) // 100% Frameless Windows 11 Fluent window
            .with_inner_size(LogicalSize::new(width as f64, height as f64))
            .build(&event_loop)
            .expect("Failed to build native window");

        use std::sync::Arc;
        let window = Arc::new(window);
        let win_for_ipc = window.clone();

        // Prevent EBWebView / .WebView2 folder creation next to executable:
        // Direct all WebView2 caches and runtime files to %LOCALAPPDATA%\OxiDesk\WebView2
        let user_data_dir = std::env::var_os("LOCALAPPDATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("OxiDesk")
            .join("WebView2");
        let _ = std::fs::create_dir_all(&user_data_dir);
        std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", &user_data_dir);

        // Also clean up any legacy EBWebView directory in current working folder if present
        if let Ok(cwd) = std::env::current_dir() {
            let legacy_eb = cwd.join("EBWebView");
            if legacy_eb.exists() && legacy_eb.is_dir() {
                let _ = std::fs::remove_dir_all(&legacy_eb);
            }
        }

        let _webview = WebViewBuilder::new()
            .with_url(url)
            .with_ipc_handler(move |req| {
                let msg = req.body();
                if msg == "close" {
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
                        let _ = SendMessageW(hwnd, WM_SYSCOMMAND, windows::Win32::Foundation::WPARAM(0xF012), windows::Win32::Foundation::LPARAM(0));
                    }
                    let _ = win_for_ipc.drag_window();
                }
            })
            .build(&*window)
            .expect("Failed to create embedded WebView2");

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested | WindowEvent::Destroyed,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                    crate::force_terminate_process();
                }
                Event::LoopDestroyed => {
                    crate::force_terminate_process();
                }
                _ => (),
            }
        });
    }
}
