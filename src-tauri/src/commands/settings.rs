use tauri::{AppHandle, State};
use tauri_plugin_store::StoreExt;

use crate::errors::Result;
use crate::models::AppConfig;
use crate::services::{torrent_engine, folder_watcher};
use crate::state::AppState;

const STORE_FILE: &str = "settings.json";
const STORE_KEY: &str = "config";

#[tauri::command]
pub async fn settings_get(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<AppConfig> {
    // Try loading from store first
    if let Ok(store) = app.store(STORE_FILE) {
        if let Some(value) = store.get(STORE_KEY) {
            if let Ok(config) = serde_json::from_value::<AppConfig>(value) {
                let mut current = state.config.write().await;
                *current = config.clone();
                return Ok(config);
            }
        }
    }
    let config = state.config.read().await;
    Ok(config.clone())
}

#[tauri::command]
pub async fn settings_update(
    app: AppHandle,
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<AppConfig> {
    let old_config = state.config.read().await.clone();
    let mut current = state.config.write().await;
    *current = config.clone();
    drop(current);

    // Apply speed limits to the running session
    if let Some(session) = state.torrent_session.read().await.as_ref() {
        torrent_engine::apply_speed_limits(session, config.max_download_speed, config.max_upload_speed);
    }

    // Restart folder watcher if watch config changed
    if old_config.watch_folders != config.watch_folders
        || old_config.watch_folders_enabled != config.watch_folders_enabled
    {
        folder_watcher::stop_watching(&state.folder_watcher).await;
        if config.watch_folders_enabled && !config.watch_folders.is_empty() {
            if let Some(handle) = folder_watcher::start_watching(
                config.watch_folders.clone(),
                app.clone(),
            ) {
                *state.folder_watcher.lock().await = Some(handle);
            }
        }
    }

    // Persist to store
    if let Ok(store) = app.store(STORE_FILE) {
        if let Ok(value) = serde_json::to_value(&config) {
            let _ = store.set(STORE_KEY, value);
            let _ = store.save();
        }
    }

    Ok(config)
}
