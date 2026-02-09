use std::path::PathBuf;

use tracing::info;

use crate::errors::{WhenThenError, Result};
use crate::models::SubtitleDownloadResult;
use crate::services::{media_info, opensub_client, subtitle_scorer, torrent_engine::expand_path};
use crate::state::AppState;

pub async fn search_and_download(
    state: &AppState,
    torrent_id: usize,
    file_index: usize,
    languages: Vec<String>,
) -> Result<SubtitleDownloadResult> {
    // Get API key and base directory from config
    let (api_key, download_dir) = {
        let cfg = state.config.read().await;
        (cfg.opensubtitles_api_key.clone(), cfg.download_directory.clone())
    };

    // Check if torrent was moved to a different location
    let moved_location = state.torrent_locations.read().await.get(&torrent_id).cloned();


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

    // Build the absolute path to the video file, checking moved location first
    let video_file_path = if let Some(ref loc) = moved_location {
        // Files were moved - check the new location
        let moved_path = PathBuf::from(loc);
        // Try with torrent folder name (multi-file torrents)
        let with_name = moved_path.join(&torrent_name).join(file_path_str);
        if with_name.exists() {
            with_name
        } else {
            // Try direct path (single-file or flat structure)
            let file_name = PathBuf::from(file_path_str).file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| file_path_str.clone());
            moved_path.join(&torrent_name).join(&file_name)
        }
    } else {
        // Use default download directory
        expand_path(&download_dir).join(file_path_str)
    };
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

    // Parse video file metadata for scoring
    let video_info = media_info::parse(&video_file_name);

    // OpenSubtitles requires an API key
    if api_key.is_empty() {
        return Err(WhenThenError::OpenSubtitles(
            "OpenSubtitles API key not configured. Add your API key in Settings to enable subtitle search.".into()
        ));
    }

    let (original_name, content, selected_lang) = search_opensubtitles(
        &api_key,
        &languages,
        &video_file_name,
        movie_hash.as_deref(),
        &video_info,
    ).await?;

    // Determine output path alongside the video file
    let extension = original_name
        .rsplit('.')
        .next()
        .unwrap_or("srt");

    let subtitle_filename = format!("{}.{}.{}", video_file_name, selected_lang, extension);
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

/// Search OpenSubtitles and return best match using scoring.
async fn search_opensubtitles(
    api_key: &str,
    languages: &[String],
    video_file_name: &str,
    movie_hash: Option<&str>,
    video_info: &crate::models::MediaInfo,
) -> Result<(String, Vec<u8>, String)> {
    let results = opensub_client::search(api_key, languages, video_file_name, movie_hash).await?;

    if results.is_empty() {
        return Err(WhenThenError::OpenSubtitles(format!(
            "No subtitles found for '{}'",
            video_file_name
        )));
    }

    // Score each result and pick the best
    let mut scored: Vec<_> = results
        .iter()
        .map(|r| {
            let sub_info = media_info::parse(&r.file_name);
            let score = subtitle_scorer::score_infos(video_info, &sub_info);
            (r, score)
        })
        .collect();

    // Sort by score descending, then by download count as tiebreaker
    scored.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(b.0.download_count.cmp(&a.0.download_count))
    });

    let best = scored[0].0;
    info!(
        "Selected subtitle: {} (language: {}, score: {:.2}, downloads: {})",
        best.file_name, best.language, scored[0].1, best.download_count
    );

    let (original_name, content) = opensub_client::download(api_key, best.file_id).await?;
    Ok((original_name, content, best.language.clone()))
}
