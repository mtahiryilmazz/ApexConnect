mod address_book;
mod capture;
mod discovery;
mod input;
mod server;
mod state;

use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use rand::Rng;
use tokio::net::TcpListener;

use crate::address_book::AddressBookManager;
use crate::discovery::DiscoveryEngine;
use crate::server::create_router;
use crate::state::AppState;

fn get_local_ip() -> String {
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}

fn generate_apex_id() -> String {
    let mut rng = rand::thread_rng();
    let num: u32 = rng.gen_range(100_000_000..999_999_999);
    num.to_string()
}

fn generate_pin() -> String {
    let mut rng = rand::thread_rng();
    let num: u16 = rng.gen_range(1000..9999);
    num.to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let local_ip = get_local_ip();
    let port = 8080;
    let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Windows-PC".to_string());
    let device_id = generate_apex_id();
    let pin = generate_pin();

    let state = Arc::new(AppState::new(
        device_id.clone(),
        pin.clone(),
        local_ip.clone(),
        port,
        hostname.clone(),
    ));

    // Load saved address book
    let saved_items = AddressBookManager::load();
    *state.address_book.lock().unwrap() = saved_items;

    // Start LAN UDP Discovery
    DiscoveryEngine::start_broadcaster(device_id.clone(), hostname.clone(), port);
    DiscoveryEngine::start_listener(state.clone());

    // Print IT Helpdesk Startup Banner
    let formatted_id = format!(
        "{} {} {}",
        &device_id[0..3],
        &device_id[3..6],
        &device_id[6..9]
    );

    println!();
    println!("  ===============================================================");
    println!("  🚀  ApexConnect - IT Uzak Masaüstü ve Cihaz Yönetim Ekosistemi");
    println!("  ===============================================================");
    println!("  🖥️  Bilgisayar Adı     : {}", hostname);
    println!("  🆔  Cihaz Kimliği (ID)  : \x1b[1;36m{}\x1b[0m", formatted_id);
    println!("  🔑  Geçici PIN Kodu     : \x1b[1;33m{}\x1b[0m", pin);
    println!("  🌐  Yerel Ağ IP         : {}:{}", local_ip, port);
    println!("  ---------------------------------------------------------------");
    println!("  📱  Mobil / Web Konsol  : \x1b[1;32mhttp://{}:{}\x1b[0m", local_ip, port);
    println!("  💻  Yerel Erişim        : \x1b[1;34mhttp://localhost:{}\x1b[0m", port);
    println!("  📡  Ağ Keşfi (UDP)      : Port {} üzerinden aktif", discovery::DISCOVERY_PORT);
    println!("  ===============================================================");
    println!("  [Bilgi] Son kullanıcılar veya IT personeli tarayıcıdan bağlanabilir.");
    println!("  [Bilgi] Çıkış yapmak için Ctrl+C tuşlarına basın.\n");

    let app = create_router(state);
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
