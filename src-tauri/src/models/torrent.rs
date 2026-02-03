use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentAddedResponse {
    pub id: usize,
    pub name: String,
    pub info_hash: String,
    pub files: Vec<TorrentFileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentSummary {
    pub id: usize,
    pub name: String,
    pub info_hash: String,
    pub state: TorrentState,
    pub progress: f64,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub peers_connected: usize,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentDetails {
    pub id: usize,
    pub name: String,
    pub info_hash: String,
    pub state: TorrentState,
    pub progress: f64,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub peers_connected: usize,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub file_count: usize,
    pub files: Vec<TorrentFileInfo>,
    pub output_folder: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentFileInfo {
    pub index: usize,
    pub name: String,
    pub path: String,
    pub length: u64,
    pub is_playable: bool,
    pub mime_type: Option<String>,
    pub stream_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentAddOptions {
    pub output_folder: Option<String>,
    pub only_files: Option<Vec<usize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TorrentState {
    Initializing,
    Downloading,
    Paused,
    Completed,
    Error,
}
