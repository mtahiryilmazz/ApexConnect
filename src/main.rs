#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use rand::Rng;
use tokio::net::TcpListener;

use apex_connect::address_book::AddressBookManager;
use apex_connect::discovery::DiscoveryEngine;
use apex_connect::gui::GuiLauncher;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let (local_url, _selected_port) = rt.block_on(async {
        let local_ip = get_local_ip();
        let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Windows-PC".to_string());
        let device_id = generate_apex_id();
        let pin = generate_pin();

        // Auto-detect an available port (try 8080 to 8090)
        let mut selected_port = 8080;
        let mut bound_listener = None;

        for p in 8080..=8090 {
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
                eprintln!("\n[HATA] 8080-8090 aralığındaki tüm portlar meşgul!");
                return Err::<_, Box<dyn std::error::Error>>("Ports 8080-8090 all occupied".into());
            }
        };

        let state = Arc::new(AppState::new(
            device_id.clone(),
            pin.clone(),
            local_ip.clone(),
            selected_port,
            hostname.clone(),
        ));

        // Load saved address book
        let saved_items = AddressBookManager::load();
        *state.address_book.lock().unwrap() = saved_items;

        // Start LAN UDP Discovery
        DiscoveryEngine::start_broadcaster(device_id.clone(), hostname.clone(), selected_port);
        DiscoveryEngine::start_listener(state.clone());

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
        println!("  🖥️   Bilgisayar Adı     : {}", hostname);
        println!("  🆔  Cihaz Kimliği (ID)  : \x1b[1;36m{}\x1b[0m", formatted_id);
        println!("  🔑  Geçici PIN Kodu     : \x1b[1;33m{}\x1b[0m", pin);
        println!("  🌐  Yerel Ağ IP         : {}:{}", local_ip, selected_port);
        println!("  ---------------------------------------------------------------");
        println!("  📱  Mobil / Web Konsol  : \x1b[1;32mhttp://{}:{}\x1b[0m", local_ip, selected_port);
        println!("  💻  Yerel Erişim        : \x1b[1;34mhttp://localhost:{}\x1b[0m", selected_port);
        println!("  📡  Ağ Keşfi (UDP)      : Port {} üzerinden aktif", apex_connect::discovery::DISCOVERY_PORT);
        println!("  ===============================================================");
        println!("  [Bilgi] Windows 11 Fluent Masaüstü Yönetim Konsolu başlatılıyor...\n");

        let app = create_router(state);
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        let local_url = format!("http://localhost:{}", selected_port);
        Ok::<_, Box<dyn std::error::Error>>((local_url, selected_port))
    })?;

    // Run native embedded frameless WebView2 window on main UI thread
    GuiLauncher::run_native_window(&local_url, "ApexConnect", 1280, 820);

    Ok(())
}
