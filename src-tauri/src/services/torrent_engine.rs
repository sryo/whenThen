use std::sync::Arc;
use std::num::NonZeroU32;
use std::path::PathBuf;
use librqbit::{
    AddTorrent, AddTorrentOptions, AddTorrentResponse, Session, SessionOptions,
    SessionPersistenceConfig,
    dht::PersistentDhtConfig,
    limits::LimitsConfig,
};
use tauri::{AppHandle, Emitter};
use tracing::{info, debug, warn};

use crate::errors::{WhenThenError, Result};
use crate::models::{
    AppConfig, TorrentAddedResponse, TorrentFileInfo, TorrentSummary, TorrentDetails,
    TorrentState, TorrentAddOptions,
};
use crate::state::AppState;

fn speed_limit(bps: u64) -> Option<NonZeroU32> {
    if bps == 0 { None } else { NonZeroU32::new(bps as u32) }
}

pub fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    } else if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(path)
}

pub async fn init_session(config: &AppConfig, persistence_dir: PathBuf) -> Result<Arc<Session>> {
    let output_dir = if config.download_directory.is_empty() {
        dirs::download_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"))
    } else {
        expand_path(&config.download_directory)
    };
    let output_dir_display = output_dir.display().to_string();

    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir)
            .map_err(|e| WhenThenError::Config(format!("Cannot create download dir: {e}")))?;
    }

    if !persistence_dir.exists() {
        std::fs::create_dir_all(&persistence_dir)
            .map_err(|e| WhenThenError::Config(format!("Cannot create persistence dir: {e}")))?;
    }

    let port = config.listen_port;

    let session = Session::new_with_opts(
        output_dir,
        SessionOptions {
            disable_dht: false,
            disable_dht_persistence: false,
            dht_config: Some(PersistentDhtConfig::default()),
            persistence: Some(SessionPersistenceConfig::Json {
                folder: Some(persistence_dir.clone()),
            }),
            fastresume: true,
            listen_port_range: Some(port..port + 20),
            enable_upnp_port_forwarding: config.enable_upnp,
            ratelimits: LimitsConfig {
                download_bps: speed_limit(config.max_download_speed),
                upload_bps: speed_limit(config.max_upload_speed),
            },
            ..Default::default()
        },
    )
    .await
    .map_err(|e| WhenThenError::Torrent(format!("Failed to init torrent session: {e}")))?;

    info!(
        "Torrent session initialized — download dir: {}, persistence: {}, listen port: {}..{}, UPnP: {}",
        output_dir_display, persistence_dir.display(), port, port + 20, config.enable_upnp
    );
    Ok(session)
}

/// Safe to call on a running session.
pub fn apply_speed_limits(session: &Session, download_bps: u64, upload_bps: u64) {
    session.ratelimits.set_download_bps(speed_limit(download_bps));
    session.ratelimits.set_upload_bps(speed_limit(upload_bps));
    info!("Speed limits updated — download: {} B/s, upload: {} B/s (0 = unlimited)", download_bps, upload_bps);
}

pub async fn sync_restored_torrents(
    state: &AppState,
    app_handle: &AppHandle,
) -> Result<Vec<TorrentSummary>> {
    let session = {
        let guard = state.torrent_session.read().await;
        match guard.as_ref() {
            Some(s) => s.clone(),
            None => return Ok(vec![]),
        }
    };

    let download_dir = {
        let cfg = state.config.read().await;
        expand_path(&cfg.download_directory)
    };

    let torrent_list: Vec<_> = session.with_torrents(|torrents| {
        torrents.map(|(id, h)| (id, h.clone())).collect::<Vec<_>>()
    });

    let mut summaries = Vec::new();

    for (id, handle) in torrent_list {
        let name = handle.name().unwrap_or_else(|| "Unknown".to_string());
        let stats = handle.stats();

        // Remove completed torrents whose files were deleted externally.
        if stats.finished {
            let output_path = download_dir.join(&name);
            if !output_path.exists() {
                info!(torrent_id = id, name = %name, "Removing completed torrent (files moved/deleted)");
                let _ = session
                    .delete(librqbit::api::TorrentIdOrHash::Id(id), false)
                    .await;
                continue;
            }
        }

        {
            let mut names = state.torrent_names.write().await;
            names.entry(id).or_insert_with(|| name.clone());
        }

        let state_val = if stats.finished {
            TorrentState::Completed
        } else {
            match stats.state {
                librqbit::TorrentStatsState::Paused => TorrentState::Paused,
                librqbit::TorrentStatsState::Error => TorrentState::Error,
                librqbit::TorrentStatsState::Initializing => TorrentState::Initializing,
                _ => TorrentState::Downloading,
            }
        };

        if state_val != TorrentState::Completed {
            spawn_progress_emitter(state, app_handle.clone(), id);
        }

        let total_bytes = stats.total_bytes;
        let downloaded = stats.progress_bytes;
        let progress = if total_bytes > 0 {
            downloaded as f64 / total_bytes as f64
        } else {
            0.0
        };

        let (dl_speed, ul_speed, peers) = if let Some(ref live) = stats.live {
            (
                (live.download_speed.mbps * 1024.0 * 1024.0) as u64,
                (live.upload_speed.mbps * 1024.0 * 1024.0) as u64,
                live.snapshot.peer_stats.live,
            )
        } else {
            (0, 0, 0)
        };

        let file_count = stats.file_progress.len();

        summaries.push(TorrentSummary {
            id,
            name,
            info_hash: handle.info_hash().as_string(),
            state: state_val,
            progress,
            download_speed: dl_speed,
            upload_speed: ul_speed,
            peers_connected: peers,
            total_bytes,
            downloaded_bytes: downloaded,
            file_count,
        });
    }

    Ok(summaries)
}

