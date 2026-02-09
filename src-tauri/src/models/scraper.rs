// Web scraper configuration for non-RSS torrent sites.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScraperConfig {
    pub id: String,
    pub name: String,
    pub base_url: String,
    /// URL template with {search} placeholder for search queries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_url_template: Option<String>,
    /// CSS selector for item container elements.
    pub item_selector: String,
    /// CSS selector for title, relative to item container.
    pub title_selector: String,
    /// CSS selector for magnet/torrent link, relative to item container.
    pub link_selector: String,
    /// CSS selector for file size, relative to item container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_selector: Option<String>,
    pub enabled: bool,
    /// Delay between requests in milliseconds.
    #[serde(default = "default_delay")]
    pub request_delay_ms: u64,
}

fn default_delay() -> u64 {
    500
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedItem {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub magnet_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub torrent_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScraperTestResult {
    pub items: Vec<ScrapedItem>,
    pub total_count: usize,
}
