use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEINPUT, VIRTUAL_KEY, VK_CONTROL, VK_DELETE,
    VK_MENU,
};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SetCursorPos, SM_CXSCREEN, SM_CYSCREEN};

#[allow(dead_code)]
pub struct InputSimulator {
    pub screen_width: i32,
    pub screen_height: i32,
}

impl InputSimulator {
    pub fn new() -> Self {
        unsafe {
            let screen_width = GetSystemMetrics(SM_CXSCREEN);
            let screen_height = GetSystemMetrics(SM_CYSCREEN);
            Self {
                screen_width,
                screen_height,
            }
        }
    }

    /// Detect Windows Secure Desktop (UAC consent prompt, Winlogon lock screen)
    pub fn is_secure_desktop_active() -> bool {
        #[cfg(target_os = "windows")]
        unsafe {
            use windows::Win32::System::StationsAndDesktops::{
                OpenInputDesktop, CloseDesktop, GetUserObjectInformationW, UOI_NAME,
                DESKTOP_CONTROL_FLAGS, DESKTOP_ACCESS_FLAGS,
            };
            use windows::Win32::Foundation::HANDLE;

            // DESKTOP_SWITCHDESKTOP = 0x0100
            if let Ok(hdesktop) = OpenInputDesktop(DESKTOP_CONTROL_FLAGS(0), false, DESKTOP_ACCESS_FLAGS(0x0100)) {
                let mut buf = [0u16; 64];
                let mut needed = 0u32;
                let handle = HANDLE(hdesktop.0);
                let success = GetUserObjectInformationW(
                    handle,
                    UOI_NAME,
                    Some(buf.as_mut_ptr() as *mut _),
                    (buf.len() * 2) as u32,
                    Some(&mut needed),
                );
                let _ = CloseDesktop(hdesktop);
                if success.is_ok() {
                    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
                    let name = String::from_utf16_lossy(&buf[..len]);
                    return !name.eq_ignore_ascii_case("default");
                }
            }
            false
        }
        #[cfg(not(target_os = "windows"))]
        false
    }

    /// Direct hardware cursor positioning for ultra-low-latency mouse moves (unbuffered fast-path)
    pub fn mouse_move_fast(&self, norm_x: f64, norm_y: f64) {
        if Self::is_secure_desktop_active() {
            return;
        }
        let px = (norm_x.clamp(0.0, 1.0) * self.screen_width as f64) as i32;
        let py = (norm_y.clamp(0.0, 1.0) * self.screen_height as f64) as i32;
        unsafe {
            let _ = SetCursorPos(px, py);
        }
    }

    /// Native Windows CF_HDROP clipboard file integration for instant Explorer paste
    pub fn set_clipboard_files(paths: &[std::path::PathBuf]) {
        if paths.is_empty() {
            return;
        }
        let formatted_paths = paths
            .iter()
            .map(|p| format!("'{}'", p.to_string_lossy().replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(",");

        let ps_cmd = format!("Set-Clipboard -Path @({})", formatted_paths);
        let _ = std::process::Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_cmd])
            .spawn();
    }

    pub fn mouse_move(&self, norm_x: f64, norm_y: f64) {
        self.mouse_move_fast(norm_x, norm_y);
    }

