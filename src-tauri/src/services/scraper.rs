// Web scraper service for non-RSS torrent sites.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use regex::Regex;
use scraper::{Html, Selector};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn};

use crate::errors::{Result, WhenThenError};
use crate::models::{Interest, PendingMatch, ScrapedItem, ScraperConfig, ScraperTestResult};
use crate::services::rss::{evaluate_filters_with_logic, ParsedFeedItem, RssState};
use crate::state::AppState;

pub struct ScraperState {
    pub configs: Arc<RwLock<Vec<ScraperConfig>>>,
    /// Seen items: key -> ISO timestamp
    pub seen_items: Arc<Mutex<HashMap<String, String>>>,
}

impl ScraperState {
    pub fn new() -> Self {
        Self {
            configs: Arc::new(RwLock::new(Vec::new())),
            seen_items: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// Scrape a page using the given config.
pub async fn scrape_page(config: &ScraperConfig, url: &str) -> Result<Vec<ScrapedItem>> {
    // Rate limit
    tokio::time::sleep(std::time::Duration::from_millis(config.request_delay_ms)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
        .send()
        .await
        .map_err(|e| WhenThenError::Scraper(format!("Request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(WhenThenError::Scraper(format!(
            "Request returned status {}",
            response.status()
        )));
    }

    let html = response
        .text()
        .await
        .map_err(|e| WhenThenError::Scraper(format!("Failed to read response: {}", e)))?;

    parse_page(&html, config)
}

/// Parse HTML page using scraper config selectors.
fn parse_page(html: &str, config: &ScraperConfig) -> Result<Vec<ScrapedItem>> {
    let document = Html::parse_document(html);

    let item_sel = Selector::parse(&config.item_selector)
        .map_err(|_| WhenThenError::Scraper(format!("Invalid item selector: {}", config.item_selector)))?;

    let title_sel = Selector::parse(&config.title_selector)
        .map_err(|_| WhenThenError::Scraper(format!("Invalid title selector: {}", config.title_selector)))?;

    let link_sel = Selector::parse(&config.link_selector)
        .map_err(|_| WhenThenError::Scraper(format!("Invalid link selector: {}", config.link_selector)))?;

    let size_sel = config
        .size_selector
        .as_ref()
        .map(|s| Selector::parse(s))
        .transpose()
        .map_err(|_| WhenThenError::Scraper("Invalid size selector".into()))?;

    let mut items = Vec::new();

    for item in document.select(&item_sel) {
        // Get title
        let title = item
            .select(&title_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        if title.is_empty() {
            continue;
        }

        // Get link (magnet or torrent URL)
        let mut magnet_uri = None;
        let mut torrent_url = None;

        if let Some(link_elem) = item.select(&link_sel).next() {
            if let Some(href) = link_elem.value().attr("href") {
                if href.starts_with("magnet:") {
                    magnet_uri = Some(href.to_string());
                } else if href.ends_with(".torrent") || href.contains("/download") {
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("{}{}", config.base_url, href)
                    };
                    torrent_url = Some(url);
                } else {
                    // Try to find magnet in the element text or data attributes
                    let text = link_elem.text().collect::<String>();
                    if let Some(mag) = extract_magnet(&text) {
                        magnet_uri = Some(mag);
                    }
                }
            }
        }

        // Skip items without any download link
        if magnet_uri.is_none() && torrent_url.is_none() {
            continue;
        }

        // Get size
        let size = size_sel.as_ref().and_then(|sel| {
            item.select(sel)
                .next()
                .and_then(|e| parse_size(&e.text().collect::<String>()))
        });

        items.push(ScrapedItem {
            title,
            magnet_uri,
            torrent_url,
            size,
        });
    }

    Ok(items)
}

/// Extract magnet link from text.
fn extract_magnet(text: &str) -> Option<String> {
    if let Some(start) = text.find("magnet:?") {
        let rest = &text[start..];
        let end = rest
            .find(|c: char| c.is_whitespace() || c == '<' || c == '"' || c == '\'')
            .unwrap_or(rest.len());
        return Some(rest[..end].to_string());
    }
    None
}

/// Parse size string like "1.5 GB" to bytes.
fn parse_size(text: &str) -> Option<u64> {
    let size_re = Regex::new(r"(\d+(?:\.\d+)?)\s*(GB|MB|KB|GiB|MiB|KiB|TB|TiB)").ok()?;
    let caps = size_re.captures(text)?;

    let value: f64 = caps.get(1)?.as_str().parse().ok()?;
    let unit = caps.get(2)?.as_str();

    let multiplier = match unit {
        "KB" | "KiB" => 1024.0,
        "MB" | "MiB" => 1024.0 * 1024.0,
        "GB" | "GiB" => 1024.0 * 1024.0 * 1024.0,
        "TB" | "TiB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };

    Some((value * multiplier) as u64)
}

/// Build search URL from template.
fn build_search_url(config: &ScraperConfig, interest: &Interest) -> Option<String> {
    config.search_url_template.as_ref().map(|template| {
        let term = interest
            .search_term
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(&interest.name);
        let encoded = urlencoding::encode(term);
        template.replace("{search}", &encoded)
    })
}

/// Check a scraper config against all interests and queue matches.
pub async fn check_scraper_for_matches(
    app_handle: &AppHandle,
    scraper_state: &ScraperState,
    rss_state: &RssState,
    config: &ScraperConfig,
    interests: &[&Interest],
) -> Result<usize> {
    let mut matched_count = 0;

    for interest in interests {
        let url = match build_search_url(config, interest) {
            Some(u) => u,
            None => config.base_url.clone(),
        };

        info!("Scraping {} for interest '{}'", url, interest.name);

        match scrape_page(config, &url).await {
            Ok(items) => {
                let count = process_scraped_items(
                    app_handle,
                    scraper_state,
                    rss_state,
                    config,
                    interest,
                    &items,
                )
                .await;
                matched_count += count;
            }
            Err(e) => {
                warn!("Failed to scrape {} for '{}': {}", url, interest.name, e);
            }
        }
    }

    Ok(matched_count)
}

/// Process scraped items and create pending matches.
async fn process_scraped_items(
    app_handle: &AppHandle,
    scraper_state: &ScraperState,
    rss_state: &RssState,
    config: &ScraperConfig,
    interest: &Interest,
    items: &[ScrapedItem],
) -> usize {
    let mut matched_count = 0;

    for item in items {
        let mut seen = scraper_state.seen_items.lock().await;
        let item_key = format!("{}:{}:{}", config.id, interest.id, item.title);

        if seen.contains_key(&item_key) {
            continue;
        }

        let now = Utc::now().to_rfc3339();

        // Convert to ParsedFeedItem for filter evaluation
        let feed_item = ParsedFeedItem {
            id: item.title.clone(),
            guid: item.title.clone(),
            title: item.title.clone(),
            magnet_uri: item.magnet_uri.clone(),
            torrent_url: item.torrent_url.clone(),
            size: item.size,
            published_date: Some(now.clone()),
        };

        let matched = evaluate_filters_with_logic(&feed_item, &interest.filters, &interest.filter_logic);
        if matched.is_none() {
            seen.insert(item_key, now);
            continue;
        }

        seen.insert(item_key, now.clone());
        drop(seen);

        let pending = PendingMatch {
            id: uuid::Uuid::new_v4().to_string(),
            source_id: config.id.clone(),
            source_name: format!("{} (scraper)", config.name),
            interest_id: interest.id.clone(),
            interest_name: interest.name.clone(),
            title: item.title.clone(),
            magnet_uri: item.magnet_uri.clone(),
            torrent_url: item.torrent_url.clone(),
            created_at: now,
            metadata: None,
        };

        rss_state.pending_matches.write().await.push(pending.clone());
        matched_count += 1;

        let _ = app_handle.emit(
            "rss:new-match",
            serde_json::json!({
                "id": pending.id,
                "source_name": config.name,
                "interest_name": interest.name,
                "title": item.title,
            }),
        );
    }

    matched_count
}

/// Test a scraper config.
pub async fn test_scraper(config: &ScraperConfig) -> Result<ScraperTestResult> {
    let url = config.search_url_template.as_ref().unwrap_or(&config.base_url);
    let items = scrape_page(config, url).await?;

    Ok(ScraperTestResult {
        total_count: items.len(),
        items,
    })
}
