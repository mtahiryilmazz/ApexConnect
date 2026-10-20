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
    #[serde(default)]
    pub is_sos: bool,
    #[serde(default)]
    pub user_name: Option<String>,
    #[serde(default)]
    pub status_note: Option<String>,
}

pub struct AppState {
    pub info: Mutex<HostInfo>,
    pub address_book: Mutex<Vec<AddressBookItem>>,
    pub discovered: Mutex<HashMap<String, DiscoveredDevice>>,
    pub file_transfer: Arc<FileTransferManager>,
}

pub type SharedState = Arc<AppState>;

impl AppState {
    pub fn new(device_id: String, pin: String, local_ip: String, port: u16, hostname: String) -> Self {
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
        }
    }
}
