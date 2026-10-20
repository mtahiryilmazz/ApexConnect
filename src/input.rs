use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEINPUT, VIRTUAL_KEY, VK_CONTROL, VK_DELETE,
    VK_MENU,
};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

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

    pub fn mouse_move(&self, norm_x: f64, norm_y: f64) {
        let abs_x = (norm_x.clamp(0.0, 1.0) * 65535.0) as i32;
        let abs_y = (norm_y.clamp(0.0, 1.0) * 65535.0) as i32;

        unsafe {
            let input = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: abs_x,
                        dy: abs_y,
                        mouseData: 0,
                        dwFlags: MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
    }

    pub fn mouse_down(&self, button: u8) {
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
}
