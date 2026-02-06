// RSS Tauri commands for sources, interests, and screener.

use tauri::State;
use tauri_plugin_store::StoreExt;

use crate::errors::Result;
use crate::models::{FeedFilter, FeedTestResult, Interest, PendingMatch, Source, TorrentMetadata};
use crate::services::rss;
use crate::state::AppState;

const SOURCES_STORE: &str = "sources.json";
const INTERESTS_STORE: &str = "interests.json";

async fn persist_sources(app: &tauri::AppHandle, state: &AppState) {
    if let Ok(store) = app.store(SOURCES_STORE) {
        let sources = state.rss_state.sources.read().await;
        if let Ok(value) = serde_json::to_value(&*sources) {
            store.set("sources", value);
            let _ = store.save();
        }
    }
}

async fn persist_interests(app: &tauri::AppHandle, state: &AppState) {
    if let Ok(store) = app.store(INTERESTS_STORE) {
        let interests = state.rss_state.interests.read().await;
        if let Ok(value) = serde_json::to_value(&*interests) {
            store.set("interests", value);
            let _ = store.save();
        }
    }
}

pub async fn load_sources(app: &tauri::AppHandle, state: &AppState) {
    if let Ok(store) = app.store(SOURCES_STORE) {
        if let Some(value) = store.get("sources") {
            if let Ok(sources) = serde_json::from_value::<Vec<Source>>(value) {
                *state.rss_state.sources.write().await = sources;
            }
        }
    }
}

pub async fn load_interests(app: &tauri::AppHandle, state: &AppState) {
    if let Ok(store) = app.store(INTERESTS_STORE) {
        if let Some(value) = store.get("interests") {
            if let Ok(interests) = serde_json::from_value::<Vec<Interest>>(value) {
                *state.rss_state.interests.write().await = interests;
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
pub async fn rss_list_sources(state: State<'_, AppState>) -> Result<Vec<Source>> {
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
pub async fn rss_list_interests(state: State<'_, AppState>) -> Result<Vec<Interest>> {
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
