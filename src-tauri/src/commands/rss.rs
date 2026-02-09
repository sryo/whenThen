// RSS Tauri commands for sources, interests, and screener.

use tauri::State;
use tauri_plugin_store::StoreExt;

use crate::errors::Result;
use crate::models::{BadItem, FeedFilter, FeedTestResult, Interest, PendingMatch, Source, TorrentFilePreview, TorrentMetadata};
use crate::services::rss;
use crate::state::AppState;

const SOURCES_STORE: &str = "sources.json";
const INTERESTS_STORE: &str = "interests.json";
const SEEN_ITEMS_STORE: &str = "seen_items.json";
const BAD_ITEMS_STORE: &str = "bad_items.json";

/// Max age for seen items before cleanup (60 days in seconds).
const SEEN_ITEMS_MAX_AGE_SECS: i64 = 60 * 24 * 60 * 60;

async fn persist_sources(app: &tauri::AppHandle, state: &AppState) {
    if let Ok(store) = app.store(SOURCES_STORE) {
        let sources = state.rss_state.sources.read().await;
        if let Ok(value) = serde_json::to_value(&*sources) {
            store.set("sources", value);
            if let Err(e) = store.save() {
                tracing::error!("Failed to save RSS sources: {}", e);
            }
        }
    }
}

/// Internal version callable from rss service.
pub async fn persist_sources_internal(app: &tauri::AppHandle, state: &AppState) {
    persist_sources(app, state).await;
}

async fn persist_interests(app: &tauri::AppHandle, state: &AppState) {
    if let Ok(store) = app.store(INTERESTS_STORE) {
        let interests = state.rss_state.interests.read().await;
        if let Ok(value) = serde_json::to_value(&*interests) {
            store.set("interests", value);
            if let Err(e) = store.save() {
                tracing::error!("Failed to save RSS interests: {}", e);
            }
        }
    }
}

pub async fn load_sources(app: &tauri::AppHandle, state: &AppState) {
    if let Ok(store) = app.store(SOURCES_STORE) {
        // Load store contents from disk file before reading
        if let Err(e) = store.reload() {
            tracing::warn!("Could not load sources store: {}", e);
        }
        if let Some(value) = store.get("sources") {
            if let Ok(sources) = serde_json::from_value::<Vec<Source>>(value) {
                tracing::info!("Loaded {} RSS sources from disk", sources.len());
                *state.rss_state.sources.write().await = sources;
            }
        }
    }
}

pub async fn load_interests(app: &tauri::AppHandle, state: &AppState) {
    if let Ok(store) = app.store(INTERESTS_STORE) {
        // Load store contents from disk file before reading
        if let Err(e) = store.reload() {
            tracing::warn!("Could not load interests store: {}", e);
        }
        if let Some(value) = store.get("interests") {
            if let Ok(interests) = serde_json::from_value::<Vec<Interest>>(value) {
                tracing::info!("Loaded {} RSS interests from disk", interests.len());
                *state.rss_state.interests.write().await = interests;
            }
        }
    }
}

pub async fn load_seen_items(app: &tauri::AppHandle, state: &AppState) {
    use std::collections::HashMap;

    if let Ok(store) = app.store(SEEN_ITEMS_STORE) {
        if let Err(e) = store.reload() {
            tracing::warn!("Could not load seen items store: {}", e);
        }
        if let Some(value) = store.get("seen_items") {
            if let Ok(items) = serde_json::from_value::<HashMap<String, String>>(value) {
                // Clean up entries older than 60 days
                let now = chrono::Utc::now();
                let cleaned: HashMap<String, String> = items
                    .into_iter()
                    .filter(|(_, timestamp)| {
                        chrono::DateTime::parse_from_rfc3339(timestamp)
                            .map(|t| (now - t.with_timezone(&chrono::Utc)).num_seconds() < SEEN_ITEMS_MAX_AGE_SECS)
                            .unwrap_or(false)
                    })
                    .collect();

                tracing::info!("Loaded {} seen RSS items from disk", cleaned.len());
                *state.rss_state.seen_items.lock().await = cleaned;
            }
        }
    }
}

