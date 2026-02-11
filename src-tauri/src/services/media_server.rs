use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use axum::{
    Router,
    body::Body,
    extract::{Path, State as AxumState},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use tokio::sync::RwLock;
use tokio::io::AsyncReadExt;
use tower_http::cors::CorsLayer;
use tracing::{info, error};

use crate::models::SubtitleData;

/// Tokens expire after 1 hour.
const TOKEN_TTL_SECS: u64 = 3600;
/// Cleanup runs every 10 minutes.
const TOKEN_CLEANUP_INTERVAL_SECS: u64 = 600;

#[derive(Clone)]
pub struct TokenEntry {
    pub path: String,
    pub created_at: std::time::Instant,
}

#[derive(Clone)]
pub struct MediaServerState {
    pub torrent_session: Arc<RwLock<Option<Arc<librqbit::Session>>>>,
    pub current_subtitles: Arc<RwLock<Option<SubtitleData>>>,
    pub local_file_tokens: Arc<RwLock<HashMap<String, TokenEntry>>>,
}

pub struct MediaServerHandle {
    pub port: u16,
    shutdown_tx: Arc<RwLock<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl MediaServerHandle {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            shutdown_tx: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start(&self, state: MediaServerState) {
        let port = self.port;
        let shutdown_tx = self.shutdown_tx.clone();

        let cors = CorsLayer::new()
            .allow_origin([
                "tauri://localhost".parse().unwrap(),
                "https://tauri.localhost".parse().unwrap(),
                "http://localhost".parse().unwrap(),
                "http://127.0.0.1".parse().unwrap(),
            ])
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any);

        let app = Router::new()
            .route("/torrent/{torrent_id}/stream/{file_idx}", get(stream_torrent))
            .route("/torrent/{torrent_id}/playlist.m3u8", get(serve_playlist))
            .route("/local/{token}", get(serve_local_file))
            .route("/subtitles.vtt", get(serve_subtitles))
            .route("/health", get(health_check))
            .layer(cors)
            .with_state(state.clone());

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind media server to port {}: {}", port, e);
                return;
            }
        };

        info!("Media server listening on http://0.0.0.0:{}", port);

        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        *shutdown_tx.write().await = Some(tx);

        let tokens = state.local_file_tokens.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(TOKEN_CLEANUP_INTERVAL_SECS)).await;
                let mut map = tokens.write().await;
                let before = map.len();
                map.retain(|_, entry| entry.created_at.elapsed().as_secs() < TOKEN_TTL_SECS);
                let removed = before - map.len();
                if removed > 0 {
                    info!("Expired {} local file token(s)", removed);
                }
            }
        });

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    rx.await.ok();
                })
                .await
                .unwrap_or_else(|e| error!("Media server error: {}", e));
        });
    }

    pub async fn stop(&self) {
        if let Some(tx) = self.shutdown_tx.write().await.take() {
            let _ = tx.send(());
        }
    }
}

