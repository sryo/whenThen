export interface TorrentAddedResponse {
  id: number;
  name: string;
  info_hash: string;
  files: TorrentFileInfo[];
}

export interface TorrentSummary {
  id: number;
  name: string;
  info_hash: string;
  state: TorrentState;
  progress: number;
  download_speed: number;
  upload_speed: number;
  peers_connected: number;
  queued_peers: number;
  connecting_peers: number;
  total_bytes: number;
  downloaded_bytes: number;
  uploaded_bytes?: number;
  file_count: number;
}

export interface TorrentDetails extends TorrentSummary {
  files: TorrentFileInfo[];
  added_at: string;
  output_folder: string;
}

export interface TorrentFileInfo {
  index: number;
  name: string;
  path: string;
  length: number;
  is_playable: boolean;
  mime_type: string | null;
  stream_url: string | null;
}

export interface TorrentAddOptions {
  output_folder?: string;
  only_files?: number[];
}

export interface TorrentProgress {
  id: number;
  progress: number;
  download_speed: number;
  upload_speed: number;
  peers_connected: number;
  queued_peers: number;
  connecting_peers: number;
  downloaded_bytes: number;
  uploaded_bytes: number;
  total_bytes: number;
  state: TorrentState;
}

export type TorrentState =
  | "initializing"
  | "downloading"
  | "paused"
  | "completed"
  | "error";
