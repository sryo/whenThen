// RSS sources, interests, and screener inbox.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use regex::Regex;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn};

use crate::errors::Result;
use crate::models::{
    FeedFilter, FeedTestItem, FeedTestResult, FilterLogic, FilterType, Interest, PendingMatch,
    Source, TorrentFilePreview, TorrentMetadata,
};
use crate::services::torrent_engine;
use crate::state::AppState;

pub struct RssServiceHandle {
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

impl RssServiceHandle {
    #[allow(dead_code)]
    pub fn stop(self) {
        let _ = self.shutdown_tx.send(());
    }
}

pub struct RssState {
    pub sources: Arc<RwLock<Vec<Source>>>,
    pub interests: Arc<RwLock<Vec<Interest>>>,
    pub seen_items: Arc<Mutex<HashSet<String>>>,
    pub pending_matches: Arc<RwLock<Vec<PendingMatch>>>,
    pub service_handle: Arc<Mutex<Option<RssServiceHandle>>>,
}

impl RssState {
    pub fn new() -> Self {
        Self {
            sources: Arc::new(RwLock::new(Vec::new())),
            interests: Arc::new(RwLock::new(Vec::new())),
            seen_items: Arc::new(Mutex::new(HashSet::new())),
            pending_matches: Arc::new(RwLock::new(Vec::new())),
            service_handle: Arc::new(Mutex::new(None)),
        }
    }
}

/// Fetch and parse an RSS feed from URL.
pub async fn fetch_feed(url: &str) -> Result<Vec<ParsedFeedItem>> {
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    let feed = feed_rs::parser::parse(&bytes[..])?;

    let items: Vec<ParsedFeedItem> = feed
        .entries
        .into_iter()
        .map(|entry| {
            let id = entry.id.clone();
            let title = entry.title.map(|t| t.content).unwrap_or_default();

            // Look for magnet URI in links or content
            let mut magnet_uri = None;
            let mut torrent_url = None;

            for link in &entry.links {
                if link.href.starts_with("magnet:") {
                    magnet_uri = Some(link.href.clone());
                } else if link.href.ends_with(".torrent") {
                    torrent_url = Some(link.href.clone());
                }
            }

            // Check enclosure for magnet or torrent
            if let Some(media) = entry.media.first() {
                for content in &media.content {
                    if let Some(url) = &content.url {
                        let url_str = url.to_string();
                        if url_str.starts_with("magnet:") {
                            magnet_uri = Some(url_str);
                        } else if url_str.ends_with(".torrent") {
                            torrent_url = Some(url_str);
                        }
                    }
                }
            }

            // Try to extract size from content or description
            let size = extract_size_from_title(&title);

            let published = entry.published.map(|d| d.to_rfc3339());

            ParsedFeedItem {
                id,
                title,
                magnet_uri,
                torrent_url,
                size,
                published_date: published,
            }
        })
        .collect();

    Ok(items)
}

#[derive(Debug, Clone)]
pub struct ParsedFeedItem {
    pub id: String,
    pub title: String,
    pub magnet_uri: Option<String>,
    pub torrent_url: Option<String>,
    pub size: Option<u64>,
    #[allow(dead_code)]
    pub published_date: Option<String>,
}

/// Extract size in bytes from title patterns like "1.5 GB" or "500 MB".
fn extract_size_from_title(title: &str) -> Option<u64> {
    let size_re = Regex::new(r"(\d+(?:\.\d+)?)\s*(GB|MB|KB|GiB|MiB|KiB)").ok()?;
    if let Some(caps) = size_re.captures(title) {
        let value: f64 = caps.get(1)?.as_str().parse().ok()?;
        let unit = caps.get(2)?.as_str();
        let multiplier = match unit {
            "KB" | "KiB" => 1024.0,
            "MB" | "MiB" => 1024.0 * 1024.0,
            "GB" | "GiB" => 1024.0 * 1024.0 * 1024.0,
            _ => 1.0,
        };
        return Some((value * multiplier) as u64);
    }
    None
}

/// Check if value matches with word boundaries (case-insensitive).
fn matches_word_boundary(title: &str, value: &str) -> bool {
    let pattern = format!(r"(?i)\b{}\b", regex::escape(value));
    Regex::new(&pattern)
        .map(|re| re.is_match(title))
        .unwrap_or(false)
}

/// Evaluate a single filter against a feed item.
fn evaluate_single_filter(item: &ParsedFeedItem, filter: &FeedFilter) -> bool {
    let title_lower = item.title.to_lowercase();

    match filter.filter_type {
        FilterType::MustContain => {
            let pattern = filter.value.to_lowercase();
            title_lower.contains(&pattern)
        }
        FilterType::MustNotContain => {
            let pattern = filter.value.to_lowercase();
            !title_lower.contains(&pattern)
        }
        FilterType::Regex => Regex::new(&filter.value)
            .map(|re| re.is_match(&item.title))
            .unwrap_or(false),
        FilterType::SizeRange => {
            if let Some(size) = item.size {
                let parts: Vec<&str> = filter.value.split('-').collect();
                if parts.len() == 2 {
                    let min_mb: u64 = parts[0].parse().unwrap_or(0);
                    let max_mb: u64 = parts[1].parse().unwrap_or(u64::MAX);
                    let size_mb = size / (1024 * 1024);
                    size_mb >= min_mb && size_mb <= max_mb
                } else {
                    true
                }
            } else {
                true // No size info = pass through
            }
        }
        FilterType::Episode => matches_word_boundary(&item.title, &filter.value),
        FilterType::Resolution => matches_word_boundary(&item.title, &filter.value),
        FilterType::Source => matches_word_boundary(&item.title, &filter.value),
        FilterType::Codec => matches_word_boundary(&item.title, &filter.value),
        FilterType::Audio => matches_word_boundary(&item.title, &filter.value),
        FilterType::Hdr => matches_word_boundary(&item.title, &filter.value),
    }
}

/// Evaluate filters against a feed item.
pub fn evaluate_filters(item: &ParsedFeedItem, filters: &[FeedFilter]) -> Option<String> {
    evaluate_filters_with_logic(item, filters, &FilterLogic::And)
}

/// Evaluate filters with specified and/or logic.
pub fn evaluate_filters_with_logic(
    item: &ParsedFeedItem,
    filters: &[FeedFilter],
    logic: &FilterLogic,
) -> Option<String> {
    let enabled_filters: Vec<_> = filters.iter().filter(|f| f.enabled).collect();
    if enabled_filters.is_empty() {
        return Some("no filters".to_string());
    }

    let results: Vec<bool> = enabled_filters
        .iter()
        .map(|f| evaluate_single_filter(item, f))
        .collect();

    let matches = match logic {
        FilterLogic::Or => results.iter().any(|&r| r),
        FilterLogic::And => results.iter().all(|&r| r),
    };

    if !matches {
        return None;
    }

    // Build matched filter description
    let desc: Vec<String> = enabled_filters
        .iter()
        .zip(results.iter())
        .filter_map(|(f, matched)| {
            if !matched {
                return None;
            }
            match f.filter_type {
                FilterType::MustContain => Some(format!("contains \"{}\"", f.value)),
                FilterType::MustNotContain => Some(format!("excludes \"{}\"", f.value)),
                FilterType::Regex => Some(format!("regex /{}/", f.value)),
                FilterType::Episode => Some(format!("episode \"{}\"", f.value)),
                FilterType::Resolution => Some(format!("resolution \"{}\"", f.value)),
                FilterType::Source => Some(format!("source \"{}\"", f.value)),
                FilterType::Codec => Some(format!("codec \"{}\"", f.value)),
                FilterType::Audio => Some(format!("audio \"{}\"", f.value)),
                FilterType::Hdr => Some(format!("HDR \"{}\"", f.value)),
                FilterType::SizeRange => Some(format!("size {}", f.value)),
            }
        })
        .collect();

    Some(desc.join(", "))
}

/// Test a feed URL with filters without downloading anything.
pub async fn test_feed(url: &str, filters: &[FeedFilter]) -> Result<FeedTestResult> {
    let items = fetch_feed(url).await?;
    let total_count = items.len();

    let test_items: Vec<FeedTestItem> = items
        .iter()
        .map(|item| {
            let matched_filter = evaluate_filters(item, filters);
            FeedTestItem {
                title: item.title.clone(),
                matches: matched_filter.is_some(),
                matched_filter,
                size: item.size,
            }
        })
        .collect();

    let matched_count = test_items.iter().filter(|i| i.matches).count();

    Ok(FeedTestResult {
        items: test_items,
        total_count,
        matched_count,
    })
}

/// Start the RSS polling service.
pub fn start_service(app_handle: AppHandle, rss_state: Arc<RssState>) -> RssServiceHandle {
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();

    let handle = app_handle.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        let mut last_checked: HashMap<String, std::time::Instant> = HashMap::new();

        loop {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    info!("RSS service shutting down");
                    break;
                }
                _ = interval.tick() => {
                    let sources = rss_state.sources.read().await.clone();
                    let interests = rss_state.interests.read().await.clone();
                    let now = std::time::Instant::now();

                    // Skip if no interests defined
                    let enabled_interests: Vec<_> = interests.iter().filter(|i| i.enabled).collect();
                    if enabled_interests.is_empty() {
                        continue;
                    }

                    for source in sources {
                        if !source.enabled {
                            continue;
                        }

                        let interval_secs = (source.check_interval_minutes as u64) * 60;
                        let should_check = last_checked
                            .get(&source.id)
                            .map(|t| now.duration_since(*t).as_secs() >= interval_secs)
                            .unwrap_or(true);

                        if !should_check {
                            continue;
                        }

                        last_checked.insert(source.id.clone(), now);

                        match check_source_for_matches(&handle, &rss_state, &source, &enabled_interests).await {
                            Ok(count) => {
                                if count > 0 {
                                    info!("Source {} queued {} new items for screening", source.name, count);
                                }
                            }
                            Err(e) => {
                                warn!("Failed to check source {}: {}", source.name, e);
                            }
                        }
                    }
                }
            }
        }
    });

    RssServiceHandle { shutdown_tx }
}

