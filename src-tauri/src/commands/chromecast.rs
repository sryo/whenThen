use tauri::{AppHandle, Emitter, State};

use crate::errors::{WhenThenError, Result};
use crate::models::{ChromecastDeviceInfo, DeviceStatus};
use crate::services::chromecast_device::ChromecastConnection;
use crate::services::chromecast_discovery;
use crate::state::AppState;

#[tauri::command]
pub async fn chromecast_start_discovery(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<()> {
    let mut shutdown_guard = state.discovery_shutdown.lock().await;
    if shutdown_guard.is_some() {
        return Ok(()); // Already running
    }

    let (tx, rx) = tokio::sync::oneshot::channel();
    *shutdown_guard = Some(tx);
    drop(shutdown_guard);

    let devices = state.discovered_devices.clone();
    tokio::spawn(async move {
        chromecast_discovery::start_discovery(app_handle, devices, rx).await;
    });

    Ok(())
}

#[tauri::command]
pub async fn chromecast_stop_discovery(state: State<'_, AppState>) -> Result<()> {
    let mut shutdown_guard = state.discovery_shutdown.lock().await;
    if let Some(tx) = shutdown_guard.take() {
        let _ = tx.send(());
    }
    Ok(())
}

#[tauri::command]
pub async fn chromecast_list_devices(
    state: State<'_, AppState>,
) -> Result<Vec<ChromecastDeviceInfo>> {
    let discovered = state.discovered_devices.read().await;
    let connections = state.active_connections.lock().await;

    let devices: Vec<ChromecastDeviceInfo> = discovered
        .values()
        .map(|d| {
            let status = if connections.contains_key(&d.id) {
                DeviceStatus::Connected
            } else {
                DeviceStatus::Discovered
            };
            d.to_info(status)
        })
        .collect();

    Ok(devices)
}

#[tauri::command]
pub async fn chromecast_connect(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    device_id: String,
) -> Result<()> {
    let device = {
        let devices = state.discovered_devices.read().await;
        devices
            .get(&device_id)
            .ok_or_else(|| WhenThenError::DeviceNotFound(device_id.clone()))?
            .clone()
    };

    let connection = ChromecastConnection::connect(
        device.id.clone(),
        device.name.clone(),
        device.address.clone(),
        device.port,
        Some(app_handle.clone()),
    )
    .await?;

    state
        .active_connections
        .lock()
        .await
        .insert(device_id.clone(), connection);

    #[derive(serde::Serialize, Clone)]
    struct Connected {
        id: String,
        name: String,
    }

    app_handle
        .emit(
            "chromecast:connected",
            Connected {
                id: device_id,
                name: device.name,
            },
        )
        .unwrap_or_default();

    Ok(())
}

#[tauri::command]
pub async fn chromecast_disconnect(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    device_id: String,
) -> Result<()> {
    let mut connections = state.active_connections.lock().await;
    if let Some(conn) = connections.remove(&device_id) {
        conn.disconnect().await;

        #[derive(serde::Serialize, Clone)]
        struct Disconnected {
            id: String,
            reason: String,
        }

        app_handle
            .emit(
                "chromecast:disconnected",
                Disconnected {
                    id: device_id,
                    reason: "User disconnected".into(),
                },
            )
            .unwrap_or_default();
    }

    Ok(())
}
