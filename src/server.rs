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
    routing::{delete, get, post},
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
        .route("/qs", get(qs_handler))
        .route("/static/*file", get(static_handler))
        .route("/api/info", get(get_info_handler))
        .route("/api/scan", get(get_scan_handler))
        .route("/api/address-book", get(get_address_book_handler).post(add_address_book_handler))
        .route("/api/address-book/:id", delete(delete_address_book_handler))
        .route("/api/files", get(get_files_handler))
        .route("/api/files/download/:filename", get(download_file_handler))
        .route("/api/files/upload", post(upload_file_handler))
        .route("/api/files/open-folder", get(open_folder_handler))
        .route("/api/session/status", get(get_session_status_handler))
        .route("/api/session/disconnect", post(disconnect_session_handler))
        .route("/api/admin-password", get(get_admin_password_status_handler).post(set_admin_password_handler))
        .route("/api/processes", get(get_processes_handler))
        .route("/api/processes/kill", post(kill_process_handler))
        .route("/api/rendezvous/status", get(get_rendezvous_status_handler).post(update_rendezvous_config_handler))
        .route("/api/resolve", get(resolve_target_handler))
        .route("/api/pin/refresh", get(refresh_pin_handler))
        .route("/api/audit", get(get_audit_handler))
        .route("/api/policy", get(get_policy_handler).post(update_policy_handler))
        .route("/api/chat", get(get_chat_handler).post(send_chat_handler))
        .route("/api/service/status", get(get_service_status_handler))
        .route("/api/window/minimize", post(window_minimize_handler))
        .route("/api/window/maximize", post(window_maximize_handler))
        .route("/api/window/close", post(window_close_handler))
        .route("/ws/client", get(ws_handler))
        .with_state(state)
}

#[derive(Deserialize)]
struct ResolveQuery {
    target: String,
}

#[derive(Serialize)]
struct ResolveResponse {
    found: bool,
    ip: String,
    port: u16,
    hostname: Option<String>,
}

async fn resolve_target_handler(
    State(state): State<SharedState>,
    Query(query): Query<ResolveQuery>,
) -> Json<ResolveResponse> {
    let clean = query.target.replace(" ", "");

    // 1. Check if it's already an explicit host:port or ip:port
    if clean.contains(':') {
        let parts: Vec<&str> = clean.split(':').collect();
        if parts.len() == 2 {
            if let Ok(p) = parts[1].parse::<u16>() {
                return Json(ResolveResponse {
                    found: true,
                    ip: parts[0].to_string(),
                    port: p,
                    hostname: None,
                });
            }
        }
    }

    // 2. Search in local UDP discovered devices
    {
        let disc = state.discovered.lock().unwrap();
        for (_, dev) in disc.iter() {
            if let Some(id) = &dev.device_id {
                if id.replace(" ", "") == clean {
                    return Json(ResolveResponse {
                        found: true,
                        ip: dev.ip.clone(),
                        port: dev.port,
                        hostname: Some(dev.hostname.clone()),
                    });
                }
            }
            if dev.ip == clean {
                return Json(ResolveResponse {
                    found: true,
                    ip: dev.ip.clone(),
                    port: dev.port,
                    hostname: Some(dev.hostname.clone()),
                });
            }
        }
    }

    // 3. Search in address book
    {
        let book = state.address_book.lock().unwrap();
        for item in book.iter() {
            let item_target = item.target.replace(" ", "");
            if item.id == clean || item_target == clean {
                if item_target.contains(':') {
                    let parts: Vec<&str> = item_target.split(':').collect();
                    if let Ok(p) = parts[1].parse::<u16>() {
                        return Json(ResolveResponse {
                            found: true,
                            ip: parts[0].to_string(),
                            port: p,
                            hostname: Some(item.name.clone()),
                        });
                    }
                }
                return Json(ResolveResponse {
                    found: true,
                    ip: item.target.clone(),
                    port: 8080,
                    hostname: Some(item.name.clone()),
                });
            }
        }
    }

    // 4. If target is a standard IP without port, default to 8080
    if clean.split('.').count() == 4 {
        return Json(ResolveResponse {
            found: true,
            ip: clean,
            port: 8080,
            hostname: None,
        });
    }

    // 5. Fallback for self/local testing
    let self_id = state.info.lock().unwrap().device_id.clone();
    if self_id.replace(" ", "") == clean {
        let (ip, port) = {
            let info = state.info.lock().unwrap();
            (info.local_ip.clone(), info.port)
        };
        return Json(ResolveResponse {
            found: true,
            ip,
            port,
            hostname: Some("Bu Cihaz".to_string()),
        });
    }

    Json(ResolveResponse {
        found: false,
        ip: String::new(),
        port: 0,
        hostname: None,
    })
}

