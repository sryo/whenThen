mod commands;
mod dock;
mod errors;
mod models;
mod services;
mod state;
mod tray;

use std::sync::atomic::Ordering;

use models::AppConfig;
use services::media_server::MediaServerState;
use state::AppState;
#[cfg(any(target_os = "macos", target_os = "ios"))]
use tauri::Emitter;
use tauri::{Manager, RunEvent, WindowEvent};
use tracing::info;
use tracing_subscriber::EnvFilter;

/// Load saved config from tauri-plugin-store, falling back to defaults.
fn load_saved_config(app: &tauri::App) -> AppConfig {
    use tauri_plugin_store::StoreExt;
    const STORE_FILE: &str = "settings.json";
    const STORE_KEY: &str = "config";

    if let Ok(store) = app.store(STORE_FILE) {
        if let Some(value) = store.get(STORE_KEY) {
            if let Ok(config) = serde_json::from_value::<AppConfig>(value) {
                info!("Loaded saved config from store");
                return config;
            }
        }
    }
    info!("No saved config found, using defaults");
    AppConfig::default()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("when_then=info".parse().unwrap()))
        .init();

    // Start with defaults; saved config is loaded in setup() once the store is available
    let config = AppConfig::default();
    let app_state = AppState::new(config);

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // Focus main window when second instance is launched
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .manage(app_state)
        .setup(|app| {
            let saved_config = load_saved_config(app);
            let state = app.state::<AppState>();
            {
                let config = state.config.clone();
                tauri::async_runtime::block_on(async {
                    *config.write().await = saved_config;
                });
            }

            let torrent_session = state.torrent_session.clone();
            let config = state.config.clone();
            let media_server = state.media_server.clone();
            let current_subtitles = state.current_subtitles.clone();
            let local_file_tokens = state.local_file_tokens.clone();

            let app_data_dir = app.path().app_data_dir()
                .map_err(|e| {
                    tracing::error!("Failed to resolve app data dir: {e}");
                    e
                })?;
            let persistence_dir = app_data_dir.join("session");

            // Set up tray icon
            tray::setup(app.handle())?;

            // Set up macOS application menu
            #[cfg(target_os = "macos")]
            {
                use tauri::menu::{Menu, MenuItem, Submenu, PredefinedMenuItem};

                let app_handle = app.handle();
                let quit_item = MenuItem::with_id(app_handle, "quit", "Quit whenThen", true, Some("CmdOrCtrl+Q"))?;
                let hide_item = PredefinedMenuItem::hide(app_handle, Some("Hide whenThen"))?;
                let hide_others_item = PredefinedMenuItem::hide_others(app_handle, Some("Hide Others"))?;
                let show_all_item = PredefinedMenuItem::show_all(app_handle, Some("Show All"))?;
                let separator = PredefinedMenuItem::separator(app_handle)?;

                let app_submenu = Submenu::with_items(
                    app_handle,
                    "whenThen",
                    true,
                    &[&hide_item, &hide_others_item, &show_all_item, &separator, &quit_item],
                )?;

                // Edit menu for clipboard operations
                let undo_item = PredefinedMenuItem::undo(app_handle, Some("Undo"))?;
                let redo_item = PredefinedMenuItem::redo(app_handle, Some("Redo"))?;
                let cut_item = PredefinedMenuItem::cut(app_handle, Some("Cut"))?;
                let copy_item = PredefinedMenuItem::copy(app_handle, Some("Copy"))?;
                let paste_item = PredefinedMenuItem::paste(app_handle, Some("Paste"))?;
                let select_all_item = PredefinedMenuItem::select_all(app_handle, Some("Select All"))?;
                let edit_separator = PredefinedMenuItem::separator(app_handle)?;

                let edit_submenu = Submenu::with_items(
                    app_handle,
                    "Edit",
                    true,
                    &[&undo_item, &redo_item, &edit_separator, &cut_item, &copy_item, &paste_item, &select_all_item],
                )?;

                let menu = Menu::with_items(app_handle, &[&app_submenu, &edit_submenu])?;
                app.set_menu(menu)?;
            }

            // Close = hide main window (background mode)
            if let Some(main_window) = app.get_webview_window("main") {
                let handle = app.handle().clone();
                main_window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(win) = handle.get_webview_window("main") {
                            let _ = win.hide();
                        }
                    }
                });
            }

            // Close = hide picker window (reuse, don't destroy)
            if let Some(picker) = app.get_webview_window("picker") {
                let handle = app.handle().clone();
                picker.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(win) = handle.get_webview_window("picker") {
                            let _ = win.hide();
                        }
                    }
                });
            }

            let folder_watcher = state.folder_watcher.clone();
            let rss_state = state.rss_state.clone();
            let app_handle_for_watcher = app.handle().clone();
            let app_handle_for_rss = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                let cfg = config.read().await;
                let port = cfg.media_server_port;
                let cfg_snapshot = cfg.clone();
                drop(cfg);

                match services::torrent_engine::init_session(&cfg_snapshot, persistence_dir).await {
                    Ok(session) => {
                        *torrent_session.write().await = Some(session);
                        info!("Torrent session ready");
                    }
                    Err(e) => {
                        tracing::error!("Failed to init torrent session: {}", e);
                    }
                }

                let media_state = MediaServerState {
                    torrent_session: torrent_session.clone(),
                    current_subtitles,
                    local_file_tokens,
                };
                media_server.start(media_state).await;
                info!("Media server ready on port {}", port);

                // Start folder watcher if enabled
                if cfg_snapshot.watch_folders_enabled && !cfg_snapshot.watch_folders.is_empty() {
                    if let Some(handle) = services::folder_watcher::start_watching(
                        cfg_snapshot.watch_folders.clone(),
                        app_handle_for_watcher,
                    ) {
                        *folder_watcher.lock().await = Some(handle);
                    }
                }

                // Load persisted RSS sources and interests
                let rss_app_state = app_handle_for_rss.state::<AppState>();
                commands::rss::load_sources(&app_handle_for_rss, &rss_app_state).await;
                commands::rss::load_interests(&app_handle_for_rss, &rss_app_state).await;

                // Start RSS polling service
                let rss_handle = services::rss::start_service(app_handle_for_rss, rss_state.clone());
                *rss_state.service_handle.lock().await = Some(rss_handle);
                info!("RSS service ready");
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Torrent commands
            commands::torrent::torrent_add_magnet,
            commands::torrent::torrent_add_file,
            commands::torrent::torrent_add_bytes,
            commands::torrent::torrent_list,
            commands::torrent::torrent_details,
            commands::torrent::torrent_files,
            commands::torrent::torrent_pause,
            commands::torrent::torrent_resume,
            commands::torrent::torrent_delete,
            commands::torrent::torrent_recheck,
            commands::torrent::torrent_sync_restored,
            commands::torrent::torrent_update_files,
            // Chromecast commands
            commands::chromecast::chromecast_start_discovery,
            commands::chromecast::chromecast_stop_discovery,
            commands::chromecast::chromecast_list_devices,
            commands::chromecast::chromecast_connect,
            commands::chromecast::chromecast_disconnect,
            // Playback commands
            commands::playback::playback_cast_torrent,
            commands::playback::playback_cast_local_file,
            commands::playback::playback_open_in_app,
            commands::playback::playback_play,
            commands::playback::playback_pause,
            commands::playback::playback_stop,
            commands::playback::playback_seek,
            commands::playback::playback_seek_relative,
            commands::playback::playback_set_volume,
            commands::playback::playback_get_status,
            // Media commands
            commands::media::subtitle_load_file,
            commands::media::subtitle_clear,
            commands::media::media_server_url,
            commands::media::list_media_players,
            commands::media::move_torrent_files,
            commands::media::subtitle_search_opensubtitles,
            // Settings commands
            commands::settings::settings_get,
            commands::settings::settings_update,
            commands::settings::check_opened_via_url,
            // Automation commands
            commands::automation::check_automation_permission,
            commands::automation::run_shortcut,
            commands::automation::run_applescript,
            commands::automation::run_shell_command,
            // Rename command
            commands::torrent::torrent_rename_files,
            // Association commands
            commands::associations::check_file_associations,
            commands::associations::set_default_for_torrents,
            commands::associations::set_default_for_magnets,
            // RSS source commands
            commands::rss::rss_add_source,
            commands::rss::rss_update_source,
            commands::rss::rss_remove_source,
            commands::rss::rss_list_sources,
            commands::rss::rss_toggle_source,
            // RSS interest commands
            commands::rss::rss_add_interest,
            commands::rss::rss_update_interest,
            commands::rss::rss_remove_interest,
            commands::rss::rss_list_interests,
            commands::rss::rss_toggle_interest,
            commands::rss::rss_test_interest,
            // RSS screener commands
            commands::rss::rss_list_pending,
            commands::rss::rss_pending_count,
            commands::rss::rss_fetch_metadata,
            commands::rss::rss_approve_match,
            commands::rss::rss_reject_match,
        ])
        .build(tauri::generate_context!())
        .expect("error while building whenThen");

    // Register macOS menu event handler after build
    #[cfg(target_os = "macos")]
    {
        app.on_menu_event(|app_handle, event| {
            if event.id().as_ref() == "quit" {
                let state = app_handle.state::<AppState>();
                state.quit_requested.store(true, Ordering::SeqCst);
                app_handle.exit(0);
            }
        });
    }

    app.run(|app_handle, event| {
        match event {
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            RunEvent::Opened { urls } => {
                handle_opened_urls(app_handle, urls);
            }
            RunEvent::ExitRequested { api, .. } => {
                let state = app_handle.state::<AppState>();
                if state.quit_requested.load(Ordering::SeqCst) {
                    handle_shutdown(app_handle);
                } else {
                    api.prevent_exit();
                }
            }
            _ => {}
        }
    });
}