    pub fn mouse_down(&self, button: u8) {
        if Self::is_secure_desktop_active() {
            self.release_all_keys();
            return;
        }

        let flags = match button {
            0 => MOUSEEVENTF_LEFTDOWN,
            1 => MOUSEEVENTF_MIDDLEDOWN,
            2 => MOUSEEVENTF_RIGHTDOWN,
            _ => return,
        };

        unsafe {
            let input = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: 0,
                        dy: 0,
                        mouseData: 0,
                        dwFlags: flags,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
    }

    pub fn mouse_up(&self, button: u8) {
        let flags = match button {
            0 => MOUSEEVENTF_LEFTUP,
            1 => MOUSEEVENTF_MIDDLEUP,
            2 => MOUSEEVENTF_RIGHTUP,
            _ => return,
        };

        unsafe {
            let input = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: 0,
                        dy: 0,
                        mouseData: 0,
                        dwFlags: flags,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
    }

    pub fn mouse_click(&self, button: u8) {
        self.mouse_down(button);
        std::thread::sleep(std::time::Duration::from_millis(15));
        self.mouse_up(button);
    }

    pub fn mouse_wheel(&self, delta: i32) {
        unsafe {
            let input = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: 0,
                        dy: 0,
                        mouseData: delta as u32,
                        dwFlags: MOUSEEVENTF_WHEEL,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
    }

    pub fn type_text(&self, text: &str) {
        for ch in text.encode_utf16() {
            unsafe {
                let down = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(0),
                            wScan: ch,
                            dwFlags: KEYEVENTF_UNICODE,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                let up = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(0),
                            wScan: ch,
                            dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                SendInput(&[down, up], std::mem::size_of::<INPUT>() as i32);
            }
        }
    }

    pub fn send_ctrl_alt_del(&self) {
        // Ctrl + Alt + Delete simulation
        unsafe {
            let ctrl_down = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_CONTROL,
                        wScan: 0,
                        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            let alt_down = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_MENU,
                        wScan: 0,
                        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            let del_down = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_DELETE,
                        wScan: 0,
                        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            let del_up = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_DELETE,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            let alt_up = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_MENU,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            let ctrl_up = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_CONTROL,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };

            SendInput(&[ctrl_down, alt_down, del_down, del_up, alt_up, ctrl_up], std::mem::size_of::<INPUT>() as i32);
        }
    }

    pub fn key_down(&self, key: &str, _code: &str) {
        if Self::is_secure_desktop_active() {
            self.release_all_keys();
            return;
        }

        if let Some(vk) = map_key_to_vk(key) {
            let mut flags = windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0);
            if is_extended_key(vk) {
                flags |= windows::Win32::UI::Input::KeyboardAndMouse::KEYEVENTF_EXTENDEDKEY;
            }

            unsafe {
                let input = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: vk,
                            wScan: 0,
                            dwFlags: flags,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
        }
    }

    pub fn key_up(&self, key: &str, _code: &str) {
        if let Some(vk) = map_key_to_vk(key) {
            let mut flags = KEYEVENTF_KEYUP;
            if is_extended_key(vk) {
                flags |= windows::Win32::UI::Input::KeyboardAndMouse::KEYEVENTF_EXTENDEDKEY;
            }

            unsafe {
                let input = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: vk,
                            wScan: 0,
                            dwFlags: flags,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
        }
    }

    /// Zero-Trust key release safety: Releases all modifier keys and mouse buttons upon disconnect
    pub fn release_all_keys(&self) {
        use windows::Win32::UI::Input::KeyboardAndMouse::*;
        let modifier_vks = [
            VK_CONTROL, VK_LCONTROL, VK_RCONTROL,
            VK_SHIFT, VK_LSHIFT, VK_RSHIFT,
            VK_MENU, VK_LMENU, VK_RMENU,
            VK_LWIN, VK_RWIN,
        ];

        let mut inputs = Vec::new();
        for &vk in &modifier_vks {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: vk,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
        }

        for &flags in &[MOUSEEVENTF_LEFTUP, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_MIDDLEUP] {
            inputs.push(INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: 0,
                        dy: 0,
                        mouseData: 0,
                        dwFlags: flags,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
        }

        unsafe {
            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        }
    }
}

fn map_key_to_vk(key: &str) -> Option<VIRTUAL_KEY> {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    match key {
        "Enter" => Some(VK_RETURN),
        "Backspace" => Some(VK_BACK),
        "Tab" => Some(VK_TAB),
        "Escape" => Some(VK_ESCAPE),
        "Space" | " " => Some(VK_SPACE),
        "ArrowUp" => Some(VK_UP),
        "ArrowDown" => Some(VK_DOWN),
        "ArrowLeft" => Some(VK_LEFT),
        "ArrowRight" => Some(VK_RIGHT),
        "Shift" => Some(VK_SHIFT),
        "Control" => Some(VK_CONTROL),
        "Alt" => Some(VK_MENU),
        "Delete" => Some(VK_DELETE),
        "Home" => Some(VK_HOME),
        "End" => Some(VK_END),
        "PageUp" => Some(VK_PRIOR),
        "PageDown" => Some(VK_NEXT),
        "CapsLock" => Some(VK_CAPITAL),
        _ => None,
    }
}

fn is_extended_key(vk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY) -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    matches!(
        vk,
        VK_UP | VK_DOWN | VK_LEFT | VK_RIGHT | VK_HOME | VK_END | VK_PRIOR | VK_NEXT | VK_DELETE | VK_INSERT | VK_RCONTROL | VK_RMENU
    )
}
