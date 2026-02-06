// RSS Tauri commands for sources, interests, and screener.

use tauri::State;

use crate::errors::Result;
use crate::models::{FeedFilter, FeedTestResult, Interest, PendingMatch, Source, TorrentMetadata};
use crate::services::rss;
use crate::state::AppState;

// ── Source commands ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn rss_add_source(state: State<'_, AppState>, source: Source) -> Result<Source> {
    let mut sources = state.rss_state.sources.write().await;

    if sources.iter().any(|s| s.url == source.url) {
        return Err(crate::errors::AppError::InvalidInput("Source URL already exists".into()));
    }

    sources.push(source.clone());
    Ok(source)
}

#[tauri::command]
pub async fn rss_update_source(state: State<'_, AppState>, source: Source) -> Result<Source> {
    let mut sources = state.rss_state.sources.write().await;

    if let Some(existing) = sources.iter_mut().find(|s| s.id == source.id) {
        *existing = source.clone();
        Ok(source)
    } else {
        Err(crate::errors::AppError::NotFound("Source not found".into()))
    }
}

#[tauri::command]
pub async fn rss_remove_source(state: State<'_, AppState>, source_id: String) -> Result<()> {
    let mut sources = state.rss_state.sources.write().await;
    sources.retain(|s| s.id != source_id);
    Ok(())
}

#[tauri::command]
pub async fn rss_list_sources(state: State<'_, AppState>) -> Result<Vec<Source>> {
    let sources = state.rss_state.sources.read().await;
    Ok(sources.clone())
}

#[tauri::command]
pub async fn rss_toggle_source(state: State<'_, AppState>, source_id: String, enabled: bool) -> Result<()> {
    let mut sources = state.rss_state.sources.write().await;

    if let Some(source) = sources.iter_mut().find(|s| s.id == source_id) {
        source.enabled = enabled;
        Ok(())
    } else {
        Err(crate::errors::AppError::NotFound("Source not found".into()))
    }
}

// ── Interest commands ─────────────────────────────────────────────────────────

#[tauri::command]
pub async fn rss_add_interest(state: State<'_, AppState>, interest: Interest) -> Result<Interest> {
    let mut interests = state.rss_state.interests.write().await;
    interests.push(interest.clone());
    Ok(interest)
}

#[tauri::command]
pub async fn rss_update_interest(state: State<'_, AppState>, interest: Interest) -> Result<Interest> {
    let mut interests = state.rss_state.interests.write().await;

    if let Some(existing) = interests.iter_mut().find(|i| i.id == interest.id) {
        *existing = interest.clone();
        Ok(interest)
    } else {
        Err(crate::errors::AppError::NotFound("Interest not found".into()))
    }
}

#[tauri::command]
pub async fn rss_remove_interest(state: State<'_, AppState>, interest_id: String) -> Result<()> {
    let mut interests = state.rss_state.interests.write().await;
    interests.retain(|i| i.id != interest_id);
    Ok(())
}

#[tauri::command]
pub async fn rss_list_interests(state: State<'_, AppState>) -> Result<Vec<Interest>> {
    let interests = state.rss_state.interests.read().await;
    Ok(interests.clone())
}

#[tauri::command]
pub async fn rss_toggle_interest(state: State<'_, AppState>, interest_id: String, enabled: bool) -> Result<()> {
    let mut interests = state.rss_state.interests.write().await;

    if let Some(interest) = interests.iter_mut().find(|i| i.id == interest_id) {
        interest.enabled = enabled;
        Ok(())
    } else {
        Err(crate::errors::AppError::NotFound("Interest not found".into()))
    }
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
