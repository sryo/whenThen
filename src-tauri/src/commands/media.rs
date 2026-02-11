use serde::Serialize;
use tauri::State;

use crate::errors::Result;
use crate::models::{SubtitleInfo, SubtitleDownloadResult};
use crate::services::subtitle_handler;
use crate::services::subtitle_search;
use crate::services::torrent_engine::{get_local_ip, move_torrent_files as engine_move_files};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct MediaPlayer {
    pub id: String,
    pub name: String,
    pub path: String,
}

#[tauri::command]
pub async fn subtitle_load_file(
    state: State<'_, AppState>,
    path: String,
) -> Result<SubtitleInfo> {
    let data = subtitle_handler::load_subtitle_file(&path)?;

    let name = data.original_name.clone();
    let format = if path.ends_with(".srt") {
        "srt".to_string()
    } else {
        "vtt".to_string()
    };

    *state.current_subtitles.write().await = Some(data);

    let local_ip = get_local_ip();
    let port = state.media_server.port;
    let url = format!("http://{}:{}/subtitles.vtt", local_ip, port);

    Ok(SubtitleInfo { url, name, format })
}

#[tauri::command]
pub async fn subtitle_clear(state: State<'_, AppState>) -> Result<()> {
    *state.current_subtitles.write().await = None;
    Ok(())
}

#[tauri::command]
pub async fn media_server_url(state: State<'_, AppState>) -> Result<String> {
    let local_ip = get_local_ip();
    let port = state.media_server.port;
    Ok(format!("http://{}:{}", local_ip, port))
}

#[tauri::command]
pub async fn get_playlist_url(state: State<'_, AppState>, torrent_id: usize) -> Result<String> {
    let local_ip = get_local_ip();
    let port = state.media_server.port;
    Ok(format!("http://{}:{}/torrent/{}/playlist.m3u8", local_ip, port, torrent_id))
}

#[tauri::command]
pub async fn move_torrent_files(
    state: State<'_, AppState>,
    torrent_id: usize,
    destination: String,
) -> Result<()> {
    engine_move_files(&state, torrent_id, destination).await
}

#[tauri::command]
pub async fn subtitle_search_opensubtitles(
    state: State<'_, AppState>,
    torrent_id: usize,
    file_index: usize,
    languages: Vec<String>,
) -> Result<SubtitleDownloadResult> {
    subtitle_search::search_and_download(&state, torrent_id, file_index, languages).await
}

#[tauri::command]
pub async fn list_media_players() -> Result<Vec<MediaPlayer>> {
    #[cfg(target_os = "macos")]
    {
        Ok(launch_services::discover_media_players())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(Vec::new())
    }
}

#[cfg(target_os = "macos")]
mod launch_services {
    use super::MediaPlayer;
    use std::collections::BTreeSet;
    use std::os::raw::c_void;

    type CFTypeRef = *const c_void;
    type CFAllocatorRef = *const c_void;
    type CFArrayRef = *const c_void;
    type CFStringRef = *const c_void;
    type CFURLRef = *const c_void;
    type CFErrorRef = *const c_void;
    type CFIndex = isize;
    type Boolean = u8;

    type LSRolesMask = u32;
    const K_LS_ROLES_ALL: LSRolesMask = 0xFFFFFFFF;
    const K_CF_STRING_ENCODING_UTF8: u32 = 0x08000100;

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        static kCFAllocatorDefault: CFAllocatorRef;

