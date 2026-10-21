use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, State,
    },
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};

/// Peer information registered on the Rendezvous server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredPeer {
    pub device_id: String,
    pub hostname: String,
    pub local_ip: String,
    pub local_port: u16,
    pub peer_type: String, // "FullClient", "QuickSupport", "Controller"
    #[serde(skip)]
    pub public_addr: Option<SocketAddr>,
    pub registered_at: String,
}

/// Signaling messages exchanged over WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "data")]
pub enum SignalMessage {
    /// Peer registration
    Register {
        device_id: String,
        hostname: String,
        local_ip: String,
        local_port: u16,
        peer_type: String,
    },
    /// Periodic heartbeat from client
    Heartbeat {
        device_id: String,
    },
    /// Query target peer availability
    QueryPeer {
        target_id: String,
    },
    /// Server response for peer query
    QueryPeerResponse {
        target_id: String,
        online: bool,
        peer_info: Option<RegisteredPeer>,
    },
    /// Request P2P punch / relay handoff
    ConnectRequest {
        from_id: String,
        target_id: String,
        session_id: String,
        candidate_addr: String,
    },
    /// Target response to connection request
    ConnectResponse {
        from_id: String,
        target_id: String,
        session_id: String,
        accepted: bool,
        candidate_addr: String,
    },
    /// Relay data transmission when P2P NAT traversal fails
    RelayPayload {
        session_id: String,
        target_id: String,
        payload_base64: String,
    },
    /// General server notification / error
    Notification {
        level: String,
        message: String,
    },
}

struct ConnectedClient {
    device_id: String,
    tx: mpsc::UnboundedSender<Message>,
}

pub struct RendezvousState {
    peers: Arc<RwLock<HashMap<String, RegisteredPeer>>>,
    active_connections: Arc<RwLock<HashMap<String, ConnectedClient>>>,
    start_time: Instant,
}

impl Default for RendezvousState {
    fn default() -> Self {
        Self::new()
    }
}

impl RendezvousState {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            active_connections: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }
}

/// Build Axum router for the Rendezvous & Relay Server
pub fn create_rendezvous_router(state: Arc<RendezvousState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/status", get(status_handler))
        .route("/api/peers", get(peers_handler))
        .route("/ws/signaling", get(ws_signaling_handler))
        .with_state(state)
}

async fn health_handler() -> &'static str {
    "ApexConnect Rendezvous & Signaling Server - OK (v1.2.0)"
}

#[derive(Serialize)]
struct ServerStatusResponse {
    status: &'static str,
    version: &'static str,
    uptime_seconds: u64,
    registered_peers: usize,
    active_ws_connections: usize,
}

async fn status_handler(State(state): State<Arc<RendezvousState>>) -> Json<ServerStatusResponse> {
    let peers_count = state.peers.read().await.len();
    let conns_count = state.active_connections.read().await.len();

    Json(ServerStatusResponse {
        status: "online",
        version: "1.2.0",
        uptime_seconds: state.start_time.elapsed().as_secs(),
        registered_peers: peers_count,
        active_ws_connections: conns_count,
    })
}

async fn peers_handler(State(state): State<Arc<RendezvousState>>) -> Json<Vec<RegisteredPeer>> {
    let peers = state.peers.read().await;
    let list: Vec<RegisteredPeer> = peers.values().cloned().collect();
    Json(list)
}

async fn ws_signaling_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<RendezvousState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_signaling_socket(socket, addr, state))
}

