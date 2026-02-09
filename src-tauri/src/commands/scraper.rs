// Scraper commands for web scraping torrent sites.

use tauri::State;

use crate::errors::Result;
use crate::models::{ScraperConfig, ScraperTestResult};
use crate::services::scraper;
use crate::state::AppState;

#[tauri::command]
pub async fn scraper_add_config(state: State<'_, AppState>, config: ScraperConfig) -> Result<()> {
    let mut configs = state.scraper_state.configs.write().await;
    configs.push(config);
    Ok(())
}

#[tauri::command]
pub async fn scraper_update_config(state: State<'_, AppState>, config: ScraperConfig) -> Result<()> {
    let mut configs = state.scraper_state.configs.write().await;
    if let Some(existing) = configs.iter_mut().find(|c| c.id == config.id) {
        *existing = config;
    }
    Ok(())
}

#[tauri::command]
pub async fn scraper_remove_config(state: State<'_, AppState>, id: String) -> Result<()> {
    let mut configs = state.scraper_state.configs.write().await;
    configs.retain(|c| c.id != id);
    Ok(())
}

#[tauri::command]
pub async fn scraper_list_configs(state: State<'_, AppState>) -> Result<Vec<ScraperConfig>> {
    let configs = state.scraper_state.configs.read().await;
    Ok(configs.clone())
}

#[tauri::command]
pub async fn scraper_toggle(state: State<'_, AppState>, id: String, enabled: bool) -> Result<()> {
    let mut configs = state.scraper_state.configs.write().await;
    if let Some(config) = configs.iter_mut().find(|c| c.id == id) {
        config.enabled = enabled;
    }
    Ok(())
}

#[tauri::command]
pub async fn scraper_test(config: ScraperConfig) -> Result<ScraperTestResult> {
    scraper::test_scraper(&config).await
}
