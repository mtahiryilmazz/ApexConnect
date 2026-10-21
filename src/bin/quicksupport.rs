#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use rand::Rng;
use tokio::net::TcpListener;

use oxidesk::discovery::DiscoveryEngine;
use oxidesk::server::create_router;
use oxidesk::state::AppState;

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

fn generate_oxidesk_id() -> String {
    let mut rng = rand::thread_rng();
    let num: u32 = rng.gen_range(100_000_000..999_999_999);
    num.to_string()
}

fn generate_pin() -> String {
    let mut rng = rand::thread_rng();
    let num: u16 = rng.gen_range(1000..9999);
    num.to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let (state, _selected_port) = rt.block_on(async {
        let local_ip = get_local_ip();
        let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Windows-PC".to_string());
        let username = std::env::var("USERNAME").unwrap_or_else(|_| "Kullanıcı".to_string());
        let device_id = generate_oxidesk_id();
        let pin = generate_pin();

        // Auto-detect available port (try 8080 to 8095)
        let mut selected_port = 8080;
        let mut bound_listener = None;

        for p in 8080..=8095 {
            let addr: SocketAddr = format!("0.0.0.0:{}", p).parse().unwrap();
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
                eprintln!("\n[HATA] Port bulunamadı!");
                return Err::<_, Box<dyn std::error::Error>>("Ports occupied".into());
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
        println!("  🚨  OxiDesk Hızlı Destek (QuickSupport Portable v5.5.0 - Native Fluent UI)");
        println!("  ===============================================================");
        println!("  👤  Kullanıcı / Cihaz  : {} ({})", hostname, username);
        println!("  🆔  Oxi Kimliğiniz     : \x1b[1;36m{}\x1b[0m", formatted_id);
        println!("  🔑  Geçici Bağlantı PIN: \x1b[1;33m{}\x1b[0m", pin);
        println!("  🌐  Yerel Ağ Adresi    : {}:{}", local_ip, selected_port);
        println!("  ---------------------------------------------------------------");
        println!("  📡  Ağ Keşfi Durumu    : \x1b[1;32mAKTİF (Bilgi İşleme Anons Gönderiliyor)\x1b[0m");
        println!("  ===============================================================");
        println!("  [Bilgi] Bilgi İşlem personeli aynı ağdaysa bu cihazı doğrudan görebilir.");
        println!("  [Bilgi] Saf Win32 Hızlı Destek Penceresi açılıyor (Sıfır Edge Bağımlılığı)...\n");

        let app = create_router(state.clone());
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        Ok::<_, Box<dyn std::error::Error>>((state, selected_port))
    })?;

    // Run 100% pure native Win32 QuickSupport desktop window (Zero Edge / Zero WebView2 dependency)
    oxidesk::native_qs::NativeQuickSupportGui::run(state);

    oxidesk::force_terminate_process();
}
