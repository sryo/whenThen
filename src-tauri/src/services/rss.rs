// RSS sources, interests, and screener inbox.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use regex::Regex;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn};

use crate::errors::Result;
use crate::models::{
    BadItem, FeedFilter, FeedTestItem, FeedTestResult, FilterLogic, FilterType, Interest,
    PendingMatch, Source, TorrentFilePreview, TorrentMetadata,
};
use crate::services::torrent_engine;
use crate::state::AppState;

/// Check if a URL contains the {search} placeholder.
fn has_search_placeholder(url: &str) -> bool {
    url.contains("{search}")
}

/// Build a search URL by substituting {search} with the interest's search term.
fn build_search_url(url_template: &str, interest: &Interest) -> String {
    let term = interest
        .search_term
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(&interest.name);
    let encoded = urlencoding::encode(term);
    url_template.replace("{search}", &encoded)
}

pub struct RssServiceHandle {
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

impl RssServiceHandle {
    /// Stop the RSS polling service.
    pub fn stop(self) {
        let _ = self.shutdown_tx.send(());
    }
}

pub struct RssState {
    pub sources: Arc<RwLock<Vec<Source>>>,
    pub interests: Arc<RwLock<Vec<Interest>>>,
    /// Seen items: key -> ISO timestamp (for persistence and cleanup)
    pub seen_items: Arc<Mutex<HashMap<String, String>>>,
    /// Bad items: info_hash -> BadItem metadata
    pub bad_items: Arc<RwLock<HashMap<String, BadItem>>>,
    pub pending_matches: Arc<RwLock<Vec<PendingMatch>>>,
    pub service_handle: Arc<Mutex<Option<RssServiceHandle>>>,
}

impl RssState {
    pub fn new() -> Self {
        Self {
            sources: Arc::new(RwLock::new(Vec::new())),
            interests: Arc::new(RwLock::new(Vec::new())),
            seen_items: Arc::new(Mutex::new(HashMap::new())),
            bad_items: Arc::new(RwLock::new(HashMap::new())),
            pending_matches: Arc::new(RwLock::new(Vec::new())),
            service_handle: Arc::new(Mutex::new(None)),
        }
    }
}

/// Extract magnet link from text content.
fn extract_magnet_from_text(text: &str) -> Option<String> {
    // Find magnet:?xt= pattern
    if let Some(start) = text.find("magnet:?") {
        // Find the end of the magnet link (space, newline, < or end of string)
        let rest = &text[start..];
        let end = rest
            .find(|c: char| c.is_whitespace() || c == '<' || c == '"' || c == '\'')
            .unwrap_or(rest.len());
        return Some(rest[..end].to_string());
    }
    None
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

            // Check all links
            for link in &entry.links {
                if link.href.starts_with("magnet:") {
                    magnet_uri = Some(link.href.clone());
                } else if link.href.ends_with(".torrent") {
                    torrent_url = Some(link.href.clone());
                } else if link.rel.as_deref() == Some("enclosure") {
                    // Enclosure link - likely a torrent
                    if torrent_url.is_none() {
                        torrent_url = Some(link.href.clone());
                    }
                } else if link.media_type.as_deref() == Some("application/x-bittorrent") {
                    if torrent_url.is_none() {
                        torrent_url = Some(link.href.clone());
                    }
                }
            }

            // Check enclosure for magnet or torrent
            if let Some(media) = entry.media.first() {
                for content in &media.content {
                    if let Some(url) = &content.url {
                        let url_str = url.to_string();
                        if url_str.starts_with("magnet:") {
                            magnet_uri = Some(url_str);
                        } else if url_str.ends_with(".torrent") || torrent_url.is_none() {
                            torrent_url = Some(url_str);
                        }
                    }
                }
            }

            // Check content/summary for embedded magnet links
            if magnet_uri.is_none() {
                if let Some(content) = &entry.content {
                    if let Some(body) = &content.body {
                        if let Some(mag) = extract_magnet_from_text(body) {
                            magnet_uri = Some(mag);
                        }
                    }
                }
                if magnet_uri.is_none() {
                    if let Some(summary) = &entry.summary {
                        if let Some(mag) = extract_magnet_from_text(&summary.content) {
                            magnet_uri = Some(mag);
                        }
                    }
                }
            }

            // Fallback: if we have links but no torrent URL, use first non-html link
            if torrent_url.is_none() && magnet_uri.is_none() {
                for link in &entry.links {
                    let href = &link.href;
                    // Skip obvious webpage links
                    if !href.ends_with(".html") && !href.ends_with(".htm") && !href.contains("/wiki/") {
                        // Check if it looks like a download link
                        if href.contains("/download") || href.contains("/torrent/") || href.contains("get.php") {
                            torrent_url = Some(href.clone());
                            break;
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
        let mut last_check = std::time::Instant::now() - Duration::from_secs(3600); // Check immediately on startup

        loop {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    info!("RSS service shutting down");
                    break;
                }
                _ = interval.tick() => {
                    // Get check interval from settings
                    let state = handle.state::<crate::state::AppState>();
                    let interval_mins = state.config.read().await.rss_check_interval_minutes;
                    let interval_secs = (interval_mins as u64) * 60;

                    let now = std::time::Instant::now();
                    if now.duration_since(last_check).as_secs() < interval_secs {
                        continue;
                    }
                    last_check = now;

                    let sources = rss_state.sources.read().await.clone();
                    let interests = rss_state.interests.read().await.clone();

                    // Skip if no interests defined
                    let enabled_interests: Vec<_> = interests.iter().filter(|i| i.enabled).collect();
                    if enabled_interests.is_empty() {
                        continue;
                    }

                    for source in sources {
                        if !source.enabled {
                            continue;
                        }

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

                    // Persist seen items after checking all sources
                    crate::commands::rss::persist_seen_items(&handle, &state).await;
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
    let mut matched_count = 0;

    if has_search_placeholder(&source.url) {
        // Placeholder mode: fetch per interest with substituted search term
        for interest in interests {
            let url = build_search_url(&source.url, interest);
            info!("Fetching search URL for interest '{}': {}", interest.name, url);

            match fetch_feed(&url).await {
                Ok(items) => {
                    let count = process_items_for_interest(
                        app_handle,
                        rss_state,
                        source,
                        interest,
                        &items,
                        true, // use interest-specific seen key
                    )
                    .await;
                    matched_count += count;
                }
                Err(e) => {
                    warn!(
                        "Failed to fetch search feed for interest '{}': {}",
                        interest.name, e
                    );
                }
            }
        }
    } else {
        // Standard mode: fetch once, match all interests
        let items = fetch_feed(&source.url).await?;

        for item in &items {
            let mut seen = rss_state.seen_items.lock().await;
            let item_key = format!("{}:{}", source.id, item.id);

            if seen.contains_key(&item_key) {
                continue;
            }

            let now = Utc::now().to_rfc3339();
            if item.magnet_uri.is_none() && item.torrent_url.is_none() {
                seen.insert(item_key, now);
                continue;
            }

            // Check against all interests (first match wins)
            for interest in interests {
                let matched =
                    evaluate_filters_with_logic(item, &interest.filters, &interest.filter_logic);
                if matched.is_none() {
                    continue;
                }

                seen.insert(item_key.clone(), now.clone());
                drop(seen);

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

                rss_state
                    .pending_matches
                    .write()
                    .await
                    .push(pending.clone());
                matched_count += 1;

                let _ = app_handle.emit(
                    "rss:new-match",
                    serde_json::json!({
                        "id": pending.id,
                        "source_name": source.name,
                        "interest_name": interest.name,
                        "title": item.title,
                    }),
                );

                break;
            }
        }
    }

    let count = rss_state.pending_matches.read().await.len();
    let _ = app_handle.emit("rss:pending-count", count);

    Ok(matched_count)
}

/// Process feed items for a specific interest (used in placeholder mode).
async fn process_items_for_interest(
    app_handle: &AppHandle,
    rss_state: &RssState,
    source: &Source,
    interest: &Interest,
    items: &[ParsedFeedItem],
    use_interest_key: bool,
) -> usize {
    let mut matched_count = 0;

    for item in items {
        let mut seen = rss_state.seen_items.lock().await;

        // Use interest-specific key for placeholder mode (same item can match different searches)
        let item_key = if use_interest_key {
            format!("{}:{}:{}", source.id, interest.id, item.id)
        } else {
            format!("{}:{}", source.id, item.id)
        };

        if seen.contains_key(&item_key) {
            continue;
        }

        let now = Utc::now().to_rfc3339();
        if item.magnet_uri.is_none() && item.torrent_url.is_none() {
            seen.insert(item_key, now);
            continue;
        }

        let matched =
            evaluate_filters_with_logic(item, &interest.filters, &interest.filter_logic);
        if matched.is_none() {
            seen.insert(item_key, now);
            continue;
        }

        seen.insert(item_key, now);
        drop(seen);

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

        rss_state
            .pending_matches
            .write()
            .await
            .push(pending.clone());
        matched_count += 1;

        let _ = app_handle.emit(
            "rss:new-match",
            serde_json::json!({
                "id": pending.id,
                "source_name": source.name,
                "interest_name": interest.name,
                "title": item.title,
            }),
        );
    }

    matched_count
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
    info!("Approving match: {}", match_id);
    let state = app_handle.state::<AppState>();
    let rss_state = &state.rss_state;

    // Find and remove the pending match
    let pending = {
        let mut matches = rss_state.pending_matches.write().await;
        info!("Current pending matches: {}", matches.len());
        let idx = matches
            .iter()
            .position(|m| m.id == match_id)
            .ok_or_else(|| {
                warn!("Match not found: {}", match_id);
                crate::errors::WhenThenError::NotFound("Match not found".into())
            })?;
        matches.remove(idx)
    };

    info!(
        "Found match: title={}, magnet={:?}, torrent_url={:?}",
        pending.title,
        pending.magnet_uri.as_ref().map(|s| &s[..50.min(s.len())]),
        pending.torrent_url.as_ref().map(|s| &s[..50.min(s.len())])
    );

    // Get URI
    let uri = pending
        .magnet_uri
        .clone()
        .or(pending.torrent_url.clone())
        .ok_or_else(|| {
            warn!("No torrent URI for match: {}", pending.title);
            crate::errors::WhenThenError::InvalidInput("No torrent URI".into())
        })?;

    info!("Adding torrent from URI: {}...", &uri[..50.min(uri.len())]);

    // Add torrent
    let result = if uri.starts_with("magnet:") {
        torrent_engine::add_magnet(&state, app_handle, uri, None).await
    } else {
        let bytes = download_torrent_file(&uri).await?;
        torrent_engine::add_torrent_bytes(&state, app_handle, bytes, None).await
    };

    let response = result?;
    info!("Torrent added successfully: id={}", response.id);

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

/// Manually trigger an RSS check now.
pub async fn check_feeds_now(app_handle: &AppHandle) -> Result<usize> {
    let state = app_handle.state::<AppState>();
    let rss_state = &state.rss_state;

    let sources = rss_state.sources.read().await.clone();
    let interests = rss_state.interests.read().await.clone();

    let enabled_interests: Vec<_> = interests.iter().filter(|i| i.enabled).collect();
    if enabled_interests.is_empty() {
        info!("No enabled interests, skipping RSS check");
        return Ok(0);
    }

    let mut total_matched = 0;

    for source in sources {
        if !source.enabled {
            continue;
        }

        match check_source_for_matches(app_handle, rss_state, &source, &enabled_interests).await {
            Ok(count) => {
                total_matched += count;
                if count > 0 {
                    info!("Source {} matched {} new items", source.name, count);
                }
            }
            Err(e) => {
                warn!("Failed to check source {}: {}", source.name, e);
            }
        }
    }

    Ok(total_matched)
}

/// Re-check sources for a specific interest to find alternatives.
pub async fn recheck_interest(app_handle: &AppHandle, interest_id: &str) -> Result<usize> {
    let state = app_handle.state::<AppState>();
    let rss_state = &state.rss_state;

    let sources = rss_state.sources.read().await.clone();
    let interests = rss_state.interests.read().await.clone();

    let interest = interests
        .iter()
        .find(|i| i.id == interest_id)
        .ok_or_else(|| crate::errors::WhenThenError::NotFound("Interest not found".into()))?;

    if !interest.enabled {
        return Ok(0);
    }

    let interest_vec: Vec<&Interest> = vec![interest];
    let mut total_matched = 0;

    for source in sources {
        if !source.enabled {
            continue;
        }

        match check_source_for_matches(app_handle, rss_state, &source, &interest_vec).await {
            Ok(count) => {
                total_matched += count;
                if count > 0 {
                    info!("Found {} alternatives for interest '{}' from source '{}'", count, interest.name, source.name);
                }
            }
            Err(e) => {
                warn!("Failed to check source {} for alternatives: {}", source.name, e);
            }
        }
    }

    Ok(total_matched)
}