fn check_disk_space(download_dir: &str) -> Result<()> {
    let path = std::path::Path::new(download_dir);
    if !path.exists() {
        return Ok(()); // Will be created later; skip check
    }
    // TODO: check available space when std::fs::available_space stabilizes
    let _ = path;
    Ok(())
}

pub async fn add_magnet(
    state: &AppState,
    app_handle: &AppHandle,
    magnet_url: String,
    options: Option<TorrentAddOptions>,
) -> Result<TorrentAddedResponse> {
    let session = {
        let guard = state.torrent_session.read().await;
        guard.as_ref().ok_or_else(|| {
            WhenThenError::Torrent("Torrent session not initialized".into())
        })?.clone()
    };

    let incomplete_dir = {
        let cfg = state.config.read().await;
        let _ = check_disk_space(&cfg.download_directory);
        if cfg.incomplete_directory.is_empty() {
            None
        } else {
            Some(expand_path(&cfg.incomplete_directory).to_string_lossy().to_string())
        }
    };

    let (output_folder, only_files) = if let Some(ref opts) = options {
        let folder = opts.output_folder.as_ref().map(|p| expand_path(p).to_string_lossy().to_string());
        (folder, opts.only_files.clone())
    } else {
        (None, None)
    };

    let effective_output = output_folder.or(incomplete_dir);

    let add_opts = AddTorrentOptions {
        output_folder: effective_output,
        only_files,
        overwrite: true,
        ..Default::default()
    };

    let response = session
        .add_torrent(
            AddTorrent::from_url(&magnet_url),
            Some(add_opts),
        )
        .await
        .map_err(|e| WhenThenError::Torrent(format!("Failed to add magnet: {e}")))?;

    let (handle, is_new) = match response {
        AddTorrentResponse::Added(_, handle) => (handle, true),
        AddTorrentResponse::AlreadyManaged(_, handle) => (handle, false),
        AddTorrentResponse::ListOnly(_) => {
            return Err(WhenThenError::Torrent("Torrent added in list-only mode".into()));
        }
    };

    let id = handle.id();
    let name = handle.name().unwrap_or_else(|| "Unknown".to_string());
    let info_hash = handle.info_hash().as_string();

    state.torrent_names.write().await.insert(id, name.clone());

    let media_server_port = state.media_server.port;
    let local_ip = get_local_ip();
    let files = build_file_list(&handle, &local_ip, media_server_port);

    let result = TorrentAddedResponse {
        id,
        name,
        info_hash,
        files,
    };

    if is_new {
        spawn_progress_emitter(state, app_handle.clone(), id);
        app_handle
            .emit("torrent:added", &result)
            .unwrap_or_default();
    } else {
        info!(id, "Torrent already managed, skipping torrent:added event");
    }

    Ok(result)
}