/// Check a source against all interests and queue matches for screening.
async fn check_source_for_matches(
    app_handle: &AppHandle,
    rss_state: &RssState,
    source: &Source,
    interests: &[&Interest],
) -> Result<usize> {
    let items = fetch_feed(&source.url).await?;
    let mut seen = rss_state.seen_items.lock().await;
    let mut matched_count = 0;

    for item in items {
        // Skip already seen items
        let item_key = format!("{}:{}", source.id, item.id);
        if seen.contains(&item_key) {
            continue;
        }

        // Has magnet or torrent URL?
        if item.magnet_uri.is_none() && item.torrent_url.is_none() {
            seen.insert(item_key);
            continue;
        }

        // Check item against all interests
        for interest in interests {
            let matched = evaluate_filters_with_logic(&item, &interest.filters, &interest.filter_logic);
            if matched.is_none() {
                continue;
            }

            // Found a match!
            seen.insert(item_key.clone());

            // Create pending match
            let pending = PendingMatch {
                id: uuid::Uuid::new_v4().to_string(),
                source_id: source.id.clone(),
                source_name: source.name.clone(),
                interest_id: interest.id.clone(),
                interest_name: interest.name.clone(),
                title: item.title.clone(),
                magnet_uri: item.magnet_uri.clone(),
                torrent_url: item.torrent_url.clone(),
                created_at: Utc::now().to_rfc3339(),
                metadata: None,
            };

            // Add to pending queue
            rss_state.pending_matches.write().await.push(pending.clone());
            matched_count += 1;

            // Emit event for notification
            let _ = app_handle.emit(
                "rss:new-match",
                serde_json::json!({
                    "id": pending.id,
                    "source_name": source.name,
                    "interest_name": interest.name,
                    "title": item.title,
                }),
            );

            // Only match once per item (first interest wins)
            break;
        }
    }

    // Emit pending count update
    let count = rss_state.pending_matches.read().await.len();
    let _ = app_handle.emit("rss:pending-count", count);

    Ok(matched_count)
}

