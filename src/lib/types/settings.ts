export interface AppSettings {
  download_directory: string;
  theme: ThemeMode;
  color_scheme: string;
  auto_discover: boolean;
  max_download_speed: number;
  max_upload_speed: number;
  media_server_port: number;
  auto_play_next: boolean;
  subtitle_languages: string[];
  opensubtitles_api_key: string;
  enable_upnp: boolean;
  listen_port: number;
  watch_folders: string[];
  watch_folders_enabled: boolean;
  incomplete_directory: string;
  max_concurrent_tasks: number;
  picker_countdown_seconds: number;
  skip_template_picker: boolean;
  delete_torrent_file_on_add: boolean;
  show_tray_icon: boolean;
  default_cast_device: string;
  default_media_player: string;
  default_move_destination: string;
}

export type ThemeMode = "light" | "dark" | "system";

export const DEFAULT_SETTINGS: AppSettings = {
  download_directory: "",
  theme: "system",
  color_scheme: "auto",
  auto_discover: true,
  max_download_speed: 0,
  max_upload_speed: 0,
  media_server_port: 9080,
  auto_play_next: true,
  subtitle_languages: ["en"],
  opensubtitles_api_key: "",
  enable_upnp: true,
  listen_port: 4240,
  watch_folders: [],
  watch_folders_enabled: false,
  incomplete_directory: "",
  max_concurrent_tasks: 0,
  picker_countdown_seconds: 5,
  skip_template_picker: false,
  delete_torrent_file_on_add: false,
  show_tray_icon: true,
  default_cast_device: "",
  default_media_player: "",
  default_move_destination: "",
};