#[derive(Serialize)]
struct RefreshPinResponse {
    new_pin: String,
}

async fn refresh_pin_handler(State(state): State<SharedState>) -> Json<RefreshPinResponse> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let num: u16 = rng.gen_range(1000..9999);
    let new_pin = num.to_string();

    {
        let mut info = state.info.lock().unwrap();
        info.pin = new_pin.clone();
    }

    state.audit_logger.record(
        "PIN_MANUAL_REFRESH",
        "USER",
        "127.0.0.1",
        "Tek kullanimlik gecici PIN kodu yenilendi.",
        true,
    );

    Json(RefreshPinResponse { new_pin })
}

#[derive(Serialize)]
struct RendezvousStatusDto {
    pub enabled: bool,
    pub server_url: String,
    pub status: String,
    pub nat_traversal: String,
}

async fn get_rendezvous_status_handler(State(_state): State<SharedState>) -> Json<RendezvousStatusDto> {
    let server_url = std::env::var("OXIDESK_RENDEZVOUS_SERVER")
        .or_else(|_| std::env::var("APEX_RENDEZVOUS_SERVER"))
        .unwrap_or_else(|_| "ws://127.0.0.1:8095/ws/signaling".to_string());
    Json(RendezvousStatusDto {
        enabled: true,
        server_url,
        status: "Online / Ready".to_string(),
        nat_traversal: "P2P STUN / Relay Fallback".to_string(),
    })
}

#[derive(Deserialize)]
struct UpdateRendezvousDto {
    pub server_url: String,
}

async fn update_rendezvous_config_handler(
    Json(payload): Json<UpdateRendezvousDto>,
) -> Json<serde_json::Value> {
    let url = payload.server_url.trim().to_string();
    std::env::set_var("OXIDESK_RENDEZVOUS_SERVER", &url);
    Json(serde_json::json!({ "status": "ok", "server_url": url }))
}

#[derive(Serialize)]
struct SessionStatusDto {
    is_active: bool,
    connected: bool,
    count: u32,
    active_sessions: u32,
}

async fn get_session_status_handler(State(state): State<SharedState>) -> Json<SessionStatusDto> {
    let count = state.active_session_count.load(std::sync::atomic::Ordering::Relaxed);
    Json(SessionStatusDto {
        is_active: count > 0,
        connected: count > 0,
        count,
        active_sessions: count,
    })
}

async fn disconnect_session_handler(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let _ = state.session_kill_tx.send(());
    // Auto-renew temporary PIN
    let new_pin_str = {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let new_pin: u16 = rng.gen_range(1000..9999);
        let mut info = state.info.lock().unwrap();
        info.pin = new_pin.to_string();
        info.pin.clone()
    };
    state.audit_logger.record(
        "HOST_MANUAL_DISCONNECT",
        "USER",
        "127.0.0.1",
        "Kullanici tarafindan uzaktan baglanti manuel olarak kesildi.",
        true,
    );
    Json(serde_json::json!({ "status": "disconnected", "new_pin": new_pin_str }))
}

#[derive(Deserialize)]
struct SetAdminPasswordDto {
    password: Option<String>,
}

#[derive(Serialize)]
struct AdminPasswordStatusDto {
    has_password: bool,
}

async fn get_admin_password_status_handler(State(state): State<SharedState>) -> Json<AdminPasswordStatusDto> {
    let has = state.admin_password.lock().unwrap().as_ref().map_or(false, |p| !p.is_empty());
    Json(AdminPasswordStatusDto { has_password: has })
}

async fn set_admin_password_handler(
    State(state): State<SharedState>,
    Json(payload): Json<SetAdminPasswordDto>,
) -> Json<serde_json::Value> {
    let mut lock = state.admin_password.lock().unwrap();
    let pwd = payload.password.filter(|s| !s.trim().is_empty());
    *lock = pwd.clone();
    crate::state::AppState::save_admin_password(pwd.as_deref());
    state.audit_logger.record(
        "ADMIN_PASSWORD_UPDATED",
        "ADMIN",
        "127.0.0.1",
        "Gozetimsiz erisim icin sabit yonetici parolasi guncellendi.",
        true,
    );
    Json(serde_json::json!({ "status": "ok", "has_password": pwd.is_some() }))
}