/// Fetch torrent metadata for screening preview.
pub async fn fetch_metadata(app_handle: &AppHandle, match_id: &str) -> Result<TorrentMetadata> {
    let state = app_handle.state::<AppState>();
    let rss_state = &state.rss_state;

    // Find the pending match
    let pending = {
        let matches = rss_state.pending_matches.read().await;
        matches.iter().find(|m| m.id == match_id).cloned()
    };

    let pending = pending.ok_or_else(|| crate::errors::WhenThenError::NotFound("Match not found".into()))?;

    // Get URI
    let uri = pending
        .magnet_uri
        .clone()
        .or(pending.torrent_url.clone())
        .ok_or_else(|| crate::errors::WhenThenError::InvalidInput("No torrent URI".into()))?;

    // Add torrent paused to get metadata, then delete it
    let add_torrent = if uri.starts_with("magnet:") {
        librqbit::AddTorrent::from_url(&uri)
    } else {
        let bytes = download_torrent_file(&uri).await?;
        librqbit::AddTorrent::TorrentFileBytes(bytes.into())
    };

    let metadata = fetch_torrent_metadata_via_session(&state, add_torrent).await?;

    // Update the pending match with metadata
    {
        let mut matches = rss_state.pending_matches.write().await;
        if let Some(m) = matches.iter_mut().find(|m| m.id == match_id) {
            m.metadata = Some(metadata.clone());
        }
    }

    Ok(metadata)
}

