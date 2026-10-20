use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use rand::Rng;
use tokio::net::TcpListener;

use apex_connect::discovery::DiscoveryEngine;
use apex_connect::server::create_router;
use apex_connect::state::AppState;

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
    let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Windows-PC".to_string());
    let username = std::env::var("USERNAME").unwrap_or_else(|_| "Kullanıcı".to_string());
    let device_id = generate_apex_id();
    let pin = generate_pin();

    // Auto-detect available port (try 8080 to 8095)
    let mut selected_port = 8080;
    let mut bound_listener = None;

    for p in 8080..=8095 {
        let addr: SocketAddr = format!("0.0.0.0:{}", p).parse()?;
        match TcpListener::bind(addr).await {
            Ok(l) => {
                selected_port = p;
                bound_listener = Some(l);
                break;
            }
            Err(_) => continue,
        }
    }

    let listener = match bound_listener {
        Some(l) => l,
        None => {
            eprintln!("\n[HATA] Port bulunamadı! Çıkmak için Enter'a basın...");
            let mut s = String::new();
            let _ = std::io::stdin().read_line(&mut s);
            return Err("Ports occupied".into());
        }
    };

    let state = Arc::new(AppState::new(
        device_id.clone(),
        pin.clone(),
        local_ip.clone(),
        selected_port,
        hostname.clone(),
    ));

    // Start SOS UDP Broadcaster announcing assistance request to LAN IT admins
    DiscoveryEngine::start_sos_broadcaster(
        device_id.clone(),
        hostname.clone(),
        selected_port,
        username.clone(),
    );

    let formatted_id = format!(
        "{} {} {}",
        &device_id[0..3],
        &device_id[3..6],
        &device_id[6..9]
    );

    println!();
    println!("  ===============================================================");
    println!("  🚨  ApexConnect Hızlı Destek (QuickSupport Portable v1.0)");
    println!("  ===============================================================");
    println!("  👤  Kullanıcı / Cihaz  : {} ({})", hostname, username);
    println!("  🆔  Apex Kimliğiniz    : \x1b[1;36m{}\x1b[0m", formatted_id);
    println!("  🔑  Geçici Bağlantı PIN: \x1b[1;33m{}\x1b[0m", pin);
    println!("  🌐  Yerel Ağ Adresi    : {}:{}", local_ip, selected_port);
    println!("  ---------------------------------------------------------------");
    println!("  📡  Ağ Keşfi Durumu    : \x1b[1;32mAKTİF (Bilgi İşleme Anons Gönderiliyor)\x1b[0m");
    println!("  ===============================================================");
    println!("  [Bilgi] Bilgi İşlem personeli aynı ağdaysa bu cihazı doğrudan görebilir.");
    println!("  [Bilgi] Destek penceresi açılıyor. Çıkmak için pencereyi kapatın.\n");

    // Automatically open mini QuickSupport UI
    let local_url = format!("http://localhost:{}", selected_port);
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", &local_url])
            .spawn();
    });

    let app = create_router(state);
    axum::serve(listener, app).await?;

    Ok(())
}
