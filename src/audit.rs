use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: String,
    pub event_type: String, // "SESSION_START", "SESSION_END", "AUTH_FAILURE", "FILE_TRANSFER", "UAC_ELEVATION"
    pub actor_id: String,
    pub actor_ip: String,
    pub details: String,
    pub success: bool,
}

pub struct AuditLogger {
    log_file_path: PathBuf,
    recent_events: Arc<Mutex<Vec<AuditEvent>>>,
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditLogger {
    pub fn new() -> Self {
        // Choose ProgramData or local directory
        let log_dir = std::env::var("PROGRAMDATA")
            .map(|p| PathBuf::from(p).join("OxiDesk"))
            .unwrap_or_else(|_| PathBuf::from("logs"));

        let _ = create_dir_all(&log_dir);
        let log_file_path = log_dir.join("audit.log");

        Self {
            log_file_path,
            recent_events: Arc::new(Mutex::new(Vec::with_capacity(100))),
        }
    }

    pub fn record(
        &self,
        event_type: &str,
        actor_id: &str,
        actor_ip: &str,
        details: &str,
        success: bool,
    ) {
        let event = AuditEvent {
            id: format!("audit-{}", rand::random::<u32>()),
            timestamp: Utc::now().to_rfc3339(),
            event_type: event_type.to_string(),
            actor_id: actor_id.to_string(),
            actor_ip: actor_ip.to_string(),
            details: details.to_string(),
            success,
        };

        // 1. Keep in memory ring buffer
        if let Ok(mut lock) = self.recent_events.lock() {
            if lock.len() >= 100 {
                lock.remove(0);
            }
            lock.push(event.clone());
        }

        // 2. Persist to file
        if let Ok(json_line) = serde_json::to_string(&event) {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.log_file_path)
            {
                let _ = writeln!(file, "{}", json_line);
            }
        }
    }

    pub fn get_recent_events(&self) -> Vec<AuditEvent> {
        self.recent_events.lock().unwrap().clone()
    }
}
