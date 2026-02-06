use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum WhenThenError {
    #[error("Torrent error: {0}")]
    Torrent(String),

    #[error("Torrent not found: {0}")]
    TorrentNotFound(usize),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Cast connection error: {0}")]
    CastConnection(String),

    #[error("Cast playback error: {0}")]
    CastPlayback(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Subtitle parse error: {0}")]
    SubtitleParse(String),

    #[error("OpenSubtitles error: {0}")]
    OpenSubtitles(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("RSS error: {0}")]
    Rss(String),
}

// Type alias for backwards compatibility
pub type AppError = WhenThenError;

impl Serialize for WhenThenError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<anyhow::Error> for WhenThenError {
    fn from(err: anyhow::Error) -> Self {
        WhenThenError::Internal(err.to_string())
    }
}

impl From<reqwest::Error> for WhenThenError {
    fn from(err: reqwest::Error) -> Self {
        WhenThenError::Rss(err.to_string())
    }
}

impl From<feed_rs::parser::ParseFeedError> for WhenThenError {
    fn from(err: feed_rs::parser::ParseFeedError) -> Self {
        WhenThenError::Rss(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, WhenThenError>;