async fn handle_signaling_socket(
    socket: WebSocket,
    addr: SocketAddr,
    state: Arc<RendezvousState>,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Spawn forwarding task: mpsc rx -> ws_sender
    let forward_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut current_device_id: Option<String> = None;

    while let Some(Ok(msg)) = ws_receiver.next().await {
        if let Message::Text(text) = msg {
            if let Ok(signal) = serde_json::from_str::<SignalMessage>(&text) {
                match signal {
                    SignalMessage::Register {
                        device_id,
                        hostname,
                        local_ip,
                        local_port,
                        peer_type,
                    } => {
                        let now_str = chrono::Utc::now().to_rfc3339();
                        let peer = RegisteredPeer {
                            device_id: device_id.clone(),
                            hostname,
                            local_ip,
                            local_port,
                            peer_type,
                            public_addr: Some(addr),
                            registered_at: now_str,
                        };

                        // Register peer
                        state.peers.write().await.insert(device_id.clone(), peer);
                        state.active_connections.write().await.insert(
                            device_id.clone(),
                            ConnectedClient {
                                device_id: device_id.clone(),
                                tx: tx.clone(),
                            },
                        );

                        current_device_id = Some(device_id.clone());

                        let ack = SignalMessage::Notification {
                            level: "info".into(),
                            message: format!("Device [{}] successfully registered with Rendezvous server", device_id),
                        };
                        if let Ok(ack_json) = serde_json::to_string(&ack) {
                            let _ = tx.send(Message::Text(ack_json));
                        }
                    }

                    SignalMessage::Heartbeat { device_id } => {
                        if let Some(peer) = state.peers.write().await.get_mut(&device_id) {
                            peer.registered_at = chrono::Utc::now().to_rfc3339();
                        }
                    }

                    SignalMessage::QueryPeer { target_id } => {
                        let peers = state.peers.read().await;
                        let found = peers.get(&target_id).cloned();
                        let response = SignalMessage::QueryPeerResponse {
                            target_id: target_id.clone(),
                            online: found.is_some(),
                            peer_info: found,
                        };
                        if let Ok(resp_json) = serde_json::to_string(&response) {
                            let _ = tx.send(Message::Text(resp_json));
                        }
                    }

                    SignalMessage::ConnectRequest {
                        from_id,
                        target_id,
                        session_id,
                        candidate_addr,
                    } => {
                        // Forward connect request to target if online
                        let conns = state.active_connections.read().await;
                        if let Some(target_client) = conns.get(&target_id) {
                            let fwd = SignalMessage::ConnectRequest {
                                from_id: from_id.clone(),
                                target_id: target_id.clone(),
                                session_id,
                                candidate_addr,
                            };
                            if let Ok(fwd_json) = serde_json::to_string(&fwd) {
                                let _ = target_client.tx.send(Message::Text(fwd_json));
                            }
                        } else {
                            // Target offline
                            let err_msg = SignalMessage::Notification {
                                level: "warning".into(),
                                message: format!("Target peer [{}] is currently offline", target_id),
                            };
                            if let Ok(err_json) = serde_json::to_string(&err_msg) {
                                let _ = tx.send(Message::Text(err_json));
                            }
                        }
                    }

                    SignalMessage::ConnectResponse {
                        from_id,
                        target_id,
                        session_id,
                        accepted,
                        candidate_addr,
                    } => {
                        // Forward target's response back to original requester (from_id)
                        let conns = state.active_connections.read().await;
                        if let Some(requester) = conns.get(&from_id) {
                            let fwd = SignalMessage::ConnectResponse {
                                from_id,
                                target_id,
                                session_id,
                                accepted,
                                candidate_addr,
                            };
                            if let Ok(fwd_json) = serde_json::to_string(&fwd) {
                                let _ = requester.tx.send(Message::Text(fwd_json));
                            }
                        }
                    }

                    SignalMessage::RelayPayload {
                        session_id,
                        target_id,
                        payload_base64,
                    } => {
                        // Forward relay packet directly to recipient
                        let conns = state.active_connections.read().await;
                        if let Some(target_client) = conns.get(&target_id) {
                            let fwd = SignalMessage::RelayPayload {
                                session_id,
                                target_id: target_id.clone(),
                                payload_base64,
                            };
                            if let Ok(fwd_json) = serde_json::to_string(&fwd) {
                                let _ = target_client.tx.send(Message::Text(fwd_json));
                            }
                        }
                    }

                    _ => {}
                }
            }
        } else if matches!(msg, Message::Close(_)) {
            break;
        }
    }

    // Cleanup when connection drops
    if let Some(device_id) = current_device_id {
        state.peers.write().await.remove(&device_id);
        state.active_connections.write().await.remove(&device_id);
        println!("[Rendezvous] Peer disconnected: {}", device_id);
    }

    forward_task.abort();
}
