use std::env;
use std::sync::Arc;
use oxidesk::service::ServiceManager;
use oxidesk::server::create_router;
use oxidesk::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("status");

    println!("============================================================");
    println!("  OxiDesk Unattended Windows Service Manager (v2.0.0)");
    println!("  Winlogon Kilit Ekrani ve UAC Yetkili Arka Plan Hizmeti");
    println!("============================================================");

    match command {
        "install" => {
            let current_exe = env::current_exe()?;
            let exe_str = current_exe.to_string_lossy();
            match ServiceManager::install(&exe_str) {
                Ok(msg) => println!("[BASARILI] {}", msg),
                Err(err) => eprintln!("[HATA] {}", err),
            }
        }
        "uninstall" => {
            match ServiceManager::uninstall() {
                Ok(msg) => println!("[BASARILI] {}", msg),
                Err(err) => eprintln!("[HATA] {}", err),
            }
        }
        "start" => {
            match ServiceManager::start() {
                Ok(msg) => println!("[BASARILI] {}", msg),
                Err(err) => eprintln!("[HATA] {}", err),
            }
        }
        "stop" => {
            match ServiceManager::stop() {
                Ok(msg) => println!("[BASARILI] {}", msg),
                Err(err) => eprintln!("[HATA] {}", err),
            }
        }
        "status" => {
            let status = ServiceManager::status();
            println!("[DURUM] Hizmet Durumu: {}", status);
        }
        "run-service" => {
            println!("[INFO] Windows Hizmet kipinde baslatiliyor (NT AUTHORITY\\SYSTEM)...");
            let mut rng = rand::thread_rng();
            let device_id = rand::Rng::gen_range(&mut rng, 100_000_000..999_999_999).to_string();
            let pin = "123456".to_string();
            let port = 8080u16;
            let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "OXIDESK-HOST".to_string());
            let local_ip = "0.0.0.0".to_string();

            let state = Arc::new(AppState::new(
                device_id.clone(),
                pin,
                local_ip,
                port,
                hostname.clone(),
            ));

            let app = create_router(state);
            let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
            println!("[INFO] OxiDesk Arka Plan Hizmeti Port {} uzerinde aktif.", port);

            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, app.into_make_service()).await?;
        }
        _ => {
            println!("Kullanim:");
            println!("  oxidesk_service.exe install      - Windows Hizmeti olarak kaydet");
            println!("  oxidesk_service.exe uninstall    - Windows Hizmetini sistemden sil");
            println!("  oxidesk_service.exe start        - Hizmeti baslat");
            println!("  oxidesk_service.exe stop         - Hizmeti durdur");
            println!("  oxidesk_service.exe status       - Hizmet calisma durumunu sorgula");
            println!("  oxidesk_service.exe run-service  - Hizmet arka plan dongusunu calistir");
        }
    }

    Ok(())
}
