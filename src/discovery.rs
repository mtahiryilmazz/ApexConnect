use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{TcpStream, UdpSocket};
use tokio::time::timeout;
use serde::{Deserialize, Serialize};
use crate::state::{DiscoveredDevice, SharedState};

pub const DISCOVERY_PORT: u16 = 52899;

#[derive(Serialize, Deserialize, Debug)]
pub struct DiscoveryPacket {
    pub magic: String,
    pub hostname: String,
    pub device_id: String,
    pub port: u16,
}

pub struct DiscoveryEngine;

impl DiscoveryEngine {

    /// Broadcast presence over UDP periodically
    pub fn start_broadcaster(device_id: String, hostname: String, port: u16) {
        tokio::spawn(async move {
            let socket = match UdpSocket::bind("0.0.0.0:0").await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("UDP broadcast bind error: {:?}", e);
                    return;
                }
            };
            let _ = socket.set_broadcast(true);

            let packet = DiscoveryPacket {
                magic: "APEX_DISCOVER_V1".to_string(),
                hostname,
                device_id,
                port,
            };

            let json = serde_json::to_string(&packet).unwrap_or_default();
            let target: SocketAddr = "255.255.255.255:52899".parse().unwrap();

            loop {
                let _ = socket.send_to(json.as_bytes(), target).await;
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });
    }

    /// Listen for UDP broadcasts from other ApexConnect nodes on LAN
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
                        if pkt.magic == "APEX_DISCOVER_V1" {
                            let ip = src.ip().to_string();
                            let mut disc = state.discovered.lock().unwrap();
                            disc.insert(
                                ip.clone(),
                                DiscoveredDevice {
                                    ip,
                                    hostname: pkt.hostname,
                                    device_id: Some(pkt.device_id),
                                    service_type: "ApexConnect Node".to_string(),
                                    is_online: true,
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

        // Include any UDP-discovered nodes first
        {
            let disc = state.discovered.lock().unwrap();
            for (_, dev) in disc.iter() {
                results.push(dev.clone());
            }
        }

        // Parse base subnet (e.g. 192.168.1.x)
        let ip_parts: Vec<&str> = local_ip_str.split('.').collect();
        if ip_parts.len() != 4 {
            return results;
        }

        let base_subnet = format!("{}.{}.{}", ip_parts[0], ip_parts[1], ip_parts[2]);
        let mut handles = Vec::new();

        // Common IT ports: 8080 (Apex), 3389 (RDP), 445 (SMB Windows), 80 (HTTP)
        let probe_ports = [8080, 3389, 445, 80];

        // Scan 1..254 concurrently in batches
        for i in 1..=254 {
            let target_ip = format!("{}.{}", base_subnet, i);
            if target_ip == local_ip_str {
                continue;
            }

            let ports = probe_ports;
            handles.push(tokio::spawn(async move {
                for &port in &ports {
                    let addr = format!("{}:{}", target_ip, port);
                    if let Ok(Ok(_)) = timeout(Duration::from_millis(150), TcpStream::connect(&addr)).await {
                        let service = match port {
                            8080 => "ApexConnect / Web Gateway",
                            3389 => "Windows RDP",
                            445 => "Windows PC (SMB)",
                            80 => "Web / Ağ Cihazı",
                            _ => "Aktif Cihaz",
                        };

                        return Some(DiscoveredDevice {
                            ip: target_ip.clone(),
                            hostname: format!("Cihaz-{}", target_ip.split('.').last().unwrap_or("")),
                            device_id: None,
                            service_type: service.to_string(),
                            is_online: true,
                        });
                    }
                }
                None
            }));
        }

        for h in handles {
            if let Ok(Some(dev)) = h.await {
                // Avoid duplicates
                if !results.iter().any(|d| d.ip == dev.ip) {
                    results.push(dev);
                }
            }
        }

        results
    }
}
