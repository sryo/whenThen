use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub download_directory: String,
    pub theme: ThemeMode,
    pub color_scheme: String,
    pub always_on_top: bool,
    pub auto_discover: bool,
    pub max_download_speed: u64,
    pub max_upload_speed: u64,
    pub media_server_port: u16,
    pub auto_play_next: bool,
    pub repeat_mode: RepeatMode,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RepeatMode {
    None,
    One,
    All,
}

fn default_subtitle_languages() -> Vec<String> {
    vec!["en".to_string()]
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
            always_on_top: false,
            auto_discover: true,
            max_download_speed: 0,
            max_upload_speed: 0,
            media_server_port: 9080,
            auto_play_next: true,
            repeat_mode: RepeatMode::None,
            subtitle_languages: default_subtitle_languages(),
            opensubtitles_api_key: String::new(),
            enable_upnp: true,
            listen_port: 4240,
            watch_folders: vec![],
            watch_folders_enabled: false,
            incomplete_directory: String::new(),
            max_concurrent_tasks: 0,
        }
    }
}
