use serde::{Deserialize, Serialize};

/// Role-based Access Control (RBAC) permissions for remote sessions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientRole {
    /// Can only view screen and hear audio; mouse, keyboard, and files locked
    ViewOnly,
    /// Full interactive mouse, keyboard, and clipboard control
    FullControl,
    /// Only file transfer upload and download allowed
    FileManager,
    /// Full access plus IT policy configuration and system commands
    Administrator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPolicy {
    pub role: ClientRole,
    pub allow_mouse: bool,
    pub allow_keyboard: bool,
    pub allow_file_transfer: bool,
    pub allow_clipboard_sync: bool,
    pub allow_audio_stream: bool,
    pub max_session_duration_minutes: Option<u32>,
}

impl Default for SessionPolicy {
    fn default() -> Self {
        Self::from_role(ClientRole::FullControl)
    }
}

impl SessionPolicy {
    pub fn from_role(role: ClientRole) -> Self {
        match role {
            ClientRole::ViewOnly => Self {
                role,
                allow_mouse: false,
                allow_keyboard: false,
                allow_file_transfer: false,
                allow_clipboard_sync: false,
                allow_audio_stream: true,
                max_session_duration_minutes: None,
            },
            ClientRole::FullControl => Self {
                role,
                allow_mouse: true,
                allow_keyboard: true,
                allow_file_transfer: true,
                allow_clipboard_sync: true,
                allow_audio_stream: true,
                max_session_duration_minutes: None,
            },
            ClientRole::FileManager => Self {
                role,
                allow_mouse: false,
                allow_keyboard: false,
                allow_file_transfer: true,
                allow_clipboard_sync: false,
                allow_audio_stream: false,
                max_session_duration_minutes: None,
            },
            ClientRole::Administrator => Self {
                role,
                allow_mouse: true,
                allow_keyboard: true,
                allow_file_transfer: true,
                allow_clipboard_sync: true,
                allow_audio_stream: true,
                max_session_duration_minutes: None,
            },
        }
    }

    pub fn can_simulate_input(&self) -> bool {
        self.allow_mouse || self.allow_keyboard
    }
}