pub async fn add_torrent_file(
    state: &AppState,
    app_handle: &AppHandle,
    path: String,
    options: Option<TorrentAddOptions>,
) -> Result<TorrentAddedResponse> {
    let session = {
        let guard = state.torrent_session.read().await;
        guard.as_ref().ok_or_else(|| {
            WhenThenError::Torrent("Torrent session not initialized".into())
        })?.clone()
    };

    let file_content = std::fs::read(&path)
        .map_err(|e| WhenThenError::FileNotFound(format!("{}: {}", path, e)))?;

    let incomplete_dir = {
        let cfg = state.config.read().await;
        if cfg.incomplete_directory.is_empty() {
            None
        } else {
            Some(expand_path(&cfg.incomplete_directory).to_string_lossy().to_string())
        }
    };

    let (output_folder, only_files) = if let Some(ref opts) = options {
        let folder = opts.output_folder.as_ref().map(|p| expand_path(p).to_string_lossy().to_string());
        (folder, opts.only_files.clone())
    } else {
        (None, None)
    };

    let effective_output = output_folder.or(incomplete_dir);

    let add_opts = AddTorrentOptions {
        output_folder: effective_output,
        only_files,
        overwrite: true,
        ..Default::default()
    };

    let response = session
        .add_torrent(
            AddTorrent::TorrentFileBytes(file_content.into()),
            Some(add_opts),
        )
        .await
        .map_err(|e| WhenThenError::Torrent(format!("Failed to add torrent: {e}")))?;

    let (handle, is_new) = match response {
        AddTorrentResponse::Added(_, handle) => (handle, true),
        AddTorrentResponse::AlreadyManaged(_, handle) => (handle, false),
        AddTorrentResponse::ListOnly(_) => {
            return Err(WhenThenError::Torrent("Torrent added in list-only mode".into()));
        }
    };

    let id = handle.id();
    let name = handle.name().unwrap_or_else(|| "Unknown".to_string());
    let info_hash = handle.info_hash().as_string();

    state.torrent_names.write().await.insert(id, name.clone());

    let media_server_port = state.media_server.port;
    let local_ip = get_local_ip();
    let files = build_file_list(&handle, &local_ip, media_server_port);

    let result = TorrentAddedResponse {
        id,
        name,
        info_hash,
        files,
    };

    if is_new {
        spawn_progress_emitter(state, app_handle.clone(), id);
        app_handle
            .emit("torrent:added", &result)
            .unwrap_or_default();
    } else {
        info!(id, "Torrent already managed, skipping torrent:added event");
    }

    let should_delete = state.config.read().await.delete_torrent_file_on_add;
    if should_delete {
        let _ = std::fs::remove_file(&path);
    }

    Ok(result)
}

pub async fn add_torrent_bytes(
    state: &AppState,
    app_handle: &AppHandle,
    file_bytes: Vec<u8>,
    options: Option<TorrentAddOptions>,
) -> Result<TorrentAddedResponse> {
    let session = {
        let guard = state.torrent_session.read().await;
        guard.as_ref().ok_or_else(|| {
            WhenThenError::Torrent("Torrent session not initialized".into())
        })?.clone()
    };

    let incomplete_dir = {
        let cfg = state.config.read().await;
        let _ = check_disk_space(&cfg.download_directory);
        if cfg.incomplete_directory.is_empty() {
            None
        } else {
            Some(expand_path(&cfg.incomplete_directory).to_string_lossy().to_string())
        }
    };

    let (output_folder, only_files) = if let Some(ref opts) = options {
        let folder = opts.output_folder.as_ref().map(|p| expand_path(p).to_string_lossy().to_string());
        (folder, opts.only_files.clone())
    } else {
        (None, None)
    };

    let effective_output = output_folder.or(incomplete_dir);

    let add_opts = AddTorrentOptions {
        output_folder: effective_output,
        only_files,
        overwrite: true,
        ..Default::default()
    };

    let response = session
        .add_torrent(
            AddTorrent::TorrentFileBytes(file_bytes.into()),
            Some(add_opts),
        )
        .await
        .map_err(|e| WhenThenError::Torrent(format!("Failed to add torrent: {e}")))?;

    let (handle, is_new) = match response {
        AddTorrentResponse::Added(_, handle) => (handle, true),
        AddTorrentResponse::AlreadyManaged(_, handle) => (handle, false),
        AddTorrentResponse::ListOnly(_) => {
            return Err(WhenThenError::Torrent("Torrent added in list-only mode".into()));
        }
    };

    let id = handle.id();
    let name = handle.name().unwrap_or_else(|| "Unknown".to_string());
    let info_hash = handle.info_hash().as_string();

    state.torrent_names.write().await.insert(id, name.clone());

    let media_server_port = state.media_server.port;
    let local_ip = get_local_ip();
    let files = build_file_list(&handle, &local_ip, media_server_port);

    let result = TorrentAddedResponse {
        id,
        name,
        info_hash,
        files,
    };

    if is_new {
        spawn_progress_emitter(state, app_handle.clone(), id);
        app_handle
            .emit("torrent:added", &result)
            .unwrap_or_default();
    } else {
        info!(id, "Torrent already managed, skipping torrent:added event");
    }

    Ok(result)
}

