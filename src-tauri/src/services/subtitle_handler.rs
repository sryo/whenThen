use std::path::Path;
use tracing::info;

use crate::errors::{WhenThenError, Result};
use crate::models::SubtitleData;

pub fn load_subtitle_file(path: &str) -> Result<SubtitleData> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(WhenThenError::FileNotFound(path.display().to_string()));
    }

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let original_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("subtitles")
        .to_string();

    let content = std::fs::read_to_string(path)
        .map_err(|e| WhenThenError::SubtitleParse(format!("Failed to read file: {e}")))?;

    let vtt_content = match extension.as_str() {
        "vtt" => {
            info!("Loaded VTT subtitle: {}", original_name);
            content
        }
        "srt" => {
            info!("Converting SRT to VTT: {}", original_name);
            srt_to_vtt(&content)?
        }
        _ => {
            return Err(WhenThenError::UnsupportedFormat(format!(
                "Unsupported subtitle format: .{}",
                extension
            )));
        }
    };

    Ok(SubtitleData {
        vtt_content,
        original_name,
    })
}

fn srt_to_vtt(srt_content: &str) -> Result<String> {
    let mut vtt = String::from("WEBVTT\n\n");
    let content = srt_content.replace('\r', "");
    let blocks: Vec<&str> = content.split("\n\n").collect();

    for block in blocks {
        let lines: Vec<&str> = block.trim().lines().collect();
        if lines.len() < 3 {
            continue;
        }

        // Skip the sequence number (first line)
        // Convert timestamp format: 00:00:00,000 -> 00:00:00.000
        let timestamp = lines[1].replace(',', ".");

        let text: Vec<&str> = lines[2..].to_vec();

        vtt.push_str(&timestamp);
        vtt.push('\n');
        for line in text {
            vtt.push_str(line);
            vtt.push('\n');
        }
        vtt.push('\n');
    }

    Ok(vtt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srt_to_vtt_conversion() {
        let srt = "1\n00:00:01,000 --> 00:00:04,000\nHello World\n\n2\n00:00:05,000 --> 00:00:08,000\nSecond line";
        let result = srt_to_vtt(srt).unwrap();
        assert!(result.starts_with("WEBVTT"));
        assert!(result.contains("00:00:01.000 --> 00:00:04.000"));
        assert!(result.contains("Hello World"));
    }
}