#[derive(Deserialize)]
struct UploadFileDto {
    name: String,
    data: String,
}

async fn upload_file_handler(
    State(state): State<SharedState>,
    Json(payload): Json<UploadFileDto>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&payload.data)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let saved = state.file_transfer.save_file(&payload.name, &bytes)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({ "status": "ok", "name": saved })))
}

#[derive(Serialize)]
pub struct ProcessDto {
    pub pid: u32,
    pub name: String,
    pub mem: String,
}

pub fn get_processes_list() -> Vec<ProcessDto> {
    let mut list = Vec::new();
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("tasklist.exe")
            .args(["/FO", "CSV", "/NH"])
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                let parts: Vec<&str> = line.split("\",\"").collect();
                if parts.len() >= 5 {
                    let name = parts[0].trim_matches('"').to_string();
                    let pid = parts[1].trim().parse::<u32>().unwrap_or(0);
                    let mem = parts[4].trim_matches('"').trim().to_string();
                    if pid > 0 {
                        list.push(ProcessDto { pid, name, mem });
                    }
                }
            }
        }
    }
    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    list
}

async fn get_processes_handler() -> Json<Vec<ProcessDto>> {
    Json(get_processes_list())
}

#[derive(Deserialize)]
struct KillProcessDto {
    pid: u32,
}

async fn kill_process_handler(Json(payload): Json<KillProcessDto>) -> StatusCode {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("taskkill.exe")
            .args(["/F", "/PID", &payload.pid.to_string()])
            .output();
    }
    StatusCode::OK
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

#[derive(Deserialize)]
struct SendChatDto {
    sender: String,
    text: String,
}

async fn get_chat_handler(State(state): State<SharedState>) -> Json<Vec<crate::state::ChatMessage>> {
    let list = state.chat_messages.lock().unwrap().clone();
    Json(list)
}

async fn send_chat_handler(
    State(state): State<SharedState>,
    Json(payload): Json<SendChatDto>,
) -> Json<crate::state::ChatMessage> {
    let msg = crate::state::ChatMessage {
        id: format!("{}", rand::random::<u32>()),
        sender: payload.sender,
        text: payload.text,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };
    state.chat_messages.lock().unwrap().push(msg.clone());
    let _ = state.chat_tx.send(msg.clone());
    Json(msg)
}

