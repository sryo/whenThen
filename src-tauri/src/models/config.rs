use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub download_directory: String,
    pub theme: ThemeMode,
    pub color_scheme: String,
    pub auto_discover: bool,
    pub max_download_speed: u64,
    pub max_upload_speed: u64,
    pub media_server_port: u16,
    pub auto_play_next: bool,
    #[serde(default = "default_subtitle_languages")]
    pub subtitle_languages: Vec<String>,
    #[serde(default)]
    pub opensubtitles_api_key: String,
    #[serde(default)]
    pub enable_upnp: bool,
    #[serde(default = "default_listen_port")]
    pub listen_port: u16,
    #[serde(default)]
    pub watch_folders: Vec<String>,
    #[serde(default)]
    pub watch_folders_enabled: bool,
    /// Separate folder for incomplete downloads (empty = same as download_directory)
    #[serde(default)]
    pub incomplete_directory: String,
    /// Max tasks executing at the same time (0 = unlimited)
    #[serde(default)]
    pub max_concurrent_tasks: u32,
    #[serde(default)]
    pub delete_torrent_file_on_add: bool,
    #[serde(default = "default_true")]
    pub show_tray_icon: bool,
    #[serde(default)]
    pub default_cast_device: String,
    #[serde(default)]
    pub default_media_player: String,
    #[serde(default)]
    pub default_move_destination: String,
    /// RSS feed check interval in minutes (default 15)
    #[serde(default = "default_rss_interval")]
    pub rss_check_interval_minutes: u32,
    #[serde(default = "default_locale")]
    pub locale: String,
}

fn default_rss_interval() -> u32 {
    15
}

fn default_locale() -> String {
    "system".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

fn default_subtitle_languages() -> Vec<String> {
    vec!["en".to_string()]
}

fn default_true() -> bool {
    true
}

fn default_listen_port() -> u16 {
    4240
}

impl Default for AppConfig {
    fn default() -> Self {
        let download_dir = dirs::download_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"))
            .to_string_lossy()
            .to_string();

        Self {
            download_directory: download_dir,
            theme: ThemeMode::System,
            color_scheme: "auto".to_string(),
            auto_discover: true,
            max_download_speed: 0,
            max_upload_speed: 0,
            media_server_port: 9080,
            auto_play_next: true,
            subtitle_languages: default_subtitle_languages(),
            opensubtitles_api_key: String::new(),
            enable_upnp: true,
            listen_port: 4240,
            watch_folders: vec![],
            watch_folders_enabled: false,
            incomplete_directory: String::new(),
            max_concurrent_tasks: 0,
            delete_torrent_file_on_add: false,
            show_tray_icon: true,
            default_cast_device: String::new(),
            default_media_player: String::new(),
            default_move_destination: String::new(),
            rss_check_interval_minutes: default_rss_interval(),
            locale: default_locale(),
        }
    }
}
