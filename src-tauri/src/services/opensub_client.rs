use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use serde::Deserialize;

use crate::errors::{WhenThenError, Result};
use crate::models::SubtitleSearchResult;

const API_BASE: &str = "https://api.opensubtitles.com/api/v1";
const USER_AGENT: &str = "whenThen v1.0.0";

#[derive(Deserialize)]
struct SearchResponse {
    data: Vec<SearchEntry>,
}

#[derive(Deserialize)]
struct SearchEntry {
    id: String,
    attributes: SearchAttributes,
}

#[derive(Deserialize)]
struct SearchAttributes {
    language: String,
    download_count: i64,
    ratings: f64,
    files: Vec<SearchFile>,
}

#[derive(Deserialize)]
struct SearchFile {
    file_id: i64,
    file_name: String,
}

#[derive(Deserialize)]
struct DownloadResponse {
    link: String,
    file_name: String,
}

pub async fn search(
    api_key: &str,
    languages: &[String],
    query: &str,
    movie_hash: Option<&str>,
) -> Result<Vec<SubtitleSearchResult>> {
    let client = reqwest::Client::new();

    let lang_str = languages.join(",");
    let mut url = format!(
        "{}/subtitles?languages={}&query={}",
        API_BASE,
        urlencoded(&lang_str),
        urlencoded(query),
    );
    if let Some(hash) = movie_hash {
        url.push_str(&format!("&moviehash={}", hash));
    }

    let response = client
        .get(&url)
        .header("Api-Key", api_key)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .map_err(|e| WhenThenError::OpenSubtitles(format!("Search request failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(WhenThenError::OpenSubtitles(format!(
            "Search failed with status {}: {}",
            status, body
        )));
    }

    let search_resp: SearchResponse = response
        .json()
        .await
        .map_err(|e| WhenThenError::OpenSubtitles(format!("Failed to parse search response: {e}")))?;

    let mut results = Vec::new();
    for entry in search_resp.data {
        if let Some(file) = entry.attributes.files.first() {
            results.push(SubtitleSearchResult {
                id: entry.id,
                file_id: file.file_id,
                language: entry.attributes.language.clone(),
                file_name: file.file_name.clone(),
                download_count: entry.attributes.download_count,
                ratings: entry.attributes.ratings,
            });
        }
    }

    Ok(results)
}

pub async fn download(api_key: &str, file_id: i64) -> Result<(String, Vec<u8>)> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({ "file_id": file_id });

    let response = client
        .post(format!("{}/download", API_BASE))
        .header("Api-Key", api_key)
        .header("User-Agent", USER_AGENT)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| WhenThenError::OpenSubtitles(format!("Download request failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(WhenThenError::OpenSubtitles(format!(
            "Download failed with status {}: {}",
            status, body
        )));
    }

    let dl_resp: DownloadResponse = response
        .json()
        .await
        .map_err(|e| WhenThenError::OpenSubtitles(format!("Failed to parse download response: {e}")))?;

    let file_bytes = client
        .get(&dl_resp.link)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .map_err(|e| WhenThenError::OpenSubtitles(format!("Failed to fetch subtitle file: {e}")))?
        .bytes()
        .await
        .map_err(|e| WhenThenError::OpenSubtitles(format!("Failed to read subtitle bytes: {e}")))?;

    Ok((dl_resp.file_name, file_bytes.to_vec()))
}

/// Compute the OpenSubtitles hash for a file.
/// Algorithm: sum of first 64KB + last 64KB as little-endian u64s, plus file size,
/// formatted as 16-char lowercase hex.
pub fn compute_hash(file_path: &Path) -> Option<String> {
    const CHUNK_SIZE: u64 = 65536; // 64KB

    let mut file = std::fs::File::open(file_path).ok()?;
    let file_size = file.metadata().ok()?.len();

    if file_size < CHUNK_SIZE * 2 {
        return None;
    }

    let mut hash: u64 = file_size;

    let mut buf = [0u8; 8];
    for _ in 0..(CHUNK_SIZE / 8) {
        file.read_exact(&mut buf).ok()?;
        hash = hash.wrapping_add(u64::from_le_bytes(buf));
    }

    file.seek(SeekFrom::End(-(CHUNK_SIZE as i64))).ok()?;
    for _ in 0..(CHUNK_SIZE / 8) {
        file.read_exact(&mut buf).ok()?;
        hash = hash.wrapping_add(u64::from_le_bytes(buf));
    }

    Some(format!("{:016x}", hash))
}

fn urlencoded(s: &str) -> String {
    s.bytes()
        .flat_map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b',' => {
                vec![b as char]
            }
            b' ' => vec!['+'],
            _ => format!("%{:02X}", b).chars().collect(),
        })
        .collect()
}
