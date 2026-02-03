export interface AppSettings {
  download_directory: string;
  theme: ThemeMode;
  color_scheme: string;
  always_on_top: boolean;
  auto_discover: boolean;
  max_download_speed: number;
  max_upload_speed: number;
  media_server_port: number;
  auto_play_next: boolean; // TODO: ~10% — field defined, no UI toggle or playback logic
  repeat_mode: RepeatMode; // TODO: ~5% — field defined, no UI toggle or playback logic
  subtitle_languages: string[];
  opensubtitles_api_key: string;
  enable_upnp: boolean;
  listen_port: number;
  watch_folders: string[];
  watch_folders_enabled: boolean;
  incomplete_directory: string;
  max_concurrent_tasks: number;
  skip_template_picker: boolean;
}

export type ThemeMode = "light" | "dark" | "system";
export type RepeatMode = "none" | "one" | "all";

export const DEFAULT_SETTINGS: AppSettings = {
  download_directory: "",
  theme: "system",
  color_scheme: "auto",
  always_on_top: false,
  auto_discover: true,
  max_download_speed: 0,
  max_upload_speed: 0,
  media_server_port: 9080,
  auto_play_next: true,
  repeat_mode: "none",
  subtitle_languages: ["en"],
  opensubtitles_api_key: "",
  enable_upnp: true,
  listen_port: 4240,
  watch_folders: [],
  watch_folders_enabled: false,
  incomplete_directory: "",
  max_concurrent_tasks: 0,
  skip_template_picker: false,
};