/// Fetch metadata by adding torrent paused, reading info, then deleting.
async fn fetch_torrent_metadata_via_session(
    state: &AppState,
    add_torrent: librqbit::AddTorrent<'_>,
) -> Result<TorrentMetadata> {
    let session_guard = state.torrent_session.read().await;
    let session = session_guard
        .as_ref()
        .ok_or_else(|| crate::errors::WhenThenError::Internal("Torrent session not ready".into()))?;

    let add_opts = librqbit::AddTorrentOptions {
        paused: true,
        ..Default::default()
    };

    let response = session
        .add_torrent(add_torrent, Some(add_opts))
        .await
        .map_err(|e| crate::errors::WhenThenError::Torrent(e.to_string()))?;

    let handle = match response {
        librqbit::AddTorrentResponse::Added(_, h) => h,
        librqbit::AddTorrentResponse::AlreadyManaged(_, h) => h,
        librqbit::AddTorrentResponse::ListOnly(_) => {
            return Err(crate::errors::WhenThenError::Torrent("List-only mode".into()));
        }
    };

    // Wait for metadata (with timeout)
    let metadata_result = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            // Check if we have metadata
            let has_meta = handle.with_metadata(|_| ()).is_ok();
            if has_meta {
                break;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    })
    .await;

    // Get file info from handle
    let file_infos: Vec<(String, u64)> = handle
        .with_metadata(|meta| {
            meta.info
                .iter_file_details()
                .map(|iter| {
                    iter.map(|fi| {
                        let name = fi.filename.to_string().unwrap_or_else(|_| "<invalid>".into());
                        (name, fi.len)
                    })
                    .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .unwrap_or_default();

    let torrent_name = handle.name().unwrap_or_else(|| "Unknown".to_string());

    // Delete the paused torrent
    let torrent_id = handle.id();
    let _ = session
        .delete(librqbit::api::TorrentIdOrHash::Id(torrent_id), false)
        .await;

    // Check if metadata fetch timed out
    if metadata_result.is_err() && file_infos.is_empty() {
        return Err(crate::errors::WhenThenError::Torrent(
            "Metadata fetch timed out".into(),
        ));
    }

    // Build metadata
    let files: Vec<TorrentFilePreview> = file_infos
        .into_iter()
        .map(|(name, size)| {
            let is_video = is_video_file(&name);
            let is_suspicious = is_suspicious_file(&name);
            TorrentFilePreview {
                name,
                size,
                is_video,
                is_suspicious,
            }
        })
        .collect();

    let total_size = files.iter().map(|f| f.size).sum();
    let file_count = files.len();

    Ok(TorrentMetadata {
        name: torrent_name,
        total_size,
        file_count,
        files,
    })
}

/// Check if a file is a video based on extension.
fn is_video_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".mkv")
        || lower.ends_with(".mp4")
        || lower.ends_with(".avi")
        || lower.ends_with(".mov")
        || lower.ends_with(".wmv")
        || lower.ends_with(".webm")
        || lower.ends_with(".m4v")
        || lower.ends_with(".ts")
}

/// Check if a file looks suspicious (potential malware).
fn is_suspicious_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".exe")
        || lower.ends_with(".msi")
        || lower.ends_with(".bat")
        || lower.ends_with(".cmd")
        || lower.ends_with(".scr")
        || lower.ends_with(".vbs")
        || lower.ends_with(".js")
        || lower.ends_with(".jar")
        || lower.ends_with(".ps1")
        || lower.ends_with(".dll")
}

/// Download a .torrent file from URL.
async fn download_torrent_file(url: &str) -> Result<Vec<u8>> {
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}

/// Approve a pending match and start the download.
pub async fn approve_match(app_handle: &AppHandle, match_id: &str) -> Result<i64> {
    let state = app_handle.state::<AppState>();
    let rss_state = &state.rss_state;

    // Find and remove the pending match
    let pending = {
        let mut matches = rss_state.pending_matches.write().await;
        let idx = matches
            .iter()
            .position(|m| m.id == match_id)
            .ok_or_else(|| crate::errors::WhenThenError::NotFound("Match not found".into()))?;
        matches.remove(idx)
    };

    // Get URI
    let uri = pending
        .magnet_uri
        .clone()
        .or(pending.torrent_url.clone())
        .ok_or_else(|| crate::errors::WhenThenError::InvalidInput("No torrent URI".into()))?;

    // Add torrent
    let result = if uri.starts_with("magnet:") {
        torrent_engine::add_magnet(&state, app_handle, uri, None).await
    } else {
        let bytes = download_torrent_file(&uri).await?;
        torrent_engine::add_torrent_bytes(&state, app_handle, bytes, None).await
    };

    let response = result?;

    // Emit pending count update
    let count = rss_state.pending_matches.read().await.len();
    let _ = app_handle.emit("rss:pending-count", count);

    Ok(response.id as i64)
}

/// Reject a pending match (discard it).
pub async fn reject_match(app_handle: &AppHandle, match_id: &str) -> Result<()> {
    let state = app_handle.state::<AppState>();
    let rss_state = &state.rss_state;

    let mut matches = rss_state.pending_matches.write().await;
    matches.retain(|m| m.id != match_id);

    // Emit pending count update
    let count = matches.len();
    let _ = app_handle.emit("rss:pending-count", count);

    Ok(())
}
