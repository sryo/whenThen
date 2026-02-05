use tauri::{AppHandle, State};

use crate::errors::Result;
use crate::models::{TorrentAddOptions, TorrentAddedResponse, TorrentDetails, TorrentFileInfo, TorrentSummary};
use crate::services::torrent_engine;
use crate::state::AppState;

#[tauri::command]
pub async fn torrent_sync_restored(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<TorrentSummary>> {
    torrent_engine::sync_restored_torrents(&state, &app_handle).await
}

#[tauri::command]
pub async fn torrent_add_magnet(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    magnet_url: String,
    options: Option<TorrentAddOptions>,
) -> Result<TorrentAddedResponse> {
    torrent_engine::add_magnet(&state, &app_handle, magnet_url, options).await
}

#[tauri::command]
pub async fn torrent_add_file(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    path: String,
    options: Option<TorrentAddOptions>,
) -> Result<TorrentAddedResponse> {
    torrent_engine::add_torrent_file(&state, &app_handle, path, options).await
}

#[tauri::command]
pub async fn torrent_add_bytes(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    file_bytes: Vec<u8>,
    options: Option<TorrentAddOptions>,
) -> Result<TorrentAddedResponse> {
    torrent_engine::add_torrent_bytes(&state, &app_handle, file_bytes, options).await
}

#[tauri::command]
pub async fn torrent_list(state: State<'_, AppState>) -> Result<Vec<TorrentSummary>> {
    torrent_engine::list_torrents(&state).await
}

#[tauri::command]
pub async fn torrent_details(
    state: State<'_, AppState>,
    id: usize,
) -> Result<TorrentDetails> {
    torrent_engine::get_torrent_details(&state, id).await
}

#[tauri::command]
pub async fn torrent_files(
    state: State<'_, AppState>,
    id: usize,
) -> Result<Vec<TorrentFileInfo>> {
    torrent_engine::get_torrent_files(&state, id).await
}

#[tauri::command]
pub async fn torrent_pause(state: State<'_, AppState>, id: usize) -> Result<()> {
    torrent_engine::pause_torrent(&state, id).await
}

#[tauri::command]
pub async fn torrent_resume(state: State<'_, AppState>, id: usize) -> Result<()> {
    torrent_engine::resume_torrent(&state, id).await
}

#[tauri::command]
pub async fn torrent_recheck(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    id: usize,
) -> Result<TorrentAddedResponse> {
    torrent_engine::recheck_torrent(&state, &app_handle, id).await
}

#[tauri::command]
pub async fn torrent_delete(
    state: State<'_, AppState>,
    id: usize,
    delete_files: bool,
) -> Result<()> {
    torrent_engine::delete_torrent(&state, id, delete_files).await
}

#[tauri::command]
pub async fn torrent_update_files(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    id: usize,
    only_files: Vec<usize>,
) -> Result<TorrentAddedResponse> {
    torrent_engine::update_torrent_files(&state, &app_handle, id, only_files).await
}

#[tauri::command]
pub async fn torrent_rename_files(
    state: State<'_, AppState>,
    torrent_id: usize,
    renames: Vec<(usize, String)>,
) -> Result<()> {
    torrent_engine::rename_torrent_files(&state, torrent_id, renames).await
}
