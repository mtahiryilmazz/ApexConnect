use std::net::SocketAddr;
use std::sync::Arc;
use apex_connect::rendezvous::{create_rendezvous_router, RendezvousState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!("  ApexConnect Rendezvous & Signaling Server (v1.2.0)");
    println!("  WAN NAT Traversal, WebRTC P2P Signaling & Relay Bridge");
    println!("============================================================");

    let port: u16 = std::env::var("APEX_RENDEZVOUS_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8095);

    let bind_addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    let state = Arc::new(RendezvousState::new());
    let app = create_rendezvous_router(state);

    println!("[INFO] Rendezvous WebSocket: ws://0.0.0.0:{}/ws/signaling", port);
    println!("[INFO] REST Status API:      http://0.0.0.0:{}/api/status", port);
    println!("[INFO] REST Peers API:       http://0.0.0.0:{}/api/peers", port);
    println!("[INFO] Sunucu hazir. Istemci baglantilari dinleniyor...");

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
