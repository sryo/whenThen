// Parse media metadata from torrent/video filenames.

use regex::Regex;
use std::sync::LazyLock;

use crate::models::{Codec, MediaInfo, MediaSource, Quality};

static QUALITY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(4K|2160p|1080p|720p|480p)\b").unwrap());

static SOURCE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(BluRay|BDRip|BRRip|WEB-DL|WEBDL|WEBRip|HDTV|DVDRip)\b").unwrap()
});

static CODEC_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(x264|x265|HEVC|H\.?264|H\.?265|AV1)\b").unwrap());

static TV_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bS(\d{1,2})E(\d{1,2})\b").unwrap());

static TV_ALT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(\d{1,2})x(\d{1,2})\b").unwrap());

static YEAR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b((?:19|20)\d{2})\b").unwrap());

static GROUP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-([A-Za-z0-9]+)(?:\.[a-z]{2,4})?$").unwrap());

static PROPER_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\bPROPER\b").unwrap());

static REPACK_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\bREPACK\b").unwrap());

/// Parse a filename into structured media info.
pub fn parse(name: &str) -> MediaInfo {
    let mut info = MediaInfo::default();

    // Quality
    if let Some(caps) = QUALITY_RE.captures(name) {
        let q = caps.get(1).unwrap().as_str().to_uppercase();
        info.quality = match q.as_str() {
            "4K" | "2160P" => Some(Quality::Q2160p),
            "1080P" => Some(Quality::Q1080p),
            "720P" => Some(Quality::Q720p),
            "480P" => Some(Quality::Q480p),
            _ => None,
        };
    }

    // Source
    if let Some(caps) = SOURCE_RE.captures(name) {
        let s = caps.get(1).unwrap().as_str().to_uppercase();
        info.source = match s.as_str() {
            "BLURAY" | "BDRIP" | "BRRIP" => Some(MediaSource::BluRay),
            "WEB-DL" | "WEBDL" => Some(MediaSource::WebDl),
            "WEBRIP" => Some(MediaSource::WebRip),
            "HDTV" => Some(MediaSource::Hdtv),
            "DVDRIP" => Some(MediaSource::DvdRip),
            _ => None,
        };
    }

    // Codec
    if let Some(caps) = CODEC_RE.captures(name) {
        let c = caps.get(1).unwrap().as_str().to_uppercase().replace('.', "");
        info.codec = match c.as_str() {
            "X264" | "H264" => Some(Codec::X264),
            "X265" | "HEVC" | "H265" => Some(Codec::X265),
            "AV1" => Some(Codec::Av1),
            _ => None,
        };
    }

    // TV season/episode
    if let Some(caps) = TV_RE.captures(name) {
        info.season = caps.get(1).and_then(|m| m.as_str().parse().ok());
        info.episode = caps.get(2).and_then(|m| m.as_str().parse().ok());
    } else if let Some(caps) = TV_ALT_RE.captures(name) {
        info.season = caps.get(1).and_then(|m| m.as_str().parse().ok());
        info.episode = caps.get(2).and_then(|m| m.as_str().parse().ok());
    }

    // Year
    if let Some(caps) = YEAR_RE.captures(name) {
        info.year = caps.get(1).and_then(|m| m.as_str().parse().ok());
    }

    // Release group
    if let Some(caps) = GROUP_RE.captures(name) {
        info.release_group = caps.get(1).map(|m| m.as_str().to_string());
    }

    // Proper/Repack flags
    info.is_proper = PROPER_RE.is_match(name);
    info.is_repack = REPACK_RE.is_match(name);

    // Extract title (everything before first metadata token)
    info.title = extract_title(name, &info);

    info
}

/// Extract the title portion of the filename.
fn extract_title(name: &str, info: &MediaInfo) -> String {
    // Replace dots/underscores with spaces if they're used as separators
    let normalized = if name.matches('.').count() > 2 {
        name.replace(['.', '_'], " ")
    } else {
        name.to_string()
    };

    // Find the first metadata marker
    let markers = [
        info.year.map(|y| y.to_string()),
        info.quality.map(|q| q.as_str().to_string()),
        info.source.map(|s| s.as_str().to_string()),
        info.season.map(|s| format!("S{:02}", s)),
    ];

    let mut end_pos = normalized.len();
    for marker in markers.into_iter().flatten() {
        if let Some(pos) = normalized.to_lowercase().find(&marker.to_lowercase()) {
            if pos < end_pos && pos > 0 {
                end_pos = pos;
            }
        }
    }

    let title = normalized[..end_pos].trim().to_string();

    // Clean up bracketed tags at the start
    let title = title
        .trim_start_matches('[')
        .split(']')
        .next_back()
        .unwrap_or(&title)
        .trim()
        .to_string();

    title
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_movie() {
        let info = parse("Movie.2024.1080p.BluRay.x264-GROUP");
        assert_eq!(info.title, "Movie");
        assert_eq!(info.year, Some(2024));
        assert_eq!(info.quality, Some(Quality::Q1080p));
        assert_eq!(info.source, Some(Source::BluRay));
        assert_eq!(info.codec, Some(Codec::X264));
        assert_eq!(info.release_group, Some("GROUP".to_string()));
    }

    #[test]
    fn test_parse_tv() {
        let info = parse("Show.S02E05.720p.WEB-DL");
        assert_eq!(info.title, "Show");
        assert_eq!(info.season, Some(2));
        assert_eq!(info.episode, Some(5));
        assert_eq!(info.quality, Some(Quality::Q720p));
        assert_eq!(info.source, Some(Source::WebDl));
    }

    #[test]
    fn test_parse_4k() {
        let info = parse("Film.Name.2023.2160p.WEBRip.x265-RARBG");
        assert_eq!(info.quality, Some(Quality::Q2160p));
        assert_eq!(info.codec, Some(Codec::X265));
    }

    #[test]
    fn test_parse_proper() {
        let info = parse("Movie.2024.1080p.PROPER.BluRay.x264-GROUP");
        assert!(info.is_proper);
        assert!(!info.is_repack);
    }

    #[test]
    fn test_parse_repack() {
        let info = parse("Show.S01E01.REPACK.720p.HDTV.x264-LOL");
        assert!(!info.is_proper);
        assert!(info.is_repack);
    }
}
