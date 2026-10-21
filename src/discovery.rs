use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{TcpStream, UdpSocket};
use tokio::time::timeout;
use serde::{Deserialize, Serialize};
use crate::state::{DiscoveredDevice, SharedState};

pub const DISCOVERY_PORT: u16 = 52899;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiscoveryPacket {
    pub magic: String,
    pub hostname: String,
    pub device_id: String,
    pub port: u16,
    #[serde(default)]
    pub is_sos: bool,
    #[serde(default)]
    pub user_name: Option<String>,
    #[serde(default)]
    pub status_note: Option<String>,
}

pub struct DiscoveryEngine;

impl DiscoveryEngine {

    /// Broadcast general presence over UDP periodically
    pub fn start_broadcaster(device_id: String, hostname: String, port: u16) {
        Self::start_broadcaster_with_sos(device_id, hostname, port, false, None, None);
    }

    /// Broadcast presence as SOS / QuickSupport waiting for assistance
    pub fn start_sos_broadcaster(device_id: String, hostname: String, port: u16, user_name: String) {
        Self::start_broadcaster_with_sos(
            device_id,
            hostname,
            port,
            true,
            Some(user_name),
            Some("Yardım Bekliyor".to_string()),
        );
    }

    fn start_broadcaster_with_sos(
        device_id: String,
        hostname: String,
        port: u16,
        is_sos: bool,
        user_name: Option<String>,
        status_note: Option<String>,
    ) {
        tokio::spawn(async move {
            let socket = match UdpSocket::bind("0.0.0.0:0").await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("UDP broadcast bind error: {:?}", e);
                    return;
                }
            };
            let _ = socket.set_broadcast(true);

            let magic = if is_sos { "APEX_SOS_V1" } else { "APEX_DISCOVER_V1" };
            let packet = DiscoveryPacket {
                magic: magic.to_string(),
                hostname,
                device_id,
                port,
                is_sos,
                user_name,
                status_note,
            };

            let json = serde_json::to_string(&packet).unwrap_or_default();
            let target: SocketAddr = "255.255.255.255:52899".parse().unwrap();

            loop {
                let _ = socket.send_to(json.as_bytes(), target).await;
                // SOS beacons send more frequently (every 3 seconds) for instant discovery
                let interval = if is_sos { 3 } else { 5 };
                tokio::time::sleep(Duration::from_secs(interval)).await;
            }
        });
    }

    /// Listen for UDP broadcasts from other ApexConnect and QuickSupport nodes on LAN
    pub fn start_listener(state: SharedState) {
        tokio::spawn(async move {
            let socket = match UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT)).await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[Uyarı] UDP keşif dinleyicisi başlatılamadı (Port meşgul olabilir): {:?}", e);
                    return;
                }
            };

            let mut buf = [0u8; 1024];
            loop {
                if let Ok((len, src)) = socket.recv_from(&mut buf).await {
                    if let Ok(pkt) = serde_json::from_slice::<DiscoveryPacket>(&buf[..len]) {
                        if pkt.magic == "APEX_DISCOVER_V1" || pkt.magic == "APEX_SOS_V1" {
                            let ip = src.ip().to_string();
                            let service_type = if pkt.is_sos || pkt.magic == "APEX_SOS_V1" {
                                "🚨 SOS / Yardım Bekliyor".to_string()
                            } else {
                                "ApexConnect IT Düğümü".to_string()
                            };

                            let mut disc = state.discovered.lock().unwrap();
                            disc.insert(
                                format!("{}:{}", ip, pkt.port),
                                DiscoveredDevice {
                                    ip,
                                    hostname: pkt.hostname,
                                    device_id: Some(pkt.device_id),
                                    service_type,
                                    is_online: true,
                                    port: pkt.port,
                                    is_sos: pkt.is_sos || pkt.magic == "APEX_SOS_V1",
                                    user_name: pkt.user_name,
                                    status_note: pkt.status_note,
                                },
                            );
                        }
                    }
                }
            }
        });
    }

    /// Active Subnet Scanner for IT Department (Fast concurrent TCP port sweep)
    pub async fn scan_subnet(local_ip_str: &str, state: SharedState) -> Vec<DiscoveredDevice> {
        let mut results = Vec::new();

        // Include any UDP-discovered nodes first (especially active SOS beacons)
        {
            let disc = state.discovered.lock().unwrap();
            for (_, dev) in disc.iter() {
                results.push(dev.clone());
            }
        }

        // Parse local IP subnet (e.g. 192.168.1.X)
        let parts: Vec<&str> = local_ip_str.split('.').collect();
        if parts.len() != 4 {
            return results;
        }

        let base_ip = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
        let mut handles = Vec::new();

        // Concurrent scan of common /24 range
        for i in 1..=254 {
            let target_ip = format!("{}.{}", base_ip, i);
            if target_ip == local_ip_str {
                continue;
            }

            let handle = tokio::spawn(async move {
                let target_addr: SocketAddr = match format!("{}:8080", target_ip).parse() {
                    Ok(a) => a,
                    Err(_) => return None,
                };

                // Rapid 150ms probe
                if let Ok(Ok(_stream)) = timeout(Duration::from_millis(150), TcpStream::connect(target_addr)).await {
                    return Some(DiscoveredDevice {
                        ip: target_ip.clone(),
                        hostname: format!("Host-{}", target_ip),
                        device_id: None,
                        service_type: "ApexConnect Ajanı (HTTP/WS)".to_string(),
                        is_online: true,
                        port: 8080,
                        is_sos: false,
                        user_name: None,
                        status_note: None,
                    });
                }

                None
            });

            handles.push(handle);
        }

        for handle in handles {
            if let Ok(Some(dev)) = handle.await {
                if !results.iter().any(|d| d.ip == dev.ip) {
                    results.push(dev);
                }
            }
        }

        results
    }
}
