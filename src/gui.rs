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

        let _webview = WebViewBuilder::new()
            .with_url(url)
            .with_ipc_handler(move |req| {
                let msg = req.body();
                if msg == "close" {
                    std::process::exit(0);
                } else if msg == "minimize" {
                    win_for_ipc.set_minimized(true);
                } else if msg == "maximize" {
                    let is_max = win_for_ipc.is_maximized();
                    win_for_ipc.set_maximized(!is_max);
                } else if msg == "drag" {
                    let _ = win_for_ipc.drag_window();
                }
            })
            .build(&*window)
            .expect("Failed to create embedded WebView2");

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                    std::process::exit(0);
                }
                _ => (),
            }
        });
    }
}
