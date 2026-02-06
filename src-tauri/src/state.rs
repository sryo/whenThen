use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::models::{AppConfig, DiscoveredDevice, SubtitleData};
use crate::services::chromecast_device::ChromecastConnection;
use crate::services::folder_watcher::FolderWatcherHandle;
use crate::services::media_server::{MediaServerHandle, TokenEntry};
use crate::services::rss::RssState;

pub struct AppState {
    pub torrent_session: Arc<RwLock<Option<Arc<librqbit::Session>>>>,
    pub discovered_devices: Arc<RwLock<HashMap<String, DiscoveredDevice>>>,
    pub active_connections: Arc<Mutex<HashMap<String, ChromecastConnection>>>,
    pub media_server: Arc<MediaServerHandle>,
    pub current_subtitles: Arc<RwLock<Option<SubtitleData>>>,
    pub config: Arc<RwLock<AppConfig>>,
    pub discovery_shutdown: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    pub local_file_tokens: Arc<RwLock<HashMap<String, TokenEntry>>>,
    pub torrent_names: Arc<RwLock<HashMap<usize, String>>>,
    pub folder_watcher: Arc<Mutex<Option<FolderWatcherHandle>>>,
    pub rss_state: Arc<RssState>,
    /// Set when the app is launched via file association or deep link.
    pub opened_via_url: Arc<AtomicBool>,
    /// Set when user explicitly requests quit (menu, Cmd+Q).
    pub quit_requested: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let media_server_port = config.media_server_port;
        Self {
            torrent_session: Arc::new(RwLock::new(None)),
            discovered_devices: Arc::new(RwLock::new(HashMap::new())),
            active_connections: Arc::new(Mutex::new(HashMap::new())),
            media_server: Arc::new(MediaServerHandle::new(media_server_port)),
            current_subtitles: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(config)),
            discovery_shutdown: Arc::new(Mutex::new(None)),
            local_file_tokens: Arc::new(RwLock::new(HashMap::new())),
            torrent_names: Arc::new(RwLock::new(HashMap::new())),
            folder_watcher: Arc::new(Mutex::new(None)),
            rss_state: Arc::new(RssState::new()),
            opened_via_url: Arc::new(AtomicBool::new(false)),
            quit_requested: Arc::new(AtomicBool::new(false)),
        }
    }
}
