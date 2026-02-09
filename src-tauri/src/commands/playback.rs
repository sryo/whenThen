use std::path::PathBuf;

use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::errors::{WhenThenError, Result};
use crate::models::PlaybackStatusResponse;
use crate::services::media_server::TokenEntry;
use crate::services::torrent_engine::{get_local_ip, expand_path};
use crate::state::AppState;

#[tauri::command]
pub async fn playback_cast_torrent(
    _app_handle: AppHandle,
    state: State<'_, AppState>,
    device_id: String,
    torrent_id: usize,
    file_index: usize,
) -> Result<()> {
    let local_ip = get_local_ip();
    let port = state.media_server.port;
    let url = format!(
        "http://{}:{}/torrent/{}/stream/{}",
        local_ip, port, torrent_id, file_index
    );

    let content_type = {
        let session_guard = state.torrent_session.read().await;
        let session = session_guard
            .as_ref()
            .ok_or_else(|| WhenThenError::Torrent("Session not initialized".into()))?;

        let handle = session
            .get(librqbit::api::TorrentIdOrHash::Id(torrent_id))
            .ok_or(WhenThenError::TorrentNotFound(torrent_id))?;

        let file_details: Vec<String> = handle.with_metadata(|meta| {
            meta.info.iter_file_details()
                .map(|iter| {
                    iter.map(|fi| {
                        fi.filename.to_string()
                            .unwrap_or_else(|_| "<INVALID NAME>".to_string())
                    }).collect::<Vec<_>>()
                })
                .unwrap_or_default()
        }).map_err(|e| WhenThenError::Torrent(format!("Metadata error: {e}")))?;

        let filename = file_details
            .get(file_index)
            .ok_or_else(|| WhenThenError::Torrent("File index out of range".into()))?;

        mime_guess::from_path(filename)
            .first_raw()
            .unwrap_or("application/octet-stream")
            .to_string()
    };

    let subtitle_url = {
        let subs = state.current_subtitles.read().await;
        if subs.is_some() {
            Some(format!("http://{}:{}/subtitles.vtt", local_ip, port))
        } else {
            None
        }
    };

    let connections = state.active_connections.lock().await;
    let conn = connections
        .get(&device_id)
        .ok_or_else(|| WhenThenError::DeviceNotFound(device_id.clone()))?;

    conn.load_media(url, content_type, subtitle_url).await?;

    Ok(())
}

#[tauri::command]
pub async fn playback_cast_local_file(
    _app_handle: AppHandle,
    state: State<'_, AppState>,
    device_id: String,
    file_path: String,
) -> Result<()> {
    let path = std::path::Path::new(&file_path);
    if !path.exists() {
        return Err(WhenThenError::FileNotFound(file_path));
    }

    let token = Uuid::new_v4().to_string();
    state
        .local_file_tokens
        .write()
        .await
        .insert(token.clone(), TokenEntry {
            path: file_path.clone(),
            created_at: std::time::Instant::now(),
        });

    let local_ip = get_local_ip();
    let port = state.media_server.port;
    let url = format!("http://{}:{}/local/{}", local_ip, port, token);

    let content_type = mime_guess::from_path(&file_path)
        .first_raw()
        .unwrap_or("application/octet-stream")
        .to_string();

    let subtitle_url = {
        let subs = state.current_subtitles.read().await;
        if subs.is_some() {
            Some(format!("http://{}:{}/subtitles.vtt", local_ip, port))
        } else {
            None
        }
    };

    let connections = state.active_connections.lock().await;
    let conn = connections
        .get(&device_id)
        .ok_or_else(|| WhenThenError::DeviceNotFound(device_id.clone()))?;

    conn.load_media(url, content_type, subtitle_url).await?;

    Ok(())
}

#[tauri::command]
pub async fn playback_play(
    state: State<'_, AppState>,
    device_id: String,
) -> Result<()> {
    let connections = state.active_connections.lock().await;
    let conn = connections
        .get(&device_id)
        .ok_or_else(|| WhenThenError::DeviceNotFound(device_id.clone()))?;
    conn.play().await
}

