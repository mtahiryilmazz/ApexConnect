use std::path::PathBuf;
use std::process::Command;

pub struct GuiLauncher;

impl GuiLauncher {
    /// Launch the URL in a dedicated Windows 11 Fluent App window (without browser chrome/tabs/address bar)
    pub fn launch(url: &str, _title: &str, width: u32, height: u32, app_name: &str) {
        let url_owned = url.to_string();
        let app_name_owned = app_name.to_string();

        tokio::spawn(async move {
            // Small delay to ensure HTTP server is accepting connections
            tokio::time::sleep(std::time::Duration::from_millis(400)).await;

            let temp_profile = std::env::temp_dir().join(format!("ApexConnect_{}", app_name_owned));
            let _ = std::fs::create_dir_all(&temp_profile);

            let window_arg = format!("--window-size={},{}", width, height);
            let profile_arg = format!("--user-data-dir={}", temp_profile.to_string_lossy());
            let app_arg = format!("--app={}", url_owned);

            // Collect candidate browser executables in order of preference
            let mut candidates: Vec<PathBuf> = Vec::new();

            // 1. Edge Standard Locations
            candidates.push(PathBuf::from(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"));
            candidates.push(PathBuf::from(r"C:\Program Files\Microsoft\Edge\Application\msedge.exe"));

            // 2. EdgeCore Locations (found in enterprise & modern Windows 11 updates)
            if let Ok(entries) = std::fs::read_dir(r"C:\Program Files (x86)\Microsoft\EdgeCore") {
                for entry in entries.flatten() {
                    let exe = entry.path().join("msedge.exe");
                    if exe.exists() {
                        candidates.push(exe);
                    }
                }
            }

            // 3. User LocalAppData Edge
            if let Ok(local) = std::env::var("LOCALAPPDATA") {
                candidates.push(PathBuf::from(&local).join(r"Microsoft\Edge\Application\msedge.exe"));
            }

            // 4. Google Chrome Locations
            candidates.push(PathBuf::from(r"C:\Program Files\Google\Chrome\Application\chrome.exe"));
            candidates.push(PathBuf::from(r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"));
            if let Ok(local) = std::env::var("LOCALAPPDATA") {
                candidates.push(PathBuf::from(&local).join(r"Google\Chrome\Application\chrome.exe"));
            }

            // 5. System PATH fallbacks
            candidates.push(PathBuf::from("msedge.exe"));
            candidates.push(PathBuf::from("chrome.exe"));

            // Launch in standalone Frameless App Window Mode (no browser tabs, no address bar, single Fluent titlebar)
            for exe in &candidates {
                if exe.exists() || exe.to_str() == Some("msedge.exe") || exe.to_str() == Some("chrome.exe") {
                    let res = Command::new(exe)
                        .args([
                            &app_arg,
                            &window_arg,
                            &profile_arg,
                            "--no-first-run",
                            "--no-default-browser-check",
                        ])
                        .spawn();

                    if let Ok(child) = res {
                        let pid = child.id();
                        tokio::spawn(async move {
                            for _ in 0..25 {
                                tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                                #[cfg(target_os = "windows")]
                                make_process_windows_frameless(pid);
                            }
                        });
                        return;
                    }
                }
            }

            // Fallback to standard shell open if standalone app engine fails
            let _ = Command::new("cmd")
                .args(["/C", "start", &url_owned])
                .spawn();
        });
    }
}

#[cfg(target_os = "windows")]
fn make_process_windows_frameless(target_pid: u32) {
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::*;

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let target_pid = lparam.0 as u32;
        let mut proc_id = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut proc_id));

        if proc_id == target_pid && IsWindowVisible(hwnd).as_bool() {
            let style = GetWindowLongW(hwnd, GWL_STYLE);
            if style & (WS_CAPTION.0 as i32) != 0 {
                SetWindowLongW(hwnd, GWL_STYLE, style & !(WS_CAPTION.0 as i32));
                let _ = SetWindowPos(
                    hwnd,
                    HWND(0 as _),
                    0, 0, 0, 0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
                );
            }
        }
        BOOL(1)
    }

    unsafe {
        let _ = EnumWindows(Some(enum_proc), LPARAM(target_pid as isize));
    }
}