/// Parse a header value string, returning 500 on failure.
fn parse_header(value: &str) -> Result<HeaderValue, StatusCode> {
    value.parse().map_err(|_| {
        error!("Failed to parse header value: {}", value);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

/// Build standard response headers for media streaming.
fn build_media_headers(content_type: &str) -> Result<HeaderMap, StatusCode> {
    let mut h = HeaderMap::new();
    h.insert(header::CONTENT_TYPE, parse_header(content_type)?);
    h.insert(header::ACCEPT_RANGES, parse_header("bytes")?);
    Ok(h)
}

/// Validate and parse a Range header. Returns (start, end) or a 416 response.
fn parse_range(range_str: &str, file_length: u64) -> Result<(u64, u64), StatusCode> {
    let range_str = range_str.trim_start_matches("bytes=");
    let parts: Vec<&str> = range_str.split('-').collect();

    // Suffix range: bytes=-500 means last 500 bytes
    let (start, end) = if parts.first().is_none_or(|s| s.is_empty()) {
        let suffix: u64 = parts.get(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        if suffix == 0 || suffix > file_length {
            return Err(StatusCode::RANGE_NOT_SATISFIABLE);
        }
        (file_length - suffix, file_length - 1)
    } else {
        let start: u64 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let end: u64 = parts
            .get(1)
            .and_then(|s| if s.is_empty() { None } else { s.parse().ok() })
            .unwrap_or(file_length - 1);
        (start, end)
    };

    if start > end || start >= file_length || end >= file_length {
        return Err(StatusCode::RANGE_NOT_SATISFIABLE);
    }

    Ok((start, end))
}

async fn health_check() -> &'static str {
    "ok"
}

async fn stream_torrent(
    Path((torrent_id, file_idx)): Path<(usize, usize)>,
    AxumState(state): AxumState<MediaServerState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let session = {
        let guard = state.torrent_session.read().await;
        match guard.as_ref() {
            Some(s) => s.clone(),
            None => {
                return (StatusCode::SERVICE_UNAVAILABLE, "Torrent session not ready")
                    .into_response();
            }
        }
    };

    let handle = match session.get(librqbit::api::TorrentIdOrHash::Id(torrent_id)) {
        Some(h) => h,
        None => {
            return (StatusCode::NOT_FOUND, "Torrent not found").into_response();
        }
    };

    let file_details: Vec<(String, u64)> = match handle.with_metadata(|meta| {
        meta.info.iter_file_details()
            .map(|iter| {
                iter.map(|fi| {
                    let name = fi.filename.to_string()
                        .unwrap_or_else(|_| "<INVALID NAME>".to_string());
                    (name, fi.len)
                }).collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }) {
        Ok(details) => details,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Metadata error: {e}"))
                .into_response();
        }
    };

    if file_idx >= file_details.len() {
        return (StatusCode::NOT_FOUND, "File index out of range").into_response();
    }

    let (ref filename, file_length) = file_details[file_idx];
    let content_type = mime_guess::from_path(filename)
        .first_raw()
        .unwrap_or("application/octet-stream");

    let stream = match handle.clone().stream(file_idx) {
        Ok(s) => s,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Stream error: {e}"))
                .into_response();
        }
    };

    let range_header = headers.get(header::RANGE).and_then(|v| v.to_str().ok());

    match range_header {
        Some(range_str) => {
            let (start, end) = match parse_range(range_str, file_length) {
                Ok(r) => r,
                Err(status) => {
                    let cr = format!("bytes */{}", file_length);
                    let mut h = HeaderMap::new();
                    if let Ok(v) = parse_header(&cr) { h.insert(header::CONTENT_RANGE, v); }
                    return (status, h, "Invalid range").into_response();
                }
            };

            let chunk_size = end - start + 1;

            use tokio::io::AsyncSeekExt;
            let mut stream = stream;
            if let Err(e) = stream.seek(std::io::SeekFrom::Start(start)).await {
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("Seek error: {e}"))
                    .into_response();
            }

            let mut buf = vec![0u8; chunk_size as usize];
            match stream.read_exact(&mut buf).await {
                Ok(_) => {
                    let mut response_headers = match build_media_headers(content_type) {
                        Ok(h) => h,
                        Err(s) => return (s, "Header error").into_response(),
                    };
                    let cr = format!("bytes {}-{}/{}", start, end, file_length);
                    match parse_header(&cr) {
                        Ok(v) => { response_headers.insert(header::CONTENT_RANGE, v); }
                        Err(s) => return (s, "Header error").into_response(),
                    }
                    match parse_header(&chunk_size.to_string()) {
                        Ok(v) => { response_headers.insert(header::CONTENT_LENGTH, v); }
                        Err(s) => return (s, "Header error").into_response(),
                    }

                    (StatusCode::PARTIAL_CONTENT, response_headers, buf).into_response()
                }
                Err(e) => {
                    error!("Error reading torrent file: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, format!("Read error: {e}"))
                        .into_response()
                }
            }
        }
        None => {
            let stream = stream;
            let reader = tokio_util::io::ReaderStream::new(stream);
            let body = Body::from_stream(reader);

            let mut response_headers = match build_media_headers(content_type) {
                Ok(h) => h,
                Err(s) => return (s, "Header error").into_response(),
            };
            match parse_header(&file_length.to_string()) {
                Ok(v) => { response_headers.insert(header::CONTENT_LENGTH, v); }
                Err(s) => return (s, "Header error").into_response(),
            }

            (StatusCode::OK, response_headers, body).into_response()
        }
    }
}

async fn serve_local_file(
    Path(token): Path<String>,
    AxumState(state): AxumState<MediaServerState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let file_path = {
        let tokens = state.local_file_tokens.read().await;
        match tokens.get(&token) {
            Some(entry) => {
                if entry.created_at.elapsed().as_secs() >= TOKEN_TTL_SECS {
                    return (StatusCode::GONE, "Token expired").into_response();
                }
                PathBuf::from(&entry.path)
            }
            None => {
                return (StatusCode::NOT_FOUND, "Invalid token").into_response();
            }
        }
    };

    let metadata = match tokio::fs::metadata(&file_path).await {
        Ok(m) => m,
        Err(_) => {
            return (StatusCode::NOT_FOUND, "File not found").into_response();
        }
    };

    let file_length = metadata.len();
    let filename = file_path.file_name().unwrap_or_default().to_string_lossy();
    let content_type = mime_guess::from_path(&*filename)
        .first_raw()
        .unwrap_or("application/octet-stream");

    let range_header = headers.get(header::RANGE).and_then(|v| v.to_str().ok());

    match range_header {
        Some(range_str) => {
            let (start, end) = match parse_range(range_str, file_length) {
                Ok(r) => r,
                Err(status) => {
                    let cr = format!("bytes */{}", file_length);
                    let mut h = HeaderMap::new();
                    if let Ok(v) = parse_header(&cr) { h.insert(header::CONTENT_RANGE, v); }
                    return (status, h, "Invalid range").into_response();
                }
            };

            let chunk_size = end - start + 1;

            use tokio::io::AsyncSeekExt;
            let mut file = match tokio::fs::File::open(&file_path).await {
                Ok(f) => f,
                Err(e) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, format!("Open error: {e}"))
                        .into_response();
                }
            };

            if let Err(e) = file.seek(std::io::SeekFrom::Start(start)).await {
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("Seek error: {e}"))
                    .into_response();
            }

            let mut buf = vec![0u8; chunk_size as usize];
            if let Err(e) = file.read_exact(&mut buf).await {
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("Read error: {e}"))
                    .into_response();
            }

            let mut response_headers = match build_media_headers(content_type) {
                Ok(h) => h,
                Err(s) => return (s, "Header error").into_response(),
            };
            let cr = format!("bytes {}-{}/{}", start, end, file_length);
            match parse_header(&cr) {
                Ok(v) => { response_headers.insert(header::CONTENT_RANGE, v); }
                Err(s) => return (s, "Header error").into_response(),
            }
            match parse_header(&chunk_size.to_string()) {
                Ok(v) => { response_headers.insert(header::CONTENT_LENGTH, v); }
                Err(s) => return (s, "Header error").into_response(),
            }

            (StatusCode::PARTIAL_CONTENT, response_headers, buf).into_response()
        }
        None => {
            match tokio::fs::read(&file_path).await {
                Ok(data) => {
                    let mut response_headers = match build_media_headers(content_type) {
                        Ok(h) => h,
                        Err(s) => return (s, "Header error").into_response(),
                    };
                    match parse_header(&file_length.to_string()) {
                        Ok(v) => { response_headers.insert(header::CONTENT_LENGTH, v); }
                        Err(s) => return (s, "Header error").into_response(),
                    }

                    (StatusCode::OK, response_headers, data).into_response()
                }
                Err(e) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, format!("Read error: {e}"))
                        .into_response()
                }
            }
        }
    }
}