fn handle_shutdown(app_handle: &tauri::AppHandle) {
    let state = app_handle.state::<AppState>();

    let media_server = state.media_server.clone();
    let active_connections = state.active_connections.clone();
    let discovery_shutdown = state.discovery_shutdown.clone();
    let folder_watcher = state.folder_watcher.clone();

    tauri::async_runtime::block_on(async {
        // Stop folder watcher
        services::folder_watcher::stop_watching(&folder_watcher).await;

        // Stop media server
        media_server.stop().await;
        info!("Media server stopped");

        // Stop Chromecast discovery
        if let Some(tx) = discovery_shutdown.lock().await.take() {
            let _ = tx.send(());
            info!("Discovery stopped");
        }

        // Disconnect all Chromecast devices
        let mut connections = active_connections.lock().await;
        for (id, conn) in connections.drain() {
            conn.disconnect().await;
            info!("Disconnected Chromecast: {}", id);
        }
    });
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn handle_opened_urls(app_handle: &tauri::AppHandle, urls: Vec<tauri::Url>) {
    // Mark that the app was opened via file/URL so the frontend skips showing the main window.
    if !urls.is_empty() {
        let state = app_handle.state::<AppState>();
        state.opened_via_url.store(true, Ordering::SeqCst);
    }

    let app_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let state = app_handle.state::<AppState>();
        let mut retries = 0;
        loop {
            let guard = state.torrent_session.read().await;
            if guard.is_some() {
                break;
            }
            drop(guard);
            retries += 1;
            if retries > 30 {
                tracing::error!("Torrent session not ready after 15s, giving up on opened URLs");
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        for url in &urls {
            let result: crate::errors::Result<()> = match url.scheme() {
                "magnet" => {
                    let magnet_uri = url.to_string();
                    info!("Handling magnet link: {}", magnet_uri);
                    services::torrent_engine::add_magnet(
                        &state,
                        &app_handle,
                        magnet_uri,
                        None,
                    )
                    .await
                    .map(|_| ())
                }
                "file" => {
                    if let Ok(path) = url.to_file_path() {
                        let is_torrent = path.extension()
                            .map(|ext| ext == "torrent")
                            .unwrap_or(false);
                        if is_torrent {
                            let path_str = path.to_string_lossy().to_string();
                            info!("Handling torrent file: {}", path_str);
                            services::torrent_engine::add_torrent_file(
                                &state,
                                &app_handle,
                                path_str,
                                None,
                            )
                            .await
                            .map(|_| ())
                        } else {
                            Ok(())
                        }
                    } else {
                        Ok(())
                    }
                }
                _ => Ok(()),
            };

            if let Err(e) = result {
                tracing::error!("Failed to handle opened URL {}: {}", url, e);
                let _ = app_handle.emit("torrent:error", e.to_string());
            }
        }
    });
}