        fn CFStringCreateWithBytes(
            alloc: CFAllocatorRef,
            bytes: *const u8,
            num_bytes: CFIndex,
            encoding: u32,
            is_external: Boolean,
        ) -> CFStringRef;
        fn CFStringGetCString(
            s: CFStringRef,
            buffer: *mut u8,
            buffer_size: CFIndex,
            encoding: u32,
        ) -> Boolean;
        fn CFStringGetLength(s: CFStringRef) -> CFIndex;
        fn CFArrayGetCount(arr: CFArrayRef) -> CFIndex;
        fn CFArrayGetValueAtIndex(arr: CFArrayRef, idx: CFIndex) -> *const c_void;
        fn CFURLGetFileSystemRepresentation(
            url: CFURLRef,
            resolve_against_base: Boolean,
            buffer: *mut u8,
            max_buf_len: CFIndex,
        ) -> Boolean;
        fn CFRelease(cf: CFTypeRef);
    }

    #[link(name = "CoreServices", kind = "framework")]
    extern "C" {
        fn LSCopyAllRoleHandlersForContentType(
            content_type: CFStringRef,
            role: LSRolesMask,
        ) -> CFArrayRef;
        fn LSCopyApplicationURLsForBundleIdentifier(
            bundle_id: CFStringRef,
            out_error: *mut CFErrorRef,
        ) -> CFArrayRef;
    }

    unsafe fn cfstring_from_str(s: &str) -> CFStringRef {
        CFStringCreateWithBytes(
            kCFAllocatorDefault,
            s.as_ptr(),
            s.len() as CFIndex,
            K_CF_STRING_ENCODING_UTF8,
            0,
        )
    }

    unsafe fn cfstring_to_string(s: CFStringRef) -> Option<String> {
        if s.is_null() {
            return None;
        }
        let len = CFStringGetLength(s);
        // UTF-8 can use up to 4 bytes per character
        let buf_size = (len * 4 + 1) as usize;
        let mut buf = vec![0u8; buf_size];
        if CFStringGetCString(s, buf.as_mut_ptr(), buf_size as CFIndex, K_CF_STRING_ENCODING_UTF8) != 0 {
            let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            Some(String::from_utf8_lossy(&buf[..end]).into_owned())
        } else {
            None
        }
    }

    unsafe fn cfurl_to_path(url: CFURLRef) -> Option<String> {
        if url.is_null() {
            return None;
        }
        let mut buf = [0u8; 1024];
        if CFURLGetFileSystemRepresentation(url, 1, buf.as_mut_ptr(), buf.len() as CFIndex) != 0 {
            let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            Some(String::from_utf8_lossy(&buf[..end]).into_owned())
        } else {
            None
        }
    }

    /// Collect bundle IDs for a UTI.
    unsafe fn bundle_ids_for_uti(uti: &str) -> Vec<String> {
        let cf_uti = cfstring_from_str(uti);
        if cf_uti.is_null() {
            return Vec::new();
        }
        let arr = LSCopyAllRoleHandlersForContentType(cf_uti, K_LS_ROLES_ALL);
        CFRelease(cf_uti);
        if arr.is_null() {
            return Vec::new();
        }
        let count = CFArrayGetCount(arr);
        let mut ids = Vec::with_capacity(count as usize);
        for i in 0..count {
            let val = CFArrayGetValueAtIndex(arr, i);
            if let Some(s) = cfstring_to_string(val as CFStringRef) {
                ids.push(s);
            }
        }
        CFRelease(arr);
        ids
    }

    /// Resolve a bundle ID to the first application path.
    unsafe fn app_path_for_bundle_id(bundle_id: &str) -> Option<String> {
        let cf_id = cfstring_from_str(bundle_id);
        if cf_id.is_null() {
            return None;
        }
        let mut error: CFErrorRef = std::ptr::null();
        let urls = LSCopyApplicationURLsForBundleIdentifier(cf_id, &mut error);
        CFRelease(cf_id);
        if urls.is_null() {
            return None;
        }
        let path = if CFArrayGetCount(urls) > 0 {
            let url = CFArrayGetValueAtIndex(urls, 0);
            cfurl_to_path(url as CFURLRef)
        } else {
            None
        };
        CFRelease(urls);
        path
    }

    fn display_name_from_path(path: &str) -> String {
        let file = path.rsplit('/').next().unwrap_or(path);
        file.strip_suffix(".app").unwrap_or(file).to_string()
    }

    pub fn discover_media_players() -> Vec<MediaPlayer> {
        let mut bundle_ids = BTreeSet::new();

        // Query both abstract and specific UTIs to catch all media player registrations
        const UTIS: &[&str] = &[
            "public.movie",
            "public.audio",
            "public.mpeg-4",
            "public.avi",
            "public.mp3",
            "com.apple.quicktime-movie",
            "public.mpeg-4-audio",
            "com.apple.m4a-audio",
            "org.xiph.flac",
        ];

        unsafe {
            for uti in UTIS {
                for id in bundle_ids_for_uti(uti) {
                    bundle_ids.insert(id);
                }
            }
        }

        let mut players: Vec<MediaPlayer> = bundle_ids
            .into_iter()
            .filter_map(|bid| {
                let path = unsafe { app_path_for_bundle_id(&bid) }?;
                let name = display_name_from_path(&path);
                Some(MediaPlayer {
                    id: bid.to_lowercase(),
                    name,
                    path,
                })
            })
            .collect();

        players.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        players
    }
}
