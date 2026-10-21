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

            let edge_candidates = [
                PathBuf::from(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"),
                PathBuf::from(r"C:\Program Files\Microsoft\Edge\Application\msedge.exe"),
                PathBuf::from("msedge.exe"),
            ];

            let chrome_candidates = [
                PathBuf::from(r"C:\Program Files\Google\Chrome\Application\chrome.exe"),
                PathBuf::from(r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"),
                PathBuf::from("chrome.exe"),
            ];

            let window_arg = format!("--window-size={},{}", width, height);
            let profile_arg = format!("--user-data-dir={}", temp_profile.to_string_lossy());
            let app_arg = format!("--app={}", url_owned);

            // 1. Try Microsoft Edge App Mode (Default on Windows 10 & 11)
            for exe in &edge_candidates {
                if exe.exists() || exe.to_str() == Some("msedge.exe") {
                    let res = Command::new(exe)
                        .args([
                            &app_arg,
                            &window_arg,
                            &profile_arg,
                            "--no-first-run",
                            "--no-default-browser-check",
                        ])
                        .spawn();

                    if res.is_ok() {
                        return;
                    }
                }
            }

            // 2. Try Google Chrome App Mode
            for exe in &chrome_candidates {
                if exe.exists() || exe.to_str() == Some("chrome.exe") {
                    let res = Command::new(exe)
                        .args([
                            &app_arg,
                            &window_arg,
                            &profile_arg,
                            "--no-first-run",
                            "--no-default-browser-check",
                        ])
                        .spawn();

                    if res.is_ok() {
                        return;
                    }
                }
            }

            // 3. Fallback to standard shell open if standalone app engine fails
            let _ = Command::new("cmd")
                .args(["/C", "start", &url_owned])
                .spawn();
        });
    }
}
