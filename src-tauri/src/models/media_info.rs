// Parsed media metadata from torrent/video filenames.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Quality {
    Q2160p,
    Q1080p,
    Q720p,
    Q480p,
}

impl Quality {
    pub fn as_str(&self) -> &'static str {
        match self {
            Quality::Q2160p => "2160p",
            Quality::Q1080p => "1080p",
            Quality::Q720p => "720p",
            Quality::Q480p => "480p",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaSource {
    BluRay,
    WebDl,
    WebRip,
    Hdtv,
    DvdRip,
}

impl MediaSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaSource::BluRay => "BluRay",
            MediaSource::WebDl => "WEB-DL",
            MediaSource::WebRip => "WEBRip",
            MediaSource::Hdtv => "HDTV",
            MediaSource::DvdRip => "DVDRip",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Codec {
    X264,
    X265,
    Av1,
}

impl Codec {
    pub fn as_str(&self) -> &'static str {
        match self {
            Codec::X264 => "x264",
            Codec::X265 => "x265",
            Codec::Av1 => "AV1",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MediaInfo {
    pub title: String,
    pub year: Option<u16>,
    pub quality: Option<Quality>,
    pub source: Option<MediaSource>,
    pub codec: Option<Codec>,
    pub release_group: Option<String>,
    pub season: Option<u16>,
    pub episode: Option<u16>,
    pub is_proper: bool,
    pub is_repack: bool,
}

impl MediaInfo {
    pub fn is_tv(&self) -> bool {
        self.season.is_some() || self.episode.is_some()
    }
}