#[tauri::command]
pub async fn playback_pause(
    state: State<'_, AppState>,
    device_id: String,
) -> Result<()> {
    let connections = state.active_connections.lock().await;
    let conn = connections
        .get(&device_id)
        .ok_or_else(|| WhenThenError::DeviceNotFound(device_id.clone()))?;
    conn.pause().await
}

#[tauri::command]
pub async fn playback_stop(
    state: State<'_, AppState>,
    device_id: String,
) -> Result<()> {
    let connections = state.active_connections.lock().await;
    let conn = connections
        .get(&device_id)
        .ok_or_else(|| WhenThenError::DeviceNotFound(device_id.clone()))?;
    let result = conn.stop().await;
    drop(connections);
    *state.current_subtitles.write().await = None;
    result
}

#[tauri::command]
pub async fn playback_seek(
    state: State<'_, AppState>,
    device_id: String,
    position_secs: f64,
) -> Result<()> {
    let connections = state.active_connections.lock().await;
    let conn = connections
        .get(&device_id)
        .ok_or_else(|| WhenThenError::DeviceNotFound(device_id.clone()))?;
    conn.seek(position_secs).await
}

#[tauri::command]
pub async fn playback_seek_relative(
    state: State<'_, AppState>,
    device_id: String,
    delta_secs: f64,
) -> Result<()> {
    let connections = state.active_connections.lock().await;
    let conn = connections
        .get(&device_id)
        .ok_or_else(|| WhenThenError::DeviceNotFound(device_id.clone()))?;

    let status = conn.get_status().await?;
    drop(connections);

    let new_position = (status.current_time + delta_secs).max(0.0);
    let connections = state.active_connections.lock().await;
    let conn = connections
        .get(&device_id)
        .ok_or_else(|| WhenThenError::DeviceNotFound(device_id.clone()))?;
    conn.seek(new_position).await
}

#[tauri::command]
pub async fn playback_set_volume(
    state: State<'_, AppState>,
    device_id: String,
    volume: f64,
) -> Result<()> {
    let connections = state.active_connections.lock().await;
    let conn = connections
        .get(&device_id)
        .ok_or_else(|| WhenThenError::DeviceNotFound(device_id.clone()))?;
    conn.set_volume(volume.clamp(0.0, 1.0)).await
}

#[tauri::command]
pub async fn playback_get_status(
    state: State<'_, AppState>,
    device_id: String,
) -> Result<PlaybackStatusResponse> {
    let connections = state.active_connections.lock().await;
    let conn = connections
        .get(&device_id)
        .ok_or_else(|| WhenThenError::DeviceNotFound(device_id.clone()))?;
    conn.get_status().await
}

#[tauri::command]
pub async fn playback_open_in_app(
    state: State<'_, AppState>,
    torrent_id: usize,
    file_index: usize,
    app_name: String,
) -> Result<()> {
    let (download_dir, relative_path) = {
        let session_guard = state.torrent_session.read().await;
        let session = session_guard
            .as_ref()
            .ok_or_else(|| WhenThenError::Torrent("Session not initialized".into()))?;

        let handle = session
            .get(librqbit::api::TorrentIdOrHash::Id(torrent_id))
            .ok_or(WhenThenError::TorrentNotFound(torrent_id))?;

        let file_details: Vec<String> = handle.with_metadata(|meta| {
            meta.info.iter_file_details()
                .map(|iter| {
                    iter.map(|fi| {
                        fi.filename.to_string()
                            .unwrap_or_else(|_| "<INVALID NAME>".to_string())
                    }).collect::<Vec<_>>()
                })
                .unwrap_or_default()
        }).map_err(|e| WhenThenError::Torrent(format!("Metadata error: {e}")))?;

        let relative = file_details
            .get(file_index)
            .ok_or_else(|| WhenThenError::Torrent("File index out of range".into()))?
            .clone();

        let cfg = state.config.read().await;
        (cfg.download_directory.clone(), relative)
    };

    let full_path = expand_path(&download_dir).join(&relative_path);
    if !full_path.exists() {
        return Err(WhenThenError::FileNotFound(
            full_path.to_string_lossy().to_string(),
        ));
    }

    std::process::Command::new("open")
        .args(["-a", &app_name, &full_path.to_string_lossy()])
        .spawn()
        .map_err(|e| WhenThenError::Internal(format!("Failed to open in {app_name}: {e}")))?;

    Ok(())
}
