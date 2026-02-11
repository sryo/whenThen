// Score subtitles against video metadata for better matching.

use crate::models::MediaInfo;
use crate::services::media_info;

/// Scoring weights from supper project.
const TITLE_WEIGHT: f64 = 0.40;
const QUALITY_WEIGHT: f64 = 0.25;
const SOURCE_WEIGHT: f64 = 0.20;
const GROUP_WEIGHT: f64 = 0.15;

/// Score a subtitle against a video file.
#[allow(dead_code)]
pub fn score(video_name: &str, subtitle_name: &str) -> f64 {
    let video_info = media_info::parse(video_name);
    let sub_info = media_info::parse(subtitle_name);

    score_infos(&video_info, &sub_info)
}

/// Score using pre-parsed MediaInfo structs.
pub fn score_infos(video: &MediaInfo, subtitle: &MediaInfo) -> f64 {
    let mut score = 0.0;

    // Title similarity (Jaro-Winkler)
    let title_sim = jaro_winkler(&video.title.to_lowercase(), &subtitle.title.to_lowercase());
    score += title_sim * TITLE_WEIGHT;

    // Quality match
    if video.quality.is_some() && video.quality == subtitle.quality {
        score += QUALITY_WEIGHT;
    }

    // Source match
    if video.source.is_some() && video.source == subtitle.source {
        score += SOURCE_WEIGHT;
    }

    // Release group match
    if let (Some(ref vg), Some(ref sg)) = (&video.release_group, &subtitle.release_group) {
        if vg.to_lowercase() == sg.to_lowercase() {
            score += GROUP_WEIGHT;
        }
    }

    score
}

/// Jaro-Winkler string similarity (0.0 to 1.0).
fn jaro_winkler(s1: &str, s2: &str) -> f64 {
    if s1.is_empty() && s2.is_empty() {
        return 1.0;
    }
    if s1.is_empty() || s2.is_empty() {
        return 0.0;
    }

    let jaro = jaro_similarity(s1, s2);

    // Jaro-Winkler: boost up to 0.4 for matching prefix (4 chars * 0.1)
    let prefix_len = s1
        .chars()
        .zip(s2.chars())
        .take(4)
        .take_while(|(a, b)| a == b)
        .count();

    jaro + (prefix_len as f64 * 0.1 * (1.0 - jaro))
}

/// Jaro string similarity.
fn jaro_similarity(s1: &str, s2: &str) -> f64 {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let len1 = s1_chars.len();
    let len2 = s2_chars.len();

    if len1 == 0 && len2 == 0 {
        return 1.0;
    }
    if len1 == 0 || len2 == 0 {
        return 0.0;
    }

    let match_distance = (len1.max(len2) / 2).saturating_sub(1);

    let mut s1_matches = vec![false; len1];
    let mut s2_matches = vec![false; len2];

    let mut matches = 0;
    let mut transpositions = 0;

    for i in 0..len1 {
        let start = i.saturating_sub(match_distance);
        let end = (i + match_distance + 1).min(len2);

        for j in start..end {
            if s2_matches[j] || s1_chars[i] != s2_chars[j] {
                continue;
            }
            s1_matches[i] = true;
            s2_matches[j] = true;
            matches += 1;
            break;
        }
    }

    if matches == 0 {
        return 0.0;
    }

    let mut k = 0;
    for i in 0..len1 {
        if !s1_matches[i] {
            continue;
        }
        while !s2_matches[k] {
            k += 1;
        }
        if s1_chars[i] != s2_chars[k] {
            transpositions += 1;
        }
        k += 1;
    }

    let m = matches as f64;
    let t = (transpositions / 2) as f64;

    (m / len1 as f64 + m / len2 as f64 + (m - t) / m) / 3.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let score = score(
            "Movie.2024.1080p.BluRay.x264-GROUP",
            "Movie.2024.1080p.BluRay.x264-GROUP",
        );
        assert!(score > 0.95);
    }

    #[test]
    fn test_title_only_match() {
        let score = score("Movie.2024.1080p.BluRay.x264-GROUP", "Movie.2024.720p.WEBRip");
        // Should still match on title
        assert!(score > 0.3);
    }

    #[test]
    fn test_different_titles() {
        let score = score(
            "Movie.2024.1080p.BluRay.x264-GROUP",
            "OtherFilm.2024.1080p.BluRay.x264-GROUP",
        );
        // Quality and source match, but title doesn't
        assert!(score < 0.5);
    }

    #[test]
    fn test_jaro_winkler() {
        assert!(jaro_winkler("hello", "hello") > 0.99);
        assert!(jaro_winkler("hello", "hallo") > 0.8);
        assert!(jaro_winkler("hello", "world") < 0.5);
    }
}