pub async fn list_torrents(state: &AppState) -> Result<Vec<TorrentSummary>> {
    let session = {
        let guard = state.torrent_session.read().await;
        match guard.as_ref() {
            Some(s) => s.clone(),
            None => return Ok(vec![]),
        }
    };

    let mut summaries = Vec::new();
    let names = state.torrent_names.read().await;

    let torrent_list: Vec<_> = session.with_torrents(|torrents| {
        torrents.map(|(id, h)| (id, h.clone())).collect::<Vec<_>>()
    });

    for (id, handle) in torrent_list {
        let stats = handle.stats();
        let name = names.get(&id).cloned()
            .unwrap_or_else(|| handle.name().unwrap_or_else(|| "Unknown".to_string()));
        let total_bytes = stats.total_bytes;
        let downloaded = stats.progress_bytes;
        let progress = if total_bytes > 0 {
            downloaded as f64 / total_bytes as f64
        } else {
            0.0
        };

        let (dl_speed, ul_speed, peers) = if let Some(ref live) = stats.live {
            (
                (live.download_speed.mbps * 1024.0 * 1024.0) as u64,
                (live.upload_speed.mbps * 1024.0 * 1024.0) as u64,
                live.snapshot.peer_stats.live,
            )
        } else {
            (0, 0, 0)
        };

        let state_val = if stats.finished {
            TorrentState::Completed
        } else {
            match stats.state {
                librqbit::TorrentStatsState::Paused => TorrentState::Paused,
                librqbit::TorrentStatsState::Error => TorrentState::Error,
                librqbit::TorrentStatsState::Initializing => TorrentState::Initializing,
                _ => TorrentState::Downloading,
            }
        };

        let file_count = stats.file_progress.len();

        summaries.push(TorrentSummary {
            id,
            name,
            info_hash: handle.info_hash().as_string(),
            state: state_val,
            progress,
            download_speed: dl_speed,
            upload_speed: ul_speed,
            peers_connected: peers,
            total_bytes,
            downloaded_bytes: downloaded,
            file_count,
        });
    }

    Ok(summaries)
}

pub async fn get_torrent_details(state: &AppState, id: usize) -> Result<TorrentDetails> {
    let session = {
        let guard = state.torrent_session.read().await;
        guard.as_ref().ok_or_else(|| {
            WhenThenError::Torrent("Torrent session not initialized".into())
        })?.clone()
    };

    let handle = session
        .get(librqbit::api::TorrentIdOrHash::Id(id))
        .ok_or(WhenThenError::TorrentNotFound(id))?;

    let stats = handle.stats();
    let names = state.torrent_names.read().await;
    let name = names.get(&id).cloned()
        .unwrap_or_else(|| handle.name().unwrap_or_else(|| "Unknown".to_string()));
    let total_bytes = stats.total_bytes;
    let downloaded = stats.progress_bytes;
    let progress = if total_bytes > 0 {
        downloaded as f64 / total_bytes as f64
    } else {
        0.0
    };

    let (dl_speed, ul_speed, peers) = if let Some(ref live) = stats.live {
        (
            (live.download_speed.mbps * 1024.0 * 1024.0) as u64,
            (live.upload_speed.mbps * 1024.0 * 1024.0) as u64,
            live.snapshot.peer_stats.live,
        )
    } else {
        (0, 0, 0)
    };

    let state_val = if stats.finished {
        TorrentState::Completed
    } else {
        match stats.state {
            librqbit::TorrentStatsState::Paused => TorrentState::Paused,
            librqbit::TorrentStatsState::Error => TorrentState::Error,
            librqbit::TorrentStatsState::Initializing => TorrentState::Initializing,
            _ => TorrentState::Downloading,
        }
    };

    let local_ip = get_local_ip();
    let media_server_port = state.media_server.port;
    let files = build_file_list(&handle, &local_ip, media_server_port);

    let output_folder = String::new(); // Session doesn't directly expose this

    Ok(TorrentDetails {
        id,
        name,
        info_hash: handle.info_hash().as_string(),
        state: state_val,
        progress,
        download_speed: dl_speed,
        upload_speed: ul_speed,
        peers_connected: peers,
        total_bytes,
        downloaded_bytes: downloaded,
        file_count: files.len(),
        files,
        output_folder,
    })
}