pub async fn persist_seen_items(app: &tauri::AppHandle, state: &AppState) {
    if let Ok(store) = app.store(SEEN_ITEMS_STORE) {
        let seen = state.rss_state.seen_items.lock().await;
        if let Ok(value) = serde_json::to_value(&*seen) {
            store.set("seen_items", value);
            if let Err(e) = store.save() {
                tracing::error!("Failed to save seen items: {}", e);
            }
        }
    }
}

pub async fn load_bad_items(app: &tauri::AppHandle, state: &AppState) {
    use std::collections::HashMap;

    if let Ok(store) = app.store(BAD_ITEMS_STORE) {
        if let Err(e) = store.reload() {
            tracing::warn!("Could not load bad items store: {}", e);
        }
        if let Some(value) = store.get("bad_items") {
            if let Ok(items) = serde_json::from_value::<HashMap<String, BadItem>>(value) {
                tracing::info!("Loaded {} bad items from disk", items.len());
                *state.rss_state.bad_items.write().await = items;
            }
        }
    }
}

pub async fn persist_bad_items(app: &tauri::AppHandle, state: &AppState) {
    if let Ok(store) = app.store(BAD_ITEMS_STORE) {
        let bad = state.rss_state.bad_items.read().await;
        if let Ok(value) = serde_json::to_value(&*bad) {
            store.set("bad_items", value);
            if let Err(e) = store.save() {
                tracing::error!("Failed to save bad items: {}", e);
            }
        }
    }
}

// ── Source commands ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn rss_add_source(app: tauri::AppHandle, state: State<'_, AppState>, source: Source) -> Result<Source> {
    {
        let mut sources = state.rss_state.sources.write().await;

        if sources.iter().any(|s| s.url == source.url) {
            return Err(crate::errors::AppError::InvalidInput("Source URL already exists".into()));
        }

        sources.push(source.clone());
    }
    persist_sources(&app, &state).await;
    Ok(source)
}

#[tauri::command]
pub async fn rss_update_source(app: tauri::AppHandle, state: State<'_, AppState>, source: Source) -> Result<Source> {
    {
        let mut sources = state.rss_state.sources.write().await;

        if let Some(existing) = sources.iter_mut().find(|s| s.id == source.id) {
            *existing = source.clone();
        } else {
            return Err(crate::errors::AppError::NotFound("Source not found".into()));
        }
    }
    persist_sources(&app, &state).await;
    Ok(source)
}

#[tauri::command]
pub async fn rss_remove_source(app: tauri::AppHandle, state: State<'_, AppState>, source_id: String) -> Result<()> {
    {
        let mut sources = state.rss_state.sources.write().await;
        sources.retain(|s| s.id != source_id);
    }
    persist_sources(&app, &state).await;
    Ok(())
}

