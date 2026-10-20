use std::fs;
use std::path::Path;
use crate::state::AddressBookItem;

const ADDRESS_BOOK_FILE: &str = "address_book.json";

pub struct AddressBookManager;

impl AddressBookManager {
    pub fn load() -> Vec<AddressBookItem> {
        if Path::new(ADDRESS_BOOK_FILE).exists() {
            if let Ok(content) = fs::read_to_string(ADDRESS_BOOK_FILE) {
                if let Ok(items) = serde_json::from_str::<Vec<AddressBookItem>>(&content) {
                    return items;
                }
            }
        }
        // Default seed items for IT department
        vec![
            AddressBookItem {
                id: "1".to_string(),
                name: "Muhasebe-Ana-PC".to_string(),
                group: "Muhasebe".to_string(),
                target: "192.168.1.50".to_string(),
                notes: "Logo / Zirve veritabanı".to_string(),
                saved_pin: "1234".to_string(),
                is_online: true,
                last_connected: Some("Bugün 14:30".to_string()),
                is_trusted_unattended: true,
                auth_token: None,
            },
            AddressBookItem {
                id: "2".to_string(),
                name: "Sistem-Yedek-Sunucu".to_string(),
                group: "Sistem Odası".to_string(),
                target: "192.168.1.10".to_string(),
                notes: "Hyper-V Host 01".to_string(),
                saved_pin: "".to_string(),
                is_online: true,
                last_connected: Some("Dün 18:20".to_string()),
                is_trusted_unattended: true,
                auth_token: None,
            },
            AddressBookItem {
                id: "3".to_string(),
                name: "Şube-Kasa-PC".to_string(),
                group: "Şube-1".to_string(),
                target: "492 881 204".to_string(),
                notes: "Barkod ve Yazıcı bağlı".to_string(),
                saved_pin: "".to_string(),
                is_online: false,
                last_connected: None,
                is_trusted_unattended: false,
                auth_token: None,
            },
        ]
    }

    pub fn save(items: &[AddressBookItem]) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(items)?;
        fs::write(ADDRESS_BOOK_FILE, json)?;
        Ok(())
    }
}
