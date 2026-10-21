use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::file_transfer::FileTransferManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostInfo {
    pub device_id: String,
    pub pin: String,
    pub local_ip: String,
    pub port: u16,
    pub hostname: String,
    pub os: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressBookItem {
    pub id: String,
    pub name: String,
    pub group: String,
    pub target: String,
    pub notes: String,
    pub saved_pin: String,
    pub is_online: bool,
    pub last_connected: Option<String>,
    #[serde(default)]
    pub is_trusted_unattended: bool,
    #[serde(default)]
    pub auth_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredDevice {
    pub ip: String,
    pub hostname: String,
    pub device_id: Option<String>,
    pub service_type: String,
    pub is_online: bool,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub is_sos: bool,
    #[serde(default)]
    pub user_name: Option<String>,
    #[serde(default)]
    pub status_note: Option<String>,
    #[serde(default)]
    pub last_seen: u64,
}

fn default_port() -> u16 {
    8080
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub sender: String,
    pub text: String,
    pub timestamp: u64,
}

pub struct AppState {
    pub info: Mutex<HostInfo>,
    pub address_book: Mutex<Vec<AddressBookItem>>,
    pub discovered: Mutex<HashMap<String, DiscoveredDevice>>,
    pub file_transfer: Arc<FileTransferManager>,
    pub audit_logger: Arc<crate::audit::AuditLogger>,
    pub policy: Mutex<crate::policy::SessionPolicy>,
    pub chat_messages: Mutex<Vec<ChatMessage>>,
    pub session_kill_tx: tokio::sync::broadcast::Sender<()>,
    pub chat_tx: tokio::sync::broadcast::Sender<ChatMessage>,
    pub active_session_count: std::sync::atomic::AtomicU32,
    pub admin_password: Mutex<Option<String>>,
}

pub type SharedState = Arc<AppState>;

impl AppState {
    fn load_saved_admin_password() -> Option<String> {
        let appdata = std::env::var("APPDATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("."));
        let key_file = appdata.join("OxiDesk").join("admin_secret.key");
        if let Ok(content) = std::fs::read_to_string(key_file) {
            let pwd = content.trim().to_string();
            if !pwd.is_empty() {
                return Some(pwd);
            }
        }
        None
    }

    pub fn save_admin_password(pwd: Option<&str>) {
        let appdata = std::env::var("APPDATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("."));
        let dir = appdata.join("OxiDesk");
        let _ = std::fs::create_dir_all(&dir);
        let key_file = dir.join("admin_secret.key");
        if let Some(p) = pwd {
            let _ = std::fs::write(key_file, p.trim());
        } else {
            let _ = std::fs::remove_file(key_file);
        }
    }

    pub fn new(device_id: String, pin: String, local_ip: String, port: u16, hostname: String) -> Self {
        let (session_kill_tx, _) = tokio::sync::broadcast::channel(16);
        let (chat_tx, _) = tokio::sync::broadcast::channel(64);
        let admin_pwd = Self::load_saved_admin_password();
        Self {
            info: Mutex::new(HostInfo {
                device_id,
                pin,
                local_ip,
                port,
                hostname,
                os: "Windows".to_string(),
            }),
            address_book: Mutex::new(Vec::new()),
            discovered: Mutex::new(HashMap::new()),
            file_transfer: Arc::new(FileTransferManager::new()),
            audit_logger: Arc::new(crate::audit::AuditLogger::new()),
            policy: Mutex::new(crate::policy::SessionPolicy::default()),
            chat_messages: Mutex::new(Vec::new()),
            session_kill_tx,
            chat_tx,
            active_session_count: std::sync::atomic::AtomicU32::new(0),
            admin_password: Mutex::new(admin_pwd),
        }
    }
}
