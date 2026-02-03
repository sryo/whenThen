use std::collections::HashMap;
use std::sync::Arc;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tracing::{info, warn, error};

use crate::models::DiscoveredDevice;

const CHROMECAST_SERVICE: &str = "_googlecast._tcp.local.";

pub async fn start_discovery(
    app_handle: AppHandle,
    discovered_devices: Arc<RwLock<HashMap<String, DiscoveredDevice>>>,
    mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
) {
    let mdns = match ServiceDaemon::new() {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to create mDNS daemon: {}", e);
            return;
        }
    };

    let receiver = match mdns.browse(CHROMECAST_SERVICE) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to browse for Chromecast: {}", e);
            return;
        }
    };

    info!("Started Chromecast discovery");

    loop {
        tokio::select! {
            _ = &mut shutdown_rx => {
                info!("Stopping Chromecast discovery");
                let _ = mdns.stop_browse(CHROMECAST_SERVICE);
                let _ = mdns.shutdown();
                break;
            }
            event = tokio::task::spawn_blocking({
                let receiver = receiver.clone();
                move || receiver.recv()
            }) => {
                match event {
                    Ok(Ok(service_event)) => {
                        handle_service_event(
                            service_event,
                            &app_handle,
                            &discovered_devices,
                        ).await;
                    }
                    Ok(Err(e)) => {
                        warn!("mDNS receive error: {}", e);
                        break;
                    }
                    Err(e) => {
                        warn!("mDNS task error: {}", e);
                        break;
                    }
                }
            }
        }
    }
}

async fn handle_service_event(
    event: ServiceEvent,
    app_handle: &AppHandle,
    discovered_devices: &Arc<RwLock<HashMap<String, DiscoveredDevice>>>,
) {
    match event {
        ServiceEvent::ServiceResolved(info) => {
            let addresses = info.get_addresses();
            // Prefer IPv4 to avoid duplicates from dual-stack resolution
            let address = match addresses.iter().find(|a| a.is_ipv4()).or_else(|| addresses.iter().next()) {
                Some(addr) => addr.to_string(),
                None => return,
            };
            let port = info.get_port();

            let properties = info.get_properties();
            let friendly_name = properties
                .get_property_val_str("fn")
                .unwrap_or("Chromecast")
                .to_string();
            let model = properties
                .get_property_val_str("md")
                .unwrap_or("Unknown")
                .to_string();

            let id = format!("{}:{}", address, port);

            let device = DiscoveredDevice {
                id: id.clone(),
                name: friendly_name.clone(),
                model: model.clone(),
                address: address.clone(),
                port,
            };

            info!("Chromecast found: {} ({}) at {}:{}", friendly_name, model, address, port);

            discovered_devices.write().await.insert(id.clone(), device);

            #[derive(serde::Serialize, Clone)]
            struct DeviceFound {
                id: String,
                name: String,
                model: String,
                address: String,
                port: u16,
            }

            app_handle
                .emit(
                    "chromecast:device-found",
                    DeviceFound {
                        id,
                        name: friendly_name,
                        model,
                        address,
                        port,
                    },
                )
                .unwrap_or_default();
        }
        ServiceEvent::ServiceRemoved(_, fullname) => {
            let mut devices = discovered_devices.write().await;
            // Match by device ID (address:port) or exact fullname; avoid
            // false positives from substring matching on friendly names.
            let removed_id = devices
                .iter()
                .find(|(id, _)| fullname.contains(id.as_str()))
                .or_else(|| devices.iter().find(|(_, d)| fullname.contains(&d.name)))
                .map(|(id, _)| id.clone());

            if let Some(id) = removed_id {
                devices.remove(&id);

                #[derive(serde::Serialize, Clone)]
                struct DeviceLost {
                    id: String,
                }

                app_handle
                    .emit("chromecast:device-lost", DeviceLost { id })
                    .unwrap_or_default();
            }
        }
        _ => {}
    }
}
