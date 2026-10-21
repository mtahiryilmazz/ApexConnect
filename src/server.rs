use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{delete, get},
    Json, Router,
};
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::address_book::AddressBookManager;
use crate::capture::{CaptureMode, ScreenCapture};
use crate::discovery::DiscoveryEngine;
use crate::file_transfer::FileMetadata;
use crate::input::InputSimulator;
use crate::state::{AddressBookItem, DiscoveredDevice, HostInfo, SharedState};

#[derive(RustEmbed)]
#[folder = "static/"]
struct StaticAssets;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct WsAuthQuery {
    pub target: Option<String>,
    pub pin: Option<String>,
}

pub fn create_router(state: SharedState) -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/static/*file", get(static_handler))
        .route("/api/info", get(get_info_handler))
        .route("/api/scan", get(get_scan_handler))
        .route("/api/address-book", get(get_address_book_handler).post(add_address_book_handler))
        .route("/api/address-book/:id", delete(delete_address_book_handler))
        .route("/api/files", get(get_files_handler))
        .route("/api/files/download/:filename", get(download_file_handler))
        .route("/api/files/open-folder", get(open_folder_handler))
        .route("/api/rendezvous/status", get(get_rendezvous_status_handler))
        .route("/api/audit", get(get_audit_handler))
        .route("/api/policy", get(get_policy_handler).post(update_policy_handler))
        .route("/api/service/status", get(get_service_status_handler))
        .route("/ws/client", get(ws_handler))
        .with_state(state)
}

#[derive(Serialize)]
struct RendezvousStatusDto {
    pub enabled: bool,
    pub server_url: String,
    pub status: String,
    pub nat_traversal: String,
}

async fn get_rendezvous_status_handler(State(_state): State<SharedState>) -> Json<RendezvousStatusDto> {
    let server_url = std::env::var("APEX_RENDEZVOUS_SERVER")
        .unwrap_or_else(|_| "ws://127.0.0.1:8095/ws/signaling".to_string());
    Json(RendezvousStatusDto {
        enabled: true,
        server_url,
        status: "Online / Ready".to_string(),
        nat_traversal: "P2P STUN / Relay Fallback".to_string(),
    })
}

async fn get_audit_handler(State(state): State<SharedState>) -> Json<Vec<crate::audit::AuditEvent>> {
    let events = state.audit_logger.get_recent_events();
    Json(events)
}

async fn get_policy_handler(State(state): State<SharedState>) -> Json<crate::policy::SessionPolicy> {
    let policy = state.policy.lock().unwrap().clone();
    Json(policy)
}

async fn update_policy_handler(
    State(state): State<SharedState>,
    Json(new_policy): Json<crate::policy::SessionPolicy>,
) -> Json<crate::policy::SessionPolicy> {
    let mut lock = state.policy.lock().unwrap();
    *lock = new_policy.clone();
    state.audit_logger.record(
        "POLICY_UPDATE",
        "ADMIN",
        "127.0.0.1",
        &format!("Oturum politikasi guncellendi: {:?}", new_policy.role),
        true,
    );
    Json(new_policy)
}

#[derive(Serialize)]
struct ServiceStatusDto {
    pub service_name: &'static str,
    pub display_name: &'static str,
    pub status: String,
}

async fn get_service_status_handler() -> Json<ServiceStatusDto> {
    let status = crate::service::ServiceManager::status();
    Json(ServiceStatusDto {
        service_name: crate::service::ServiceManager::SERVICE_NAME,
        display_name: crate::service::ServiceManager::DISPLAY_NAME,
        status,
    })
}

async fn index_handler() -> impl IntoResponse {
    match StaticAssets::get("index.html") {
        Some(content) => Html(content.data).into_response(),
        None => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}

async fn static_handler(Path(file): Path<String>) -> impl IntoResponse {
    match StaticAssets::get(&file) {
        Some(content) => {
            let mime = if file.ends_with(".css") {
                "text/css"
            } else if file.ends_with(".js") {
                "application/javascript"
            } else if file.ends_with(".html") {
                "text/html; charset=utf-8"
            } else {
                "application/octet-stream"
            };

            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, mime.parse().unwrap());
            (headers, content.data).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Static file not found").into_response(),
    }
}

async fn get_info_handler(State(state): State<SharedState>) -> Json<HostInfo> {
    let info = state.info.lock().unwrap().clone();
    Json(info)
}

#[derive(serde::Serialize)]
struct ScanResponse {
    devices: Vec<DiscoveredDevice>,
}

async fn get_scan_handler(State(state): State<SharedState>) -> Json<ScanResponse> {
    let local_ip = state.info.lock().unwrap().local_ip.clone();
    let devices = DiscoveryEngine::scan_subnet(&local_ip, state.clone()).await;
    Json(ScanResponse { devices })
}

async fn get_address_book_handler(State(state): State<SharedState>) -> Json<Vec<AddressBookItem>> {
    let items = state.address_book.lock().unwrap().clone();
    Json(items)
}

async fn add_address_book_handler(
    State(state): State<SharedState>,
    Json(new_item): Json<AddressBookItem>,
) -> StatusCode {
    let mut items = state.address_book.lock().unwrap();
    items.retain(|i| i.id != new_item.id);
    items.push(new_item);
    let _ = AddressBookManager::save(&items);
    StatusCode::CREATED
}

async fn delete_address_book_handler(
    Path(id): Path<String>,
    State(state): State<SharedState>,
) -> StatusCode {
    let mut items = state.address_book.lock().unwrap();
    items.retain(|i| i.id != id);
    let _ = AddressBookManager::save(&items);
    StatusCode::OK
}

// File Transfer REST Handlers
async fn get_files_handler(State(state): State<SharedState>) -> Json<Vec<FileMetadata>> {
    Json(state.file_transfer.list_files())
}

async fn download_file_handler(
    Path(filename): Path<String>,
    State(state): State<SharedState>,
) -> Response {
    match state.file_transfer.read_file(&filename) {
        Ok(data) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename).parse().unwrap(),
            );
            headers.insert(header::CONTENT_TYPE, "application/octet-stream".parse().unwrap());
            (headers, data).into_response()
        }
        Err(e) => (StatusCode::NOT_FOUND, e).into_response(),
    }
}

async fn open_folder_handler(State(state): State<SharedState>) -> StatusCode {
    let path = state.file_transfer.get_storage_path();
    let _ = std::process::Command::new("explorer")
        .arg(path)
        .spawn();
    StatusCode::OK
}

// WebSocket Remote Desktop Gateway
async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsAuthQuery>,
    State(state): State<SharedState>,
) -> Response {
    let host_pin = state.info.lock().unwrap().pin.clone();

    // Check if target or connecting client is a trusted unattended device in address book
    let is_trusted_device = if let Some(target) = &query.target {
        let clean_target = target.replace(" ", "");
        let book = state.address_book.lock().unwrap();
        book.iter().any(|item| {
            item.is_trusted_unattended && (
                item.target.replace(" ", "") == clean_target 
                || item.id == clean_target 
                || item.saved_pin == query.pin.clone().unwrap_or_default()
            )
        })
    } else {
        false
    };

    // If not trusted unattended device, verify PIN if provided
    if !is_trusted_device {
        if let Some(pin) = query.pin {
            if !pin.is_empty() && pin != host_pin {
                return (StatusCode::UNAUTHORIZED, "Geçersiz PIN").into_response();
            }
        }
    }

    ws.on_upgrade(move |socket| handle_remote_socket(socket, state))
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum ClientEvent {
    #[serde(rename = "mouse_move")]
    MouseMove { x: f64, y: f64 },
    #[serde(rename = "mouse_down")]
    MouseDown { button: u8 },
    #[serde(rename = "mouse_up")]
    MouseUp { button: u8 },
    #[serde(rename = "mouse_click")]
    MouseClick { button: u8 },
    #[serde(rename = "mouse_wheel")]
    MouseWheel { delta: i32 },
    #[serde(rename = "type_text")]
    TypeText { text: String },
    #[serde(rename = "key_down")]
    KeyDown { key: String, code: String },
    #[serde(rename = "key_up")]
    KeyUp { key: String, code: String },
    #[serde(rename = "hotkey")]
    Hotkey { action: String },
    #[serde(rename = "ping")]
    Ping { timestamp: u64 },
    #[serde(rename = "clipboard")]
    Clipboard { text: String },
    #[serde(rename = "set_quality")]
    SetQuality { quality: u8 },
    #[serde(rename = "set_monitor")]
    SetMonitor { monitor: String },
    #[serde(rename = "file_start")]
    FileStart { id: String, name: String, size: u64 },
    #[serde(rename = "file_chunk")]
    FileChunk { id: String, offset: u64, data: String },
    #[serde(rename = "file_end")]
    FileEnd { id: String },
}

async fn handle_remote_socket(socket: WebSocket, state: SharedState) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // MPSC channel to funnel outgoing WebSocket messages from multiple tasks
    let (tx, mut rx) = mpsc::channel::<Message>(128);

    // Dedicated sender task
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let capture = Arc::new(Mutex::new(ScreenCapture::new()));
    let input = Arc::new(InputSimulator::new());
    let quality = Arc::new(AtomicU8::new(65));

    // Send initial screen resolution to client
    let size_msg = {
        let cap = capture.lock().unwrap();
        serde_json::json!({
            "type": "screen_size",
            "width": cap.width,
            "height": cap.height
        })
        .to_string()
    };
    let _ = tx.send(Message::Text(size_msg)).await;

    // Task 1: Screen Capture & Adaptive Streaming Loop
    let capture_worker = capture.clone();
    let quality_worker = quality.clone();
    let tx_stream = tx.clone();

    let stream_task = tokio::spawn(async move {
        loop {
            let q = quality_worker.load(Ordering::Relaxed);
            let frame_opt = {
                let cap = capture_worker.lock().unwrap();
                cap.capture_frame_adaptive(q, false)
            };

            match frame_opt {
                Ok(Some(jpeg_bytes)) => {
                    if tx_stream.send(Message::Binary(jpeg_bytes)).await.is_err() {
                        break;
                    }
                }
                Ok(None) => {
                    // Frame unchanged (dirty rect check passed) - no data to transmit!
                }
                Err(e) => {
                    eprintln!("Capture error: {}", e);
                    break;
                }
            }

            // ~33 FPS interval
            sleep(Duration::from_millis(30)).await;
        }
    });

    // Task 2: Client Input & Control Messages
    let input_worker = input.clone();
    let capture_ctrl = capture.clone();
    let quality_ctrl = quality.clone();
    let tx_ctrl = tx.clone();
    let state_ctrl = state.clone();

    let input_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(event) = serde_json::from_str::<ClientEvent>(&text) {
                        match event {
                            ClientEvent::MouseMove { x, y } => {
                                input_worker.mouse_move(x, y);
                            }
                            ClientEvent::MouseDown { button } => {
                                input_worker.mouse_down(button);
                            }
                            ClientEvent::MouseUp { button } => {
                                input_worker.mouse_up(button);
                            }
                            ClientEvent::MouseClick { button } => {
                                input_worker.mouse_click(button);
                            }
                            ClientEvent::MouseWheel { delta } => {
                                input_worker.mouse_wheel(delta);
                            }
                            ClientEvent::TypeText { text } => {
                                input_worker.type_text(&text);
                            }
                            ClientEvent::KeyDown { key, code } => {
                                input_worker.key_down(&key, &code);
                            }
                            ClientEvent::KeyUp { key, code } => {
                                input_worker.key_up(&key, &code);
                            }
                            ClientEvent::Hotkey { action } => {
                                if action == "ctrl_alt_del" {
                                    input_worker.send_ctrl_alt_del();
                                }
                            }
                            ClientEvent::Ping { timestamp } => {
                                let pong = serde_json::json!({
                                    "type": "pong",
                                    "timestamp": timestamp
                                })
                                .to_string();
                                let _ = tx_ctrl.send(Message::Text(pong)).await;
                            }
                            ClientEvent::Clipboard { text } => {
                                // Paste into local Windows clipboard via clip
                                if let Ok(mut child) = std::process::Command::new("clip")
                                    .stdin(std::process::Stdio::piped())
                                    .spawn()
                                {
                                    if let Some(mut stdin) = child.stdin.take() {
                                        use std::io::Write;
                                        let _ = stdin.write_all(text.as_bytes());
                                    }
                                }
                            }
                            ClientEvent::SetQuality { quality: new_q } => {
                                quality_ctrl.store(new_q.clamp(10, 100), Ordering::Relaxed);
                            }
                            ClientEvent::SetMonitor { monitor } => {
                                let mode = if monitor == "virtual" {
                                    CaptureMode::VirtualDesktop
                                } else {
                                    CaptureMode::PrimaryMonitor
                                };
                                let size_msg = {
                                    let mut cap = capture_ctrl.lock().unwrap();
                                    *cap = ScreenCapture::with_mode(mode);
                                    serde_json::json!({
                                        "type": "screen_size",
                                        "width": cap.width,
                                        "height": cap.height
                                    })
                                    .to_string()
                                };
                                let _ = tx_ctrl.send(Message::Text(size_msg)).await;
                            }
                            ClientEvent::FileStart { id, name, size } => {
                                let _ = state_ctrl.file_transfer.start_upload(&id, &name, size);
                                let ack = serde_json::json!({
                                    "type": "file_ack",
                                    "id": id,
                                    "status": "ready"
                                })
                                .to_string();
                                let _ = tx_ctrl.send(Message::Text(ack)).await;
                            }
                            ClientEvent::FileChunk { id, offset, data } => {
                                if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(&data) {
                                    if let Ok(received) = state_ctrl.file_transfer.write_chunk(&id, offset, &bytes) {
                                        let ack = serde_json::json!({
                                            "type": "file_progress",
                                            "id": id,
                                            "received": received
                                        })
                                        .to_string();
                                        let _ = tx_ctrl.send(Message::Text(ack)).await;
                                    }
                                }
                            }
                            ClientEvent::FileEnd { id } => {
                                if let Ok(filename) = state_ctrl.file_transfer.finish_upload(&id) {
                                    let ack = serde_json::json!({
                                        "type": "file_complete",
                                        "id": id,
                                        "name": filename
                                    })
                                    .to_string();
                                    let _ = tx_ctrl.send(Message::Text(ack)).await;
                                }
                            }
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait until stream terminates or client disconnects
    tokio::select! {
        _ = stream_task => {},
        _ = input_task => {},
        _ = send_task => {},
    };
}
