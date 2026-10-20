use std::sync::Arc;
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
use futures_util::{SinkExt, StreamExt};
use rust_embed::RustEmbed;
use serde::Deserialize;
use tokio::time::sleep;

use crate::address_book::AddressBookManager;
use crate::capture::ScreenCapture;
use crate::discovery::DiscoveryEngine;
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
        .route("/ws/client", get(ws_handler))
        .with_state(state)
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

// WebSocket Remote Desktop Gateway
async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsAuthQuery>,
    State(state): State<SharedState>,
) -> Response {
    let host_pin = state.info.lock().unwrap().pin.clone();

    // Verify PIN if provided (or if connecting to self)
    if let Some(pin) = query.pin {
        if !pin.is_empty() && pin != host_pin {
            return (StatusCode::UNAUTHORIZED, "Invalid PIN").into_response();
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
    #[serde(rename = "hotkey")]
    Hotkey { action: String },
}

async fn handle_remote_socket(socket: WebSocket, _state: SharedState) {
    let (mut sender, mut receiver) = socket.split();

    let capture = Arc::new(ScreenCapture::new());
    let input = Arc::new(InputSimulator::new());

    // Send screen resolution to client
    let size_msg = serde_json::json!({
        "type": "screen_size",
        "width": capture.width,
        "height": capture.height
    })
    .to_string();

    let _ = sender.send(Message::Text(size_msg)).await;

    // Task 1: Screen Capture & Video Streaming Loop (25-30 FPS)
    let capture_worker = capture.clone();
    let stream_task = tokio::spawn(async move {
        loop {
            match capture_worker.capture_frame(65) {
                Ok(jpeg_bytes) => {
                    if sender.send(Message::Binary(jpeg_bytes)).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Capture error: {}", e);
                    break;
                }
            }
            // ~30 FPS frame rate
            sleep(Duration::from_millis(33)).await;
        }
    });

    // Task 2: Client Input Handling (Mouse/Keyboard events)
    let input_worker = input.clone();
    let input_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
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
                            ClientEvent::Hotkey { action } => {
                                if action == "ctrl_alt_del" {
                                    input_worker.send_ctrl_alt_del();
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

    // Wait until either stream ends or client disconnects
    tokio::select! {
        _ = stream_task => {},
        _ = input_task => {},
    };
}