async fn serve_subtitles(
    AxumState(state): AxumState<MediaServerState>,
) -> impl IntoResponse {
    let subtitles = state.current_subtitles.read().await;
    match subtitles.as_ref() {
        Some(data) => {
            let mut headers = HeaderMap::new();
            match parse_header("text/vtt; charset=utf-8") {
                Ok(v) => { headers.insert(header::CONTENT_TYPE, v); }
                Err(s) => return (s, "Header error").into_response(),
            }
            (StatusCode::OK, headers, data.vtt_content.clone()).into_response()
        }
        None => (StatusCode::NOT_FOUND, "No subtitles loaded").into_response(),
    }
}

async fn serve_playlist(
    Path(torrent_id): Path<usize>,
    AxumState(state): AxumState<MediaServerState>,
) -> impl IntoResponse {
    let session = {
        let guard = state.torrent_session.read().await;
        match guard.as_ref() {
            Some(s) => s.clone(),
            None => {
                return (StatusCode::SERVICE_UNAVAILABLE, "Torrent session not ready")
                    .into_response();
            }
        }
    };

    let handle = match session.get(librqbit::api::TorrentIdOrHash::Id(torrent_id)) {
        Some(h) => h,
        None => {
            return (StatusCode::NOT_FOUND, "Torrent not found").into_response();
        }
    };

    let file_details: Vec<(usize, String, u64)> = match handle.with_metadata(|meta| {
        meta.info.iter_file_details()
            .map(|iter| {
                iter.enumerate()
                    .map(|(idx, fi)| {
                        let name = fi.filename.to_string()
                            .unwrap_or_else(|_| "<INVALID NAME>".to_string());
                        (idx, name, fi.len)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }) {
        Ok(details) => details,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Metadata error: {e}"))
                .into_response();
        }
    };

    // Filter to playable files (video/audio)
    let playable_files: Vec<_> = file_details
        .into_iter()
        .filter(|(_, name, _)| {
            let mime = mime_guess::from_path(name).first_raw();
            mime.is_some_and(|m| m.starts_with("video/") || m.starts_with("audio/"))
        })
        .collect();

    if playable_files.is_empty() {
        return (StatusCode::NOT_FOUND, "No playable files in torrent").into_response();
    }

    // Build M3U8 playlist
    let mut playlist = String::from("#EXTM3U\n");
    for (idx, name, duration_bytes) in playable_files {
        // Use -1 for unknown duration
        let display_name = name.rsplit('/').next().unwrap_or(&name);
        playlist.push_str(&format!("#EXTINF:-1,{}\n", display_name));
        playlist.push_str(&format!("/torrent/{}/stream/{}\n", torrent_id, idx));
        let _ = duration_bytes; // silence unused warning
    }

    let mut headers = HeaderMap::new();
    match parse_header("application/x-mpegURL") {
        Ok(v) => { headers.insert(header::CONTENT_TYPE, v); }
        Err(s) => return (s, "Header error").into_response(),
    }

    (StatusCode::OK, headers, playlist).into_response()
}