pub async fn get_torrent_files(state: &AppState, id: usize) -> Result<Vec<TorrentFileInfo>> {
    let session = {
        let guard = state.torrent_session.read().await;
        guard.as_ref().ok_or_else(|| {
            WhenThenError::Torrent("Torrent session not initialized".into())
        })?.clone()
    };

    let handle = session
        .get(librqbit::api::TorrentIdOrHash::Id(id))
        .ok_or(WhenThenError::TorrentNotFound(id))?;

    let local_ip = get_local_ip();
    let media_server_port = state.media_server.port;
    Ok(build_file_list(&handle, &local_ip, media_server_port))
}

pub async fn pause_torrent(state: &AppState, id: usize) -> Result<()> {
    let session = {
        let guard = state.torrent_session.read().await;
        guard.as_ref().ok_or_else(|| {
            WhenThenError::Torrent("Torrent session not initialized".into())
        })?.clone()
    };

    let handle = session
        .get(librqbit::api::TorrentIdOrHash::Id(id))
        .ok_or(WhenThenError::TorrentNotFound(id))?;

    session.pause(&handle).await
        .map_err(|e| WhenThenError::Torrent(format!("Failed to pause: {e}")))?;
    Ok(())
}

pub async fn resume_torrent(state: &AppState, id: usize) -> Result<()> {
    let session = {
        let guard = state.torrent_session.read().await;
        guard.as_ref().ok_or_else(|| {
            WhenThenError::Torrent("Torrent session not initialized".into())
        })?.clone()
    };

    let handle = session
        .get(librqbit::api::TorrentIdOrHash::Id(id))
        .ok_or(WhenThenError::TorrentNotFound(id))?;

    session.unpause(&handle).await
        .map_err(|e| WhenThenError::Torrent(format!("Failed to resume: {e}")))?;
    Ok(())
}

/// Forces piece re-verification via delete + re-add.
pub async fn recheck_torrent(
    state: &AppState,
    app_handle: &AppHandle,
    id: usize,
) -> Result<TorrentAddedResponse> {
    let session = {
        let guard = state.torrent_session.read().await;
        guard.as_ref().ok_or_else(|| {
            WhenThenError::Torrent("Torrent session not initialized".into())
        })?.clone()
    };

    let handle = session
        .get(librqbit::api::TorrentIdOrHash::Id(id))
        .ok_or(WhenThenError::TorrentNotFound(id))?;

    let torrent_bytes = handle
        .with_metadata(|m| m.torrent_bytes.clone())
        .map_err(|e| WhenThenError::Torrent(format!("Cannot read torrent metadata: {e}")))?;

    let name = handle.name().unwrap_or_else(|| "Unknown".to_string());

    // Delete from session, keep files on disk
    session
        .delete(librqbit::api::TorrentIdOrHash::Id(id), false)
        .await
        .map_err(|e| WhenThenError::Torrent(format!("Failed to delete torrent for recheck: {e}")))?;

    state.torrent_names.write().await.remove(&id);

    // Re-add with same bytes — librqbit will hash-check all pieces on init
    let add_opts = AddTorrentOptions {
        overwrite: true,
        ..Default::default()
    };

    let response = session
        .add_torrent(
            AddTorrent::TorrentFileBytes(torrent_bytes),
            Some(add_opts),
        )
        .await
        .map_err(|e| WhenThenError::Torrent(format!("Failed to re-add torrent for recheck: {e}")))?;

    let new_handle = match response {
        AddTorrentResponse::Added(_, h) => h,
        AddTorrentResponse::AlreadyManaged(_, h) => h,
        AddTorrentResponse::ListOnly(_) => {
            return Err(WhenThenError::Torrent("Torrent re-added in list-only mode".into()));
        }
    };

    let new_id = new_handle.id();
    let info_hash = new_handle.info_hash().as_string();

    state.torrent_names.write().await.insert(new_id, name.clone());

    let media_server_port = state.media_server.port;
    let local_ip = get_local_ip();
    let files = build_file_list(&new_handle, &local_ip, media_server_port);

    let result = TorrentAddedResponse {
        id: new_id,
        name: name.clone(),
        info_hash,
        files,
    };

    spawn_progress_emitter(state, app_handle.clone(), new_id);

    #[derive(serde::Serialize, Clone)]
    struct TorrentRechecked {
        old_id: usize,
        new_id: usize,
        name: String,
    }

    app_handle
        .emit("torrent:rechecked", &TorrentRechecked { old_id: id, new_id, name })
        .unwrap_or_default();

    info!(old_id = id, new_id, "Torrent rechecked");

    Ok(result)
}

