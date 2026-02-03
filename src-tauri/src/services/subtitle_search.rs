use std::path::PathBuf;

use tracing::info;

use crate::errors::{WhenThenError, Result};
use crate::models::SubtitleDownloadResult;
use crate::services::opensub_client;
use crate::state::AppState;

pub async fn search_and_download(
    state: &AppState,
    torrent_id: usize,
    file_index: usize,
    languages: Vec<String>,
) -> Result<SubtitleDownloadResult> {
    // Get API key from config
    let (api_key, download_dir) = {
        let cfg = state.config.read().await;
        (cfg.opensubtitles_api_key.clone(), cfg.download_directory.clone())
    };

    if api_key.is_empty() {
        return Err(WhenThenError::OpenSubtitles(
            "No OpenSubtitles API key configured. Set one in Settings.".to_string(),
        ));
    }

    // Get torrent handle and file info
    let session = {
        let guard = state.torrent_session.read().await;
        guard.as_ref().ok_or_else(|| {
            WhenThenError::Torrent("Torrent session not initialized".into())
        })?.clone()
    };

    let handle = session
        .get(librqbit::api::TorrentIdOrHash::Id(torrent_id))
        .ok_or(WhenThenError::TorrentNotFound(torrent_id))?;

    // Get file info from torrent metadata
    let file_info: Vec<(String, u64)> = handle.with_metadata(|meta| {
        meta.info.iter_file_details()
            .map(|iter| {
                iter.map(|fi| {
                    let path_str = fi.filename.to_string()
                        .unwrap_or_else(|_| "<INVALID NAME>".to_string());
                    (path_str, fi.len)
                }).collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }).map_err(|_| WhenThenError::Torrent("Failed to read torrent metadata".into()))?;

    let (file_path_str, _file_len) = file_info
        .get(file_index)
        .ok_or_else(|| WhenThenError::FileNotFound(format!("File index {} not found", file_index)))?;

    let torrent_name = handle.name().unwrap_or_else(|| "Unknown".to_string());

    // Build the absolute path to the video file
    let video_file_path = PathBuf::from(&download_dir).join(file_path_str);
    let video_file_name = video_file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&torrent_name)
        .to_string();

    let movie_hash = if video_file_path.exists() {
        opensub_client::compute_hash(&video_file_path)
    } else {
        None
    };

    info!(
        "Searching subtitles for '{}' (languages: {:?}, hash: {:?})",
        video_file_name, languages, movie_hash
    );

    // Search OpenSubtitles
    let results = opensub_client::search(
        &api_key,
        &languages,
        &video_file_name,
        movie_hash.as_deref(),
    )
    .await?;

    if results.is_empty() {
        return Err(WhenThenError::OpenSubtitles(format!(
            "No subtitles found for '{}'",
            video_file_name
        )));
    }

    // Pick best result: prefer hash matches (higher download_count usually), then highest rating
    let best = results
        .iter()
        .max_by(|a, b| {
            a.ratings
                .partial_cmp(&b.ratings)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.download_count.cmp(&b.download_count))
        })
        .unwrap(); // safe: we checked non-empty

    info!(
        "Selected subtitle: {} (language: {}, rating: {}, downloads: {})",
        best.file_name, best.language, best.ratings, best.download_count
    );

    // Download the subtitle file
    let (original_name, content) = opensub_client::download(&api_key, best.file_id).await?;

    // Determine output path alongside the video file
    let extension = original_name
        .rsplit('.')
        .next()
        .unwrap_or("srt");

    let subtitle_filename = format!("{}.{}.{}", video_file_name, best.language, extension);
    let output_dir = video_file_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(&download_dir));

    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir)
            .map_err(|e| WhenThenError::Internal(format!("Cannot create output dir: {e}")))?;
    }

    let output_path = output_dir.join(&subtitle_filename);
    std::fs::write(&output_path, &content)
        .map_err(|e| WhenThenError::Internal(format!("Failed to write subtitle file: {e}")))?;

    info!("Subtitle saved to: {}", output_path.display());

    Ok(SubtitleDownloadResult {
        file_name: subtitle_filename,
        file_path: output_path.to_string_lossy().to_string(),
    })
}
