use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub name: String,
    pub size: u64,
    pub modified: String,
}

#[derive(Debug)]
struct OngoingUpload {
    pub file_path: PathBuf,
    pub expected_size: u64,
    pub received_bytes: u64,
}

pub struct FileTransferManager {
    storage_dir: PathBuf,
    uploads: Mutex<HashMap<String, OngoingUpload>>,
}

impl FileTransferManager {
    pub fn new() -> Self {
        // Safe directory: %USERPROFILE%\Downloads\ApexConnect
        let base_dir = std::env::var("USERPROFILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        let storage_dir = base_dir.join("Downloads").join("ApexConnect");
        let _ = fs::create_dir_all(&storage_dir);

        Self {
            storage_dir,
            uploads: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_storage_path(&self) -> PathBuf {
        self.storage_dir.clone()
    }

    /// Sanitize filename to prevent directory traversal
    fn sanitize_filename(filename: &str) -> String {
        let path = Path::new(filename);
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed_file")
            .to_string()
    }

    /// Begin a chunked file upload
    pub fn start_upload(&self, file_id: &str, original_name: &str, expected_size: u64) -> Result<(), String> {
        let safe_name = Self::sanitize_filename(original_name);
        let dest_path = self.storage_dir.join(&safe_name);

        // Open or create empty file
        let _ = File::create(&dest_path).map_err(|e| format!("Failed to create file: {}", e))?;

        let mut uploads = self.uploads.lock().unwrap();
        uploads.insert(
            file_id.to_string(),
            OngoingUpload {
                file_path: dest_path,
                expected_size,
                received_bytes: 0,
            },
        );

        Ok(())
    }

    /// Append a chunk of data to the ongoing upload
    pub fn write_chunk(&self, file_id: &str, offset: u64, data: &[u8]) -> Result<u64, String> {
        let mut uploads = self.uploads.lock().unwrap();
        let upload = uploads
            .get_mut(file_id)
            .ok_or_else(|| "Upload session not found".to_string())?;

        let mut file = OpenOptions::new()
            .write(true)
            .open(&upload.file_path)
            .map_err(|e| format!("Failed to open file for writing: {}", e))?;

        file.seek(SeekFrom::Start(offset))
            .map_err(|e| format!("Seek failed: {}", e))?;
        file.write_all(data)
            .map_err(|e| format!("Write failed: {}", e))?;

        upload.received_bytes += data.len() as u64;
        Ok(upload.received_bytes)
    }

    /// Finish upload and clean up session
    pub fn finish_upload(&self, file_id: &str) -> Result<String, String> {
        let mut uploads = self.uploads.lock().unwrap();
        let upload = uploads
            .remove(file_id)
            .ok_or_else(|| "Upload session not found".to_string())?;

        let path = upload.file_path.clone();
        let file_name = upload
            .file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();

        // Native Windows CF_HDROP clipboard integration:
        // Allows user to immediately Ctrl+V the transferred file anywhere in File Explorer!
        crate::input::InputSimulator::set_clipboard_files(&[path]);

        Ok(file_name)
    }

    /// List all files in the transfer directory
    pub fn list_files(&self) -> Vec<FileMetadata> {
        let mut result = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.storage_dir) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_file() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let size = meta.len();
                        let modified = meta
                            .modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| {
                                chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                                    .unwrap_or_default()
                            })
                            .unwrap_or_default();

                        result.push(FileMetadata {
                            name,
                            size,
                            modified,
                        });
                    }
                }
            }
        }
        result
    }

    /// Read file content safely for download
    pub fn read_file(&self, filename: &str) -> Result<Vec<u8>, String> {
        let safe_name = Self::sanitize_filename(filename);
        let path = self.storage_dir.join(safe_name);
        if !path.exists() {
            return Err("File not found".to_string());
        }

        let mut file = File::open(path).map_err(|e| format!("Cannot open file: {}", e))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| format!("Cannot read file: {}", e))?;

        Ok(buffer)
    }
}
