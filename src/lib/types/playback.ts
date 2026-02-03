export interface PlaybackStatusResponse {
  device_id: string;
  state: PlaybackState;
  current_time: number;
  duration: number;
  volume: number;
  is_muted: boolean;
  media_title: string | null;
  content_type: string | null;
}

export type PlaybackState = "idle" | "buffering" | "playing" | "paused";

export interface SubtitleInfo {
  url: string;
  name: string;
  format: string;
}

export interface MediaPlayer {
  id: string;
  name: string;
  path: string;
}
