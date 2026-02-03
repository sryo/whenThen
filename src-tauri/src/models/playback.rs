use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackStatusResponse {
    pub device_id: String,
    pub state: PlaybackState,
    pub current_time: f64,
    pub duration: f64,
    pub volume: f64,
    pub is_muted: bool,
    pub media_title: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PlaybackState {
    Idle,
    Buffering,
    Playing,
    Paused,
}

impl Default for PlaybackStatusResponse {
    fn default() -> Self {
        Self {
            device_id: String::new(),
            state: PlaybackState::Idle,
            current_time: 0.0,
            duration: 0.0,
            volume: 1.0,
            is_muted: false,
            media_title: None,
            content_type: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleInfo {
    pub url: String,
    pub name: String,
    pub format: String,
}

#[derive(Debug, Clone)]
pub struct SubtitleData {
    pub vtt_content: String,
    pub original_name: String,
}