pub async fn delete_torrent(state: &AppState, id: usize, delete_files: bool) -> Result<()> {
    let session = {
        let guard = state.torrent_session.read().await;
        guard.as_ref().ok_or_else(|| {
            WhenThenError::Torrent("Torrent session not initialized".into())
        })?.clone()
    };

    session
        .delete(librqbit::api::TorrentIdOrHash::Id(id), delete_files)
        .await
        .map_err(|e| WhenThenError::Torrent(format!("Failed to delete torrent: {e}")))?;

    state.torrent_names.write().await.remove(&id);
    Ok(())
}

fn build_file_list(
    handle: &Arc<librqbit::ManagedTorrent>,
    local_ip: &str,
    port: u16,
) -> Vec<TorrentFileInfo> {
    let id = handle.id();
    let mut files = Vec::new();

    let file_infos: Vec<(String, u64)> = match handle.with_metadata(|meta| {
        meta.info.iter_file_details()
            .map(|iter| {
                iter.map(|fi| {
                    let path_str = fi.filename.to_string()
                        .unwrap_or_else(|_| "<INVALID NAME>".to_string());
                    (path_str, fi.len)
                }).collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }) {
        Ok(infos) => infos,
        Err(_) => return files,
    };

    for (idx, (path_str, length)) in file_infos.into_iter().enumerate() {
        let name = path_str.rsplit('/').next().unwrap_or(&path_str).to_string();
        let mime = mime_guess::from_path(&name).first_raw().map(String::from);
        let is_playable = mime.as_ref().is_some_and(|m| {
            m.starts_with("video/") || m.starts_with("audio/")
        });
        let stream_url = if is_playable {
            Some(format!("http://{}:{}/torrent/{}/stream/{}", local_ip, port, id, idx))
        } else {
            None
        };

        files.push(TorrentFileInfo {
            index: idx,
            name,
            path: path_str,
            length,
            is_playable,
            mime_type: mime,
            stream_url,
        });
    }

    files
}

fn spawn_progress_emitter(state: &AppState, app_handle: AppHandle, torrent_id: usize) {
    let session = state.torrent_session.clone();
    let config = state.config.clone();

    debug!(torrent_id, "Progress emitter started");

    tokio::spawn(async move {
        let mut prev_state: Option<String> = None;

        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            let s = {
                let guard = session.read().await;
                match guard.as_ref() {
                    Some(s) => s.clone(),
                    None => {
                        warn!(torrent_id, "Progress emitter exiting: session gone");
                        break;
                    }
                }
            };

            let handle = match s.get(librqbit::api::TorrentIdOrHash::Id(torrent_id)) {
                Some(h) => h,
                None => {
                    warn!(torrent_id, "Progress emitter exiting: torrent not in session");
                    break;
                }
            };

            let stats = handle.stats();
            let total_bytes = stats.total_bytes;
            let downloaded = stats.progress_bytes;
            let progress = if total_bytes > 0 {
                downloaded as f64 / total_bytes as f64
            } else {
                0.0
            };

            let (dl_speed, ul_speed, peers) = if let Some(ref live) = stats.live {
                (
                    (live.download_speed.mbps * 1024.0 * 1024.0) as u64,
                    (live.upload_speed.mbps * 1024.0 * 1024.0) as u64,
                    live.snapshot.peer_stats.live,
                )
            } else {
                (0, 0, 0)
            };

            let state_val = if stats.finished {
                TorrentState::Completed
            } else {
                match stats.state {
                    librqbit::TorrentStatsState::Paused => TorrentState::Paused,
                    librqbit::TorrentStatsState::Error => TorrentState::Error,
                    librqbit::TorrentStatsState::Initializing => TorrentState::Initializing,
                    _ => TorrentState::Downloading,
                }
            };

            let state_str = format!("{:?}", state_val);
            if prev_state.as_ref() != Some(&state_str) {
                info!(
                    torrent_id,
                    state = %state_str,
                    total_bytes,
                    peers,
                    "Torrent state changed"
                );
                prev_state = Some(state_str);
            }

            #[derive(serde::Serialize, Clone)]
            struct TorrentProgress {
                id: usize,
                progress: f64,
                download_speed: u64,
                upload_speed: u64,
                peers_connected: usize,
                queued_peers: usize,
                connecting_peers: usize,
                downloaded_bytes: u64,
                uploaded_bytes: u64,
                total_bytes: u64,
                state: TorrentState,
            }

            let (uploaded_bytes, queued_peers, connecting_peers) = if let Some(ref live) = stats.live {
                (
                    live.snapshot.uploaded_bytes,
                    live.snapshot.peer_stats.queued,
                    live.snapshot.peer_stats.connecting,
                )
            } else {
                (0, 0, 0)
            };

            let progress_event = TorrentProgress {
                id: torrent_id,
                progress,
                download_speed: dl_speed,
                upload_speed: ul_speed,
                peers_connected: peers,
                queued_peers,
                connecting_peers,
                downloaded_bytes: downloaded,
                uploaded_bytes,
                total_bytes,
                state: state_val.clone(),
            };

            if let Err(e) = app_handle.emit("torrent:progress", &progress_event) {
                warn!(torrent_id, error = %e, "Failed to emit progress event");
            }

            if state_val == TorrentState::Completed {
                info!(torrent_id, "Download complete");

                let cfg = config.read().await;
                if !cfg.incomplete_directory.is_empty()
                    && cfg.incomplete_directory != cfg.download_directory
                {
                    let name = handle.name().unwrap_or_else(|| "Unknown".to_string());
                    let src = expand_path(&cfg.incomplete_directory).join(&name);
                    let dst = expand_path(&cfg.download_directory).join(&name);
                    drop(cfg);

                    if src.exists() {
                        if let Err(e) = std::fs::rename(&src, &dst) {
                            warn!(
                                torrent_id,
                                src = %src.display(),
                                dst = %dst.display(),
                                error = %e,
                                "Failed to move completed torrent from incomplete dir"
                            );
                        } else {
                            info!(torrent_id, dst = %dst.display(), "Moved completed download");
                        }
                    }
                }

                app_handle
                    .emit("torrent:completed", torrent_id)
                    .unwrap_or_default();
                break;
            }
        }

        debug!(torrent_id, "Progress emitter stopped");
    });
}

pub async fn move_torrent_files(state: &AppState, torrent_id: usize, destination: String) -> Result<()> {
    let session = {
        let guard = state.torrent_session.read().await;
        guard.as_ref().ok_or_else(|| {
            WhenThenError::Torrent("Torrent session not initialized".into())
        })?.clone()
    };

    let handle = session
        .get(librqbit::api::TorrentIdOrHash::Id(torrent_id))
        .ok_or(WhenThenError::TorrentNotFound(torrent_id))?;

    let dest_path = expand_path(&destination);
    if !dest_path.exists() {
        std::fs::create_dir_all(&dest_path)
            .map_err(|e| WhenThenError::Internal(format!("Cannot create destination: {e}")))?;
    }

    let output_folder = {
        let cfg = state.config.read().await;
        expand_path(&cfg.download_directory)
    };
    let torrent_name = handle.name().unwrap_or_else(|| "Unknown".to_string());
    let source_path = output_folder.join(&torrent_name);

    if source_path.exists() {
        if source_path.is_dir() {
            let target = dest_path.join(&torrent_name);
            std::fs::rename(&source_path, &target).map_err(|e| {
                WhenThenError::Internal(format!("Failed to move files: {e}"))
            })?;
        } else {
            let file_name = source_path.file_name().unwrap_or_default();
            let target = dest_path.join(file_name);
            std::fs::rename(&source_path, &target).map_err(|e| {
                WhenThenError::Internal(format!("Failed to move file: {e}"))
            })?;
        }
    } else {
        // Single-file torrents are placed directly in output folder.
        let file_info: Vec<String> = handle.with_metadata(|meta| {
            meta.info.iter_file_details()
                .map(|iter| {
                    iter.map(|fi| fi.filename.to_string().unwrap_or_default()).collect()
                })
                .unwrap_or_default()
        }).unwrap_or_default();

        if file_info.len() == 1 {
            let single_file = &file_info[0];
            let alt_source = output_folder.join(single_file);
            if alt_source.exists() {
                let file_name = alt_source.file_name().unwrap_or_default();
                let target = dest_path.join(file_name);
                std::fs::rename(&alt_source, &target).map_err(|e| {
                    WhenThenError::Internal(format!("Failed to move file: {e}"))
                })?;
            } else {
                return Err(WhenThenError::FileNotFound(format!(
                    "Torrent file not found at: {}",
                    alt_source.display()
                )));
            }
        } else {
            return Err(WhenThenError::FileNotFound(format!(
                "Torrent files not found at: {}",
                source_path.display()
            )));
        }
    }

    Ok(())
}

pub async fn rename_torrent_files(state: &AppState, torrent_id: usize, renames: Vec<(usize, String)>) -> Result<()> {
    let session = {
        let guard = state.torrent_session.read().await;
        guard.as_ref().ok_or_else(|| {
            WhenThenError::Torrent("Torrent session not initialized".into())
        })?.clone()
    };

    let handle = session
        .get(librqbit::api::TorrentIdOrHash::Id(torrent_id))
        .ok_or(WhenThenError::TorrentNotFound(torrent_id))?;

    let output_folder = {
        let cfg = state.config.read().await;
        expand_path(&cfg.download_directory)
    };
    let torrent_name = handle.name().unwrap_or_else(|| "Unknown".to_string());

    for (file_idx, new_name) in &renames {
        let original_path_str: Option<String> = handle.with_metadata(|meta| {
            meta.info.iter_file_details()
                .ok()
                .and_then(|mut iter| {
                    iter.nth(*file_idx).map(|fi| {
                        fi.filename.to_string().unwrap_or_default()
                    })
                })
        }).ok().flatten();

        if let Some(orig_rel) = original_path_str {
            let source = output_folder.join(&torrent_name).join(&orig_rel);
            if !source.exists() {
                // Try without torrent name prefix (single-file torrents)
                let source_alt = output_folder.join(&orig_rel);
                if source_alt.exists() {
                    let parent = source_alt.parent().unwrap_or(&output_folder);
                    let target = parent.join(new_name);
                    std::fs::rename(&source_alt, &target).map_err(|e| {
                        WhenThenError::Internal(format!("Failed to rename file: {e}"))
                    })?;
                }
                continue;
            }
            let parent = source.parent().unwrap_or(&output_folder);
            let target = parent.join(new_name);
            std::fs::rename(&source, &target).map_err(|e| {
                WhenThenError::Internal(format!("Failed to rename file: {e}"))
            })?;
        }
    }

    Ok(())
}

/// Requires delete + re-add to change file selection.
pub async fn update_torrent_files(
    state: &AppState,
    app_handle: &AppHandle,
    id: usize,
    only_files: Vec<usize>,
) -> Result<TorrentAddedResponse> {
    if only_files.is_empty() {
        return Err(WhenThenError::Torrent("Cannot deselect all files".into()));
    }

    let session = {
        let guard = state.torrent_session.read().await;
        guard.as_ref().ok_or_else(|| {
            WhenThenError::Torrent("Torrent session not initialized".into())
        })?.clone()
    };

    let handle = session
        .get(librqbit::api::TorrentIdOrHash::Id(id))
        .ok_or(WhenThenError::TorrentNotFound(id))?;

    let torrent_bytes = handle
        .with_metadata(|m| m.torrent_bytes.clone())
        .map_err(|e| WhenThenError::Torrent(format!("Cannot read torrent metadata: {e}")))?;

    let name = handle.name().unwrap_or_else(|| "Unknown".to_string());

    session
        .delete(librqbit::api::TorrentIdOrHash::Id(id), false)
        .await
        .map_err(|e| WhenThenError::Torrent(format!("Failed to delete torrent for file update: {e}")))?;

    state.torrent_names.write().await.remove(&id);

    let add_opts = AddTorrentOptions {
        only_files: Some(only_files.into_iter().collect()),
        overwrite: true,
        ..Default::default()
    };

    let response = session
        .add_torrent(
            AddTorrent::TorrentFileBytes(torrent_bytes),
            Some(add_opts),
        )
        .await
        .map_err(|e| WhenThenError::Torrent(format!("Failed to re-add torrent with new file selection: {e}")))?;

    let new_handle = match response {
        AddTorrentResponse::Added(_, h) => h,
        AddTorrentResponse::AlreadyManaged(_, h) => h,
        AddTorrentResponse::ListOnly(_) => {
            return Err(WhenThenError::Torrent("Torrent re-added in list-only mode".into()));
        }
    };

    let new_id = new_handle.id();
    let info_hash = new_handle.info_hash().as_string();

    state.torrent_names.write().await.insert(new_id, name.clone());

    let media_server_port = state.media_server.port;
    let local_ip = get_local_ip();
    let files = build_file_list(&new_handle, &local_ip, media_server_port);

    let result = TorrentAddedResponse {
        id: new_id,
        name: name.clone(),
        info_hash,
        files,
    };

    spawn_progress_emitter(state, app_handle.clone(), new_id);

    #[derive(serde::Serialize, Clone)]
    struct TorrentFilesUpdated {
        old_id: usize,
        new_id: usize,
        name: String,
    }

    app_handle
        .emit("torrent:files-updated", &TorrentFilesUpdated { old_id: id, new_id, name })
        .unwrap_or_default();

    info!(old_id = id, new_id, "Torrent file selection updated");

    Ok(result)
}

pub fn get_local_ip() -> String {
    local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string())
}