#[tauri::command]
pub async fn rss_list_sources(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<Vec<Source>> {
    // Lazy-load from disk if in-memory state is empty (handles race condition on startup)
    {
        let sources = state.rss_state.sources.read().await;
        if !sources.is_empty() {
            return Ok(sources.clone());
        }
    }
    // Try to load from disk
    load_sources(&app, &state).await;
    let sources = state.rss_state.sources.read().await;
    Ok(sources.clone())
}

#[tauri::command]
pub async fn rss_toggle_source(app: tauri::AppHandle, state: State<'_, AppState>, source_id: String, enabled: bool) -> Result<()> {
    {
        let mut sources = state.rss_state.sources.write().await;

        if let Some(source) = sources.iter_mut().find(|s| s.id == source_id) {
            source.enabled = enabled;
        } else {
            return Err(crate::errors::AppError::NotFound("Source not found".into()));
        }
    }
    persist_sources(&app, &state).await;
    Ok(())
}

// ── Interest commands ─────────────────────────────────────────────────────────

#[tauri::command]
pub async fn rss_add_interest(app: tauri::AppHandle, state: State<'_, AppState>, interest: Interest) -> Result<Interest> {
    {
        let mut interests = state.rss_state.interests.write().await;
        interests.push(interest.clone());
    }
    persist_interests(&app, &state).await;
    Ok(interest)
}

#[tauri::command]
pub async fn rss_update_interest(app: tauri::AppHandle, state: State<'_, AppState>, interest: Interest) -> Result<Interest> {
    {
        let mut interests = state.rss_state.interests.write().await;

        if let Some(existing) = interests.iter_mut().find(|i| i.id == interest.id) {
            *existing = interest.clone();
        } else {
            return Err(crate::errors::AppError::NotFound("Interest not found".into()));
        }
    }
    persist_interests(&app, &state).await;
    Ok(interest)
}

#[tauri::command]
pub async fn rss_remove_interest(app: tauri::AppHandle, state: State<'_, AppState>, interest_id: String) -> Result<()> {
    {
        let mut interests = state.rss_state.interests.write().await;
        interests.retain(|i| i.id != interest_id);
    }
    persist_interests(&app, &state).await;
    Ok(())
}

#[tauri::command]
pub async fn rss_list_interests(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<Vec<Interest>> {
    // Lazy-load from disk if in-memory state is empty (handles race condition on startup)
    {
        let interests = state.rss_state.interests.read().await;
        if !interests.is_empty() {
            return Ok(interests.clone());
        }
    }
    // Try to load from disk
    load_interests(&app, &state).await;
    let interests = state.rss_state.interests.read().await;
    Ok(interests.clone())
}

#[tauri::command]
pub async fn rss_toggle_interest(app: tauri::AppHandle, state: State<'_, AppState>, interest_id: String, enabled: bool) -> Result<()> {
    {
        let mut interests = state.rss_state.interests.write().await;

        if let Some(interest) = interests.iter_mut().find(|i| i.id == interest_id) {
            interest.enabled = enabled;
        } else {
            return Err(crate::errors::AppError::NotFound("Interest not found".into()));
        }
    }
    persist_interests(&app, &state).await;
    Ok(())
}

// ── Test command ──────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn rss_test_interest(url: String, filters: Vec<FeedFilter>) -> Result<FeedTestResult> {
    rss::test_feed(&url, &filters).await
}

// ── Screener commands ─────────────────────────────────────────────────────────

#[tauri::command]
pub async fn rss_list_pending(state: State<'_, AppState>) -> Result<Vec<PendingMatch>> {
    let matches = state.rss_state.pending_matches.read().await;
    Ok(matches.clone())
}

#[tauri::command]
pub async fn rss_pending_count(state: State<'_, AppState>) -> Result<usize> {
    let matches = state.rss_state.pending_matches.read().await;
    Ok(matches.len())
}

#[tauri::command]
pub async fn rss_fetch_metadata(app_handle: tauri::AppHandle, match_id: String) -> Result<TorrentMetadata> {
    rss::fetch_metadata(&app_handle, &match_id).await
}

#[tauri::command]
pub async fn rss_approve_match(app_handle: tauri::AppHandle, match_id: String) -> Result<i64> {
    rss::approve_match(&app_handle, &match_id).await
}

#[tauri::command]
pub async fn rss_reject_match(app_handle: tauri::AppHandle, match_id: String) -> Result<()> {
    rss::reject_match(&app_handle, &match_id).await
}

#[tauri::command]
pub async fn rss_check_now(app_handle: tauri::AppHandle) -> Result<usize> {
    rss::check_feeds_now(&app_handle).await
}

// ── Bad items commands ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn rss_mark_bad(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    info_hash: String,
    title: String,
    interest_id: Option<String>,
    interest_name: Option<String>,
    reason: Option<String>,
    trigger_rescan: bool,
) -> Result<usize> {
    let bad_item = BadItem {
        info_hash: info_hash.clone(),
        title,
        interest_id: interest_id.clone(),
        interest_name,
        marked_at: chrono::Utc::now().to_rfc3339(),
        reason,
    };

    {
        let mut bad_items = state.rss_state.bad_items.write().await;
        bad_items.insert(info_hash, bad_item);
    }
    persist_bad_items(&app_handle, &state).await;

    // Optionally trigger re-scan for the interest
    let mut new_matches = 0;
    if trigger_rescan {
        if let Some(interest_id) = interest_id {
            new_matches = rss::recheck_interest(&app_handle, &interest_id).await.unwrap_or(0);
        }
    }

    Ok(new_matches)
}

