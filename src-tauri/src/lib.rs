mod commands;
mod dock;
mod errors;
mod i18n;
mod models;
#[cfg(target_os = "macos")]
mod move_to_applications;
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
use serde_json::Value;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tauri::command]
fn get_translations(locale: Option<String>) -> Value {
    i18n::get_translations_for_locale(locale)
}

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
            // On macOS, prompt to move to /Applications if running from elsewhere
            #[cfg(target_os = "macos")]
            if move_to_applications::check_and_prompt(app) {
                // App was moved and relaunched from /Applications; this process will exit
                return Ok(());
            }

            let saved_config = load_saved_config(app);
            let state = app.state::<AppState>();
            {
                let config = state.config.clone();
                tauri::async_runtime::block_on(async {
                    *config.write().await = saved_config;
                });
            }

            // Initialize i18n
            if let Err(e) = i18n::init(app) {
                tracing::error!("Failed to initialize i18n: {}", e);
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
                use crate::i18n::t;

                let h = app.handle();

                // App menu
                let about_item = PredefinedMenuItem::about(h, Some(&t("menu.about")), None)?;
                let settings_item = MenuItem::with_id(h, "settings", &t("menu.settings"), true, Some("CmdOrCtrl+,"))?;
                let hide_item = PredefinedMenuItem::hide(h, Some(&t("menu.hide")))?;
                let hide_others_item = PredefinedMenuItem::hide_others(h, Some(&t("menu.hideOthers")))?;
                let show_all_item = PredefinedMenuItem::show_all(h, Some(&t("menu.showAll")))?;
                let quit_item = MenuItem::with_id(h, "quit", &t("menu.quit"), true, Some("CmdOrCtrl+Q"))?;

                let app_submenu = Submenu::with_items(
                    h,
                    "When",
                    true,
                    &[
                        &about_item,
                        &PredefinedMenuItem::separator(h)?,
                        &settings_item,
                        &PredefinedMenuItem::separator(h)?,
                        &hide_item,
                        &hide_others_item,
                        &show_all_item,
                        &PredefinedMenuItem::separator(h)?,
                        &quit_item,
                    ],
                )?;

                // File menu
                let add_torrent_item = MenuItem::with_id(h, "add-torrent", &t("menu.addTorrent"), true, Some("CmdOrCtrl+O"))?;
                let add_magnet_item = MenuItem::with_id(h, "add-magnet", &t("menu.addMagnet"), true, Some("CmdOrCtrl+U"))?;
                let check_feeds_item = MenuItem::with_id(h, "check-feeds", &t("menu.checkFeeds"), true, Some("CmdOrCtrl+R"))?;

                let file_submenu = Submenu::with_items(
                    h,
                    &t("menu.file"),
                    true,
                    &[
                        &add_torrent_item,
                        &add_magnet_item,
                        &PredefinedMenuItem::separator(h)?,
                        &check_feeds_item,
                    ],
                )?;

                // Edit menu
                let undo_item = PredefinedMenuItem::undo(h, Some(&t("menu.undo")))?;
                let redo_item = PredefinedMenuItem::redo(h, Some(&t("menu.redo")))?;
                let cut_item = PredefinedMenuItem::cut(h, Some(&t("menu.cut")))?;
                let copy_item = PredefinedMenuItem::copy(h, Some(&t("menu.copy")))?;
                let paste_item = PredefinedMenuItem::paste(h, Some(&t("menu.paste")))?;
                let select_all_item = PredefinedMenuItem::select_all(h, Some(&t("menu.selectAll")))?;

                let edit_submenu = Submenu::with_items(
                    h,
                    &t("menu.edit"),
                    true,
                    &[
                        &undo_item,
                        &redo_item,
                        &PredefinedMenuItem::separator(h)?,
                        &cut_item,
                        &copy_item,
                        &paste_item,
                        &select_all_item,
                    ],
                )?;

                // View menu
                let view_inbox_item = MenuItem::with_id(h, "view-inbox", &t("menu.inbox"), true, Some("CmdOrCtrl+1"))?;
                let view_playlets_item = MenuItem::with_id(h, "view-playlets", &t("menu.playlets"), true, Some("CmdOrCtrl+2"))?;
                let view_settings_item = MenuItem::with_id(h, "view-settings", &t("nav.settings"), true, Some("CmdOrCtrl+3"))?;

                let view_submenu = Submenu::with_items(
                    h,
                    &t("menu.view"),
                    true,
                    &[&view_inbox_item, &view_playlets_item, &view_settings_item],
                )?;

                // Torrents menu
                let pause_all_item = MenuItem::with_id(h, "pause-all", &t("menu.pauseAll"), true, None::<&str>)?;
                let resume_all_item = MenuItem::with_id(h, "resume-all", &t("menu.resumeAll"), true, None::<&str>)?;
                let clear_completed_item = MenuItem::with_id(h, "clear-completed", &t("menu.clearCompleted"), true, None::<&str>)?;

                let torrents_submenu = Submenu::with_items(
                    h,
                    &t("menu.torrents"),
                    true,
                    &[
                        &pause_all_item,
                        &resume_all_item,
                        &PredefinedMenuItem::separator(h)?,
                        &clear_completed_item,
                    ],
                )?;

                // Window menu
                let minimize_item = PredefinedMenuItem::minimize(h, Some(&t("menu.minimize")))?;

                let window_submenu = Submenu::with_items(
                    h,
                    &t("menu.window"),
                    true,
                    &[&minimize_item],
                )?;

                // Help menu
                let help_docs_item = MenuItem::with_id(h, "help-docs", &t("menu.helpDocs"), true, None::<&str>)?;

                let help_submenu = Submenu::with_items(
                    h,
                    &t("menu.help"),
                    true,
                    &[&help_docs_item],
                )?;

                let menu = Menu::with_items(
                    h,
                    &[
                        &app_submenu,
                        &file_submenu,
                        &edit_submenu,
                        &view_submenu,
                        &torrents_submenu,
                        &window_submenu,
                        &help_submenu,
                    ],
                )?;
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

                // Load persisted RSS sources, interests, seen items, and bad items
                let rss_app_state = app_handle_for_rss.state::<AppState>();
                commands::rss::load_sources(&app_handle_for_rss, &rss_app_state).await;
                commands::rss::load_interests(&app_handle_for_rss, &rss_app_state).await;
                commands::rss::load_seen_items(&app_handle_for_rss, &rss_app_state).await;
                commands::rss::load_bad_items(&app_handle_for_rss, &rss_app_state).await;

                // Check for demo mode (marker file in app support directory)
                let demo_marker = app_handle_for_rss.path().app_data_dir()
                    .map(|d| d.join("demo_mode"))
                    .ok();
                if let Some(marker) = demo_marker {
                    if marker.exists() {
                        info!("Demo mode detected, seeding demo data");
                        if let Err(e) = commands::rss::seed_demo_pending(&rss_app_state).await {
                            tracing::warn!("Failed to seed demo data: {}", e);
                        }
                    }
                }

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
            commands::media::get_playlist_url,
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
            commands::rss::rss_check_now,
            // RSS bad items commands
            commands::rss::rss_mark_bad,
            commands::rss::rss_unmark_bad,
            commands::rss::rss_list_bad,
            // RSS demo data
            commands::rss::rss_seed_demo,
            // Scraper commands
            commands::scraper::scraper_add_config,
            commands::scraper::scraper_update_config,
            commands::scraper::scraper_remove_config,
            commands::scraper::scraper_list_configs,
            commands::scraper::scraper_toggle,
            commands::scraper::scraper_test,
            // i18n commands
            get_translations,
        ])
        .build(tauri::generate_context!())
        .expect("error while building When");

    // Register macOS menu event handler after build
    #[cfg(target_os = "macos")]
    {
        use tauri::Emitter;

        app.on_menu_event(|app_handle, event| {
            let id = event.id().as_ref();
            match id {
                "quit" => {
                    let state = app_handle.state::<AppState>();
                    state.quit_requested.store(true, Ordering::SeqCst);
                    app_handle.exit(0);
                }
                "settings" => {
                    let _ = app_handle.emit("menu:navigate", "settings");
                }
                "add-torrent" => {
                    // Open file dialog and add torrent
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        use tauri_plugin_dialog::DialogExt;
                        let file = handle.dialog()
                            .file()
                            .add_filter("Torrent Files", &["torrent"])
                            .blocking_pick_file();
                        if let Some(path) = file {
                            let state = handle.state::<AppState>();
                            if let Some(path_str) = path.as_path().map(|p| p.to_string_lossy().to_string()) {
                                match services::torrent_engine::add_torrent_file(&state, &handle, path_str, None).await {
                                    Ok(_) => info!("Added torrent from menu"),
                                    Err(e) => {
                                        tracing::error!("Failed to add torrent: {}", e);
                                        let _ = handle.emit("torrent:error", e.to_string());
                                    }
                                }
                            }
                        }
                    });
                }
                "add-magnet" => {
                    let _ = app_handle.emit("menu:add-magnet", ());
                }
                "check-feeds" => {
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = services::rss::check_feeds_now(&handle).await {
                            tracing::error!("Failed to check feeds: {}", e);
                        }
                    });
                }
                "view-inbox" => {
                    let _ = app_handle.emit("menu:navigate", "inbox");
                }
                "view-playlets" => {
                    let _ = app_handle.emit("menu:navigate", "rules");
                }
                "view-settings" => {
                    let _ = app_handle.emit("menu:navigate", "settings");
                }
                "pause-all" => {
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let state = handle.state::<AppState>();
                        let session = {
                            let guard = state.torrent_session.read().await;
                            match guard.as_ref() {
                                Some(s) => s.clone(),
                                None => return,
                            }
                        };
                        let torrents: Vec<_> = session.with_torrents(|iter| {
                            iter.map(|(id, h)| (id, h.clone())).collect()
                        });
                        for (_id, torrent_handle) in torrents {
                            let _ = session.pause(&torrent_handle).await;
                        }
                        let _ = handle.emit("torrents:changed", ());
                    });
                }
                "resume-all" => {
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let state = handle.state::<AppState>();
                        let session = {
                            let guard = state.torrent_session.read().await;
                            match guard.as_ref() {
                                Some(s) => s.clone(),
                                None => return,
                            }
                        };
                        let torrents: Vec<_> = session.with_torrents(|iter| {
                            iter.map(|(id, h)| (id, h.clone())).collect()
                        });
                        for (_id, torrent_handle) in torrents {
                            let _ = session.unpause(&torrent_handle).await;
                        }
                        let _ = handle.emit("torrents:changed", ());
                    });
                }
                "clear-completed" => {
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let state = handle.state::<AppState>();
                        let session = {
                            let guard = state.torrent_session.read().await;
                            match guard.as_ref() {
                                Some(s) => s.clone(),
                                None => return,
                            }
                        };
                        let completed_ids: Vec<usize> = session.with_torrents(|iter| {
                            iter.filter(|(_id, h)| h.stats().finished)
                                .map(|(id, _h)| id)
                                .collect()
                        });
                        for id in completed_ids {
                            let _ = services::torrent_engine::delete_torrent(&state, id, false).await;
                        }
                        let _ = handle.emit("torrents:changed", ());
                    });
                }
                "help-docs" => {
                    use tauri_plugin_shell::ShellExt;
                    let _ = app_handle.shell().open("https://whenthen.app/docs", None);
                }
                _ => {}
            }
        });
    }

    app.run(|app_handle, event| {
        match event {
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            RunEvent::Opened { urls } => {
                handle_opened_urls(app_handle, urls);
            }
            #[cfg(target_os = "macos")]
            RunEvent::Reopen { .. } => {
                // Dock icon clicked - show main window
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
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
                    let magnet_uri = url.as_str().to_string();
                    info!("Handling magnet link: {}", magnet_uri);

                    // Timeout prevents hanging on magnets with no peers/metadata
                    let add_result = tokio::time::timeout(
                        std::time::Duration::from_secs(30),
                        services::torrent_engine::add_magnet(
                            &state,
                            &app_handle,
                            magnet_uri.clone(),
                            None,
                        )
                    ).await;

                    match add_result {
                        Ok(Ok(_)) => {
                            info!("Magnet added successfully");
                            Ok(())
                        }
                        Ok(Err(e)) => {
                            tracing::error!("Failed to add magnet: {:?}", e);
                            Err(e)
                        }
                        Err(_) => {
                            tracing::error!("Timeout adding magnet after 30s");
                            Err(crate::errors::WhenThenError::Torrent("Timeout adding magnet".into()))
                        }
                    }
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
