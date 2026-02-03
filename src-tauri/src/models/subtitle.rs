use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleSearchResult {
    pub id: String,
    pub file_id: i64,
    pub language: String,
    pub file_name: String,
    pub download_count: i64,
    pub ratings: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleDownloadResult {
    pub file_name: String,
    pub file_path: String,
}