#[tauri::command]
pub async fn rss_unmark_bad(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    info_hash: String,
) -> Result<()> {
    {
        let mut bad_items = state.rss_state.bad_items.write().await;
        bad_items.remove(&info_hash);
    }
    persist_bad_items(&app_handle, &state).await;
    Ok(())
}

#[tauri::command]
pub async fn rss_list_bad(state: State<'_, AppState>) -> Result<Vec<BadItem>> {
    let bad_items = state.rss_state.bad_items.read().await;
    Ok(bad_items.values().cloned().collect())
}

fn get_demo_matches() -> Vec<PendingMatch> {
    vec![
        PendingMatch {
            id: "demo-1".to_string(),
            source_id: "demo-source-1".to_string(),
            source_name: "Linux ISOs".to_string(),
            interest_id: "demo-interest-1".to_string(),
            interest_name: "Ubuntu".to_string(),
            title: "ubuntu-24.04.1-desktop-amd64.iso".to_string(),
            magnet_uri: Some("magnet:?xt=urn:btih:demo1".to_string()),
            torrent_url: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            metadata: Some(TorrentMetadata {
                name: "ubuntu-24.04.1-desktop-amd64.iso".to_string(),
                total_size: 5_665_497_088,
                file_count: 1,
                files: vec![TorrentFilePreview {
                    name: "ubuntu-24.04.1-desktop-amd64.iso".to_string(),
                    size: 5_665_497_088,
                    is_video: false,
                    is_suspicious: false,
                }],
            }),
        },
        PendingMatch {
            id: "demo-2".to_string(),
            source_id: "demo-source-2".to_string(),
            source_name: "Blender Films".to_string(),
            interest_id: "demo-interest-2".to_string(),
            interest_name: "Open Movies".to_string(),
            title: "Big.Buck.Bunny.2008.4K.60fps.mkv".to_string(),
            magnet_uri: Some("magnet:?xt=urn:btih:demo2".to_string()),
            torrent_url: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            metadata: Some(TorrentMetadata {
                name: "Big.Buck.Bunny.2008.4K.60fps".to_string(),
                total_size: 694_157_312,
                file_count: 2,
                files: vec![
                    TorrentFilePreview {
                        name: "Big.Buck.Bunny.2008.4K.60fps.mkv".to_string(),
                        size: 693_000_000,
                        is_video: true,
                        is_suspicious: false,
                    },
                    TorrentFilePreview {
                        name: "README.txt".to_string(),
                        size: 1_157_312,
                        is_video: false,
                        is_suspicious: false,
                    },
                ],
            }),
        },
        PendingMatch {
            id: "demo-3".to_string(),
            source_id: "demo-source-2".to_string(),
            source_name: "Blender Films".to_string(),
            interest_id: "demo-interest-2".to_string(),
            interest_name: "Open Movies".to_string(),
            title: "Sintel.2010.1080p.mkv".to_string(),
            magnet_uri: Some("magnet:?xt=urn:btih:demo3".to_string()),
            torrent_url: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            metadata: None,
        },
    ]
}

/// Seed demo pending matches (for use from setup).
pub async fn seed_demo_pending(state: &AppState) -> Result<()> {
    let mut matches = state.rss_state.pending_matches.write().await;
    *matches = get_demo_matches();
    Ok(())
}

/// Seed demo pending matches for screenshots (Tauri command).
#[tauri::command]
pub async fn rss_seed_demo(state: State<'_, AppState>) -> Result<()> {
    seed_demo_pending(&state).await
}
