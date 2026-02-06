// RSS sources and interests models.

use serde::{Deserialize, Serialize};

/// A source is an RSS feed URL to poll for content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub check_interval_minutes: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_checked: Option<String>,
}

/// An interest is a pattern to watch for across all sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interest {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub filters: Vec<FeedFilter>,
    #[serde(default)]
    pub filter_logic: FilterLogic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FilterLogic {
    #[default]
    And,
    Or,
}

/// Legacy RssFeed for migration - remove after migration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RssFeed {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub check_interval_minutes: u32,
    pub filters: Vec<FeedFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_checked: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedFilter {
    #[serde(rename = "type")]
    pub filter_type: FilterType,
    pub value: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FilterType {
    MustContain,
    MustNotContain,
    Regex,
    SizeRange,
    Episode,
    Resolution,
    Source,
    Codec,
    Audio,
    Hdr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedItem {
    pub id: String,
    pub feed_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub magnet_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub torrent_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_date: Option<String>,
    pub status: FeedItemStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub torrent_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FeedItemStatus {
    Pending,
    Downloaded,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedTestResult {
    pub items: Vec<FeedTestItem>,
    pub total_count: usize,
    pub matched_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedTestItem {
    pub title: String,
    pub matches: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// A pending RSS match awaiting user approval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingMatch {
    pub id: String,
    pub source_id: String,
    pub source_name: String,
    pub interest_id: String,
    pub interest_name: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub magnet_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub torrent_url: Option<String>,
    pub created_at: String,
    /// Torrent metadata fetched for preview.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TorrentMetadata>,
}

/// Torrent metadata for screening before download.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentMetadata {
    pub name: String,
    pub total_size: u64,
    pub file_count: usize,
    pub files: Vec<TorrentFilePreview>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentFilePreview {
    pub name: String,
    pub size: u64,
    pub is_video: bool,
    pub is_suspicious: bool,
}