async fn index_handler() -> impl IntoResponse {
    match StaticAssets::get("index.html") {
        Some(content) => Html(content.data).into_response(),
        None => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}

async fn qs_handler() -> impl IntoResponse {
    match StaticAssets::get("qs.html") {
        Some(content) => Html(content.data).into_response(),
        None => (StatusCode::NOT_FOUND, "qs.html not found").into_response(),
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

    // If not trusted unattended device, verify PIN or Admin Password
    if !is_trusted_device {
        if let Some(pin) = query.pin {
            let is_pin_match = !pin.is_empty() && pin == host_pin;
            let is_admin_match = {
                let admin_pwd = state.admin_password.lock().unwrap();
                if let Some(ref pwd) = *admin_pwd {
                    !pin.is_empty() && &pin == pwd
                } else {
                    false
                }
            };

            if !is_pin_match && !is_admin_match {
                return (StatusCode::UNAUTHORIZED, "Gecersiz PIN veya Yonetici Parolasi").into_response();
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
    #[serde(rename = "chat")]
    Chat { sender: String, text: String, timestamp: u64 },
    #[serde(rename = "get_tasks")]
    GetTasks,
    #[serde(rename = "kill_task")]
    KillTask { pid: u32 },
}

async fn handle_remote_socket(socket: WebSocket, state: SharedState) {
    state.active_session_count.fetch_add(1, Ordering::SeqCst);
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

    // Chat broadcast listener from HTTP / QS host
    let mut chat_rx = state.chat_tx.subscribe();
    let tx_chat = tx.clone();
    let chat_forward_task = tokio::spawn(async move {
        while let Ok(msg) = chat_rx.recv().await {
            let json = serde_json::json!({
                "type": "chat",
                "sender": msg.sender,
                "text": msg.text,
                "timestamp": msg.timestamp,
            })
            .to_string();
            if tx_chat.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    // Host session kill listener (disconnect button clicked on QS)
    let mut kill_rx = state.session_kill_tx.subscribe();
    let kill_wait_task = tokio::spawn(async move {
        let _ = kill_rx.recv().await;
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
        let mut force_next = true;
        loop {
            let q = quality_worker.load(Ordering::Relaxed);
            let frame_opt = {
                let cap = capture_worker.lock().unwrap();
                cap.capture_frame_adaptive(q, force_next)
            };
            force_next = false;

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
                    // Do not kill the stream task permanently on transient desktop switch or lock!
                    sleep(Duration::from_millis(100)).await;
                    force_next = true;
                    continue;
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
                            ClientEvent::Chat { sender, text, timestamp } => {
                                let chat_obj = crate::state::ChatMessage {
                                    id: format!("{}", rand::random::<u32>()),
                                    sender: sender.clone(),
                                    text: text.clone(),
                                    timestamp,
                                };
                                state_ctrl.chat_messages.lock().unwrap().push(chat_obj.clone());
                                let _ = state_ctrl.chat_tx.send(chat_obj);
                            }
                            ClientEvent::GetTasks => {
                                let procs = get_processes_list();
                                let msg = serde_json::json!({
                                    "type": "tasks_list",
                                    "tasks": procs
                                })
                                .to_string();
                                let _ = tx_ctrl.send(Message::Text(msg)).await;
                            }
                            ClientEvent::KillTask { pid } => {
                                #[cfg(target_os = "windows")]
                                {
                                    let _ = std::process::Command::new("taskkill.exe")
                                        .args(["/F", "/PID", &pid.to_string()])
                                        .output();
                                }
                                let msg = serde_json::json!({
                                    "type": "task_killed",
                                    "pid": pid
                                })
                                .to_string();
                                let _ = tx_ctrl.send(Message::Text(msg)).await;
                            }
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    let (send_abort, stream_abort, input_abort, chat_abort) = (
        send_task.abort_handle(),
        stream_task.abort_handle(),
        input_task.abort_handle(),
        chat_forward_task.abort_handle(),
    );

    // Wait until stream terminates, client disconnects, or host clicks disconnect
    tokio::select! {
        _ = stream_task => {},
        _ = input_task => {},
        _ = send_task => {},
        _ = chat_forward_task => {},
        _ = kill_wait_task => {
            eprintln!("Oturum ana bilgisayar tarafindan sonlandirildi (Disconnect).");
        },
    };

    send_abort.abort();
    stream_abort.abort();
    input_abort.abort();
    chat_abort.abort();

    // Zero-Trust input release: ensure no keys or mouse buttons remain pressed after session disconnect
    let input_sim = InputSimulator::new();
    input_sim.release_all_keys();

    // Decrement session count and auto-regenerate random one-time PIN upon all sessions disconnect
    let remaining = state.active_session_count.fetch_sub(1, Ordering::SeqCst);
    if remaining <= 1 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let new_pin: u16 = rng.gen_range(1000..9999);
        let mut info = state.info.lock().unwrap();
        info.pin = new_pin.to_string();

        state.audit_logger.record(
            "SESSION_DISCONNECTED_PIN_RENEW",
            "SECURITY",
            "127.0.0.1",
            "Uzak oturum sonlandi; guvenlik icin tek kullanimlik PIN otomatik olarak sifirlandi ve yeni PIN uretildi.",
            true,
        );
    }
}

async fn window_close_handler() -> Json<serde_json::Value> {
    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        crate::force_terminate_process();
    });
    Json(serde_json::json!({ "status": "ok" }))
}

async fn window_minimize_handler() -> Json<serde_json::Value> {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, ShowWindow, SW_MINIMIZE};
        let hwnd = GetForegroundWindow();
        if hwnd.0 != 0 as _ {
            let _ = ShowWindow(hwnd, SW_MINIMIZE);
        }
    }
    Json(serde_json::json!({ "status": "ok" }))
}

async fn window_maximize_handler() -> Json<serde_json::Value> {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, IsZoomed, ShowWindow, SW_MAXIMIZE, SW_RESTORE};
        let hwnd = GetForegroundWindow();
        if hwnd.0 != 0 as _ {
            if IsZoomed(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            } else {
                let _ = ShowWindow(hwnd, SW_MAXIMIZE);
            }
        }
    }
    Json(serde_json::json!({ "status": "ok" }))
}
