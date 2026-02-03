// Watches folders for new .torrent files and auto-adds them.
use std::path::Path;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{mpsc, Mutex};
use tracing::{info, warn};

use crate::state::AppState;
use crate::services::torrent_engine;

#[derive(Clone, Serialize)]
pub struct FolderWatchEvent {
    pub path: String,
    pub torrent_id: usize,
    pub torrent_name: String,
}

pub struct FolderWatcherHandle {
    _watcher: RecommendedWatcher,
    shutdown_tx: mpsc::Sender<()>,
}

pub fn start_watching(
    folders: Vec<String>,
    app_handle: AppHandle,
) -> Option<FolderWatcherHandle> {
    if folders.is_empty() {
        return None;
    }

    let (event_tx, mut event_rx) = mpsc::channel::<String>(64);
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

    let event_tx_clone = event_tx.clone();
    let mut watcher = match RecommendedWatcher::new(
        move |result: Result<Event, notify::Error>| {
            if let Ok(event) = result {
                if matches!(event.kind, EventKind::Create(_)) {
                    for path in event.paths {
                        if path.extension().map(|e| e == "torrent").unwrap_or(false) {
                            let path_str = path.to_string_lossy().to_string();
                            let _ = event_tx_clone.try_send(path_str);
                        }
                    }
                }
            }
        },
        Config::default(),
    ) {
        Ok(w) => w,
        Err(e) => {
            warn!("Failed to create file watcher: {e}");
            return None;
        }
    };

    for folder in &folders {
        let path = Path::new(folder);
        if path.is_dir() {
            if let Err(e) = watcher.watch(path, RecursiveMode::NonRecursive) {
                warn!("Failed to watch folder {folder}: {e}");
            } else {
                info!("Watching folder: {folder}");
            }
        } else {
            warn!("Skipping non-existent folder: {folder}");
        }
    }

    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(path) = event_rx.recv() => {
                    // Debounce: wait for the file to finish writing
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                    let file_path = Path::new(&path);
                    if !file_path.exists() {
                        continue;
                    }

                    info!("Folder watch detected: {path}");
                    let state = app_handle.state::<AppState>();
                    match torrent_engine::add_torrent_file(&state, &app_handle, path.clone(), None).await {
                        Ok(result) => {
                            let event = FolderWatchEvent {
                                path,
                                torrent_id: result.id,
                                torrent_name: result.name.clone(),
                            };
                            app_handle
                                .emit("folder_watch:torrent_detected", &event)
                                .unwrap_or_default();
                            info!("Auto-added torrent from watched folder: {}", result.name);
                        }
                        Err(e) => {
                            warn!("Failed to add torrent from watched folder: {e}");
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Folder watcher shutting down");
                    break;
                }
            }
        }
    });

    Some(FolderWatcherHandle {
        _watcher: watcher,
        shutdown_tx,
    })
}

pub async fn stop_watching(handle: &Mutex<Option<FolderWatcherHandle>>) {
    if let Some(h) = handle.lock().await.take() {
        let _ = h.shutdown_tx.send(()).await;
        info!("Folder watcher stopped");
    }
}
