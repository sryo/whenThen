use std::path::Path;

use serde::Serialize;
use tauri::State;

use crate::errors::Result;
use crate::models::{SubtitleInfo, SubtitleDownloadResult};
use crate::services::subtitle_handler;
use crate::services::subtitle_search;
use crate::services::torrent_engine::{get_local_ip, move_torrent_files as engine_move_files};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct MediaPlayer {
    pub id: String,
    pub name: String,
    pub path: String,
}

#[tauri::command]
pub async fn subtitle_load_file(
    state: State<'_, AppState>,
    path: String,
) -> Result<SubtitleInfo> {
    let data = subtitle_handler::load_subtitle_file(&path)?;

    let name = data.original_name.clone();
    let format = if path.ends_with(".srt") {
        "srt".to_string()
    } else {
        "vtt".to_string()
    };

    *state.current_subtitles.write().await = Some(data);

    let local_ip = get_local_ip();
    let port = state.media_server.port;
    let url = format!("http://{}:{}/subtitles.vtt", local_ip, port);

    Ok(SubtitleInfo { url, name, format })
}

#[tauri::command]
pub async fn subtitle_clear(state: State<'_, AppState>) -> Result<()> {
    *state.current_subtitles.write().await = None;
    Ok(())
}

#[tauri::command]
pub async fn media_server_url(state: State<'_, AppState>) -> Result<String> {
    let local_ip = get_local_ip();
    let port = state.media_server.port;
    Ok(format!("http://{}:{}", local_ip, port))
}

#[tauri::command]
pub async fn move_torrent_files(
    state: State<'_, AppState>,
    torrent_id: usize,
    destination: String,
) -> Result<()> {
    engine_move_files(&state, torrent_id, destination).await
}

#[tauri::command]
pub async fn subtitle_search_opensubtitles(
    state: State<'_, AppState>,
    torrent_id: usize,
    file_index: usize,
    languages: Vec<String>,
) -> Result<SubtitleDownloadResult> {
    subtitle_search::search_and_download(&state, torrent_id, file_index, languages).await
}

#[tauri::command]
pub async fn list_media_players() -> Result<Vec<MediaPlayer>> {
    let known_apps = [
        ("vlc", "VLC", "VLC.app"),
        ("iina", "IINA", "IINA.app"),
        ("mpv", "mpv", "mpv.app"),
        ("infuse", "Infuse", "Infuse 7.app"),
        ("elmedia", "Elmedia Player", "Elmedia Player.app"),
        ("quicktime", "QuickTime Player", "QuickTime Player.app"),
    ];

    let mut players = Vec::new();
    for (id, name, bundle) in &known_apps {
        let app_path = Path::new("/Applications").join(bundle);
        if app_path.exists() {
            players.push(MediaPlayer {
                id: id.to_string(),
                name: name.to_string(),
                path: app_path.to_string_lossy().to_string(),
            });
        }
    }

    Ok(players)
}
