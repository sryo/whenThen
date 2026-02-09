import { invoke } from "@tauri-apps/api/core";
import type {
  TorrentAddedResponse,
  TorrentSummary,
  TorrentFileInfo,
  TorrentAddOptions,
} from "$lib/types/torrent";
import type { SubtitleInfo, MediaPlayer } from "$lib/types/playback";
import type { AppSettings } from "$lib/types/settings";

const DEFAULT_TIMEOUT_MS = 30_000;

/** Wrap a promise with a timeout. Rejects if not resolved within `ms`. */
function withTimeout<T>(promise: Promise<T>, ms: number = DEFAULT_TIMEOUT_MS): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error("Command timed out")), ms);
    promise.then(
      (v) => { clearTimeout(timer); resolve(v); },
      (e) => { clearTimeout(timer); reject(e); },
    );
  });
}

function invokeWithTimeout<T>(cmd: string, args?: Record<string, unknown>, ms?: number): Promise<T> {
  return withTimeout(invoke<T>(cmd, args), ms);
}

/** Deduplication: prevents the same command from running concurrently. */
const inflight = new Map<string, Promise<unknown>>();
function dedup<T>(key: string, fn: () => Promise<T>): Promise<T> {
  const existing = inflight.get(key);
  if (existing) return existing as Promise<T>;
  const p = fn().finally(() => inflight.delete(key));
  inflight.set(key, p);
  return p;
}

// Torrent commands
export async function torrentAddMagnet(
  magnetUrl: string,
  options?: TorrentAddOptions,
): Promise<TorrentAddedResponse> {
  return invokeWithTimeout("torrent_add_magnet", { magnet_url: magnetUrl, options }, 60_000);
}

export async function torrentAddFile(
  path: string,
  options?: TorrentAddOptions,
): Promise<TorrentAddedResponse> {
  return invokeWithTimeout("torrent_add_file", { path, options }, 60_000);
}

export async function torrentAddBytes(
  fileBytes: number[],
  options?: TorrentAddOptions,
): Promise<TorrentAddedResponse> {
  return invokeWithTimeout("torrent_add_bytes", { file_bytes: fileBytes, options }, 60_000);
}

export async function torrentSyncRestored(): Promise<TorrentSummary[]> {
  return invoke("torrent_sync_restored");
}

export async function torrentFiles(id: number): Promise<TorrentFileInfo[]> {
  return invoke("torrent_files", { id });
}

export async function torrentPause(id: number): Promise<void> {
  return dedup(`torrent_pause:${id}`, () => invoke("torrent_pause", { id }));
}

export async function torrentResume(id: number): Promise<void> {
  return dedup(`torrent_resume:${id}`, () => invoke("torrent_resume", { id }));
}

export async function torrentRecheck(id: number): Promise<TorrentAddedResponse> {
  return invokeWithTimeout("torrent_recheck", { id }, 60_000);
}

export async function torrentDelete(
  id: number,
  deleteFiles: boolean,
): Promise<void> {
  return dedup(`torrent_delete:${id}`, () => invoke("torrent_delete", { id, deleteFiles }));
}

// Chromecast commands
export async function chromecastStartDiscovery(): Promise<void> {
  return invoke("chromecast_start_discovery");
}

export async function chromecastConnect(deviceId: string): Promise<void> {
  return invoke("chromecast_connect", { deviceId });
}

// Playback commands
export async function playbackCastTorrent(
  deviceId: string,
  torrentId: number,
  fileIndex: number,
): Promise<void> {
  return invoke("playback_cast_torrent", { deviceId, torrentId, fileIndex });
}

export async function playbackCastLocalFile(
  deviceId: string,
  filePath: string,
): Promise<void> {
  return invoke("playback_cast_local_file", { deviceId, filePath });
}

// Playback control commands
export async function playbackPlay(deviceId: string): Promise<void> {
  return invoke("playback_play", { deviceId });
}

export async function playbackPause(deviceId: string): Promise<void> {
  return invoke("playback_pause", { deviceId });
}

export async function playbackStop(deviceId: string): Promise<void> {
  return invoke("playback_stop", { deviceId });
}

export async function playbackSeek(deviceId: string, position: number): Promise<void> {
  return invoke("playback_seek", { deviceId, position });
}

export async function playbackSetVolume(deviceId: string, volume: number): Promise<void> {
  return invoke("playback_set_volume", { deviceId, volume });
}

// Local playback commands
export async function playbackOpenInApp(
  torrentId: number,
  fileIndex: number,
  appName: string,
): Promise<void> {
  return invoke("playback_open_in_app", { torrentId, fileIndex, appName });
}

export async function listMediaPlayers(): Promise<MediaPlayer[]> {
  return invoke("list_media_players");
}

// Move commands
export async function moveTorrentFiles(
  torrentId: number,
  destination: string,
): Promise<void> {
  return invoke("move_torrent_files", { torrentId, destination });
}

// Media commands
export async function subtitleLoadFile(
  path: string,
): Promise<SubtitleInfo> {
  return invoke("subtitle_load_file", { path });
}

export async function getPlaylistUrl(torrentId: number): Promise<string> {
  return invoke("get_playlist_url", { torrentId });
}

// Subtitle search commands
export interface SubtitleDownloadResult {
  file_name: string;
  file_path: string;
}

export async function subtitleSearchOpensubtitles(
  torrentId: number,
  fileIndex: number,
  languages: string[],
): Promise<SubtitleDownloadResult> {
  return invokeWithTimeout("subtitle_search_opensubtitles", { torrentId, fileIndex, languages }, 30_000);
}

// Automation commands
export async function checkAutomationPermission(): Promise<string> {
  return invoke("check_automation_permission");
}

export async function runShortcut(name: string, inputJson: string): Promise<string> {
  return invoke("run_shortcut", { name, inputJson });
}

export async function runApplescript(script: string): Promise<string> {
  return invoke("run_applescript", { script });
}

export async function runShellCommand(command: string): Promise<string> {
  return invoke("run_shell_command", { command });
}

export async function openSystemSettings(panel: string): Promise<void> {
  // Opens macOS System Settings to a specific panel
  // panel examples: "Privacy_Automation", "Privacy_Accessibility"
  await runShellCommand(`open "x-apple.systempreferences:com.apple.preference.security?${panel}"`);
}

// Association commands
export interface FileAssociationStatus {
  torrent_files: boolean;
  magnet_links: boolean;
}

export async function checkFileAssociations(): Promise<FileAssociationStatus> {
  return invoke("check_file_associations");
}

export async function setDefaultForTorrents(): Promise<void> {
  return invoke("set_default_for_torrents");
}

export async function setDefaultForMagnets(): Promise<void> {
  return invoke("set_default_for_magnets");
}

// App launch commands
export async function checkOpenedViaUrl(): Promise<boolean> {
  return invoke("check_opened_via_url");
}

// Settings commands
export async function settingsGet(): Promise<AppSettings> {
  return invoke("settings_get");
}

export async function settingsUpdate(
  config: AppSettings,
): Promise<AppSettings> {
  return invoke("settings_update", { config });
}

// i18n commands
export async function getTranslations(
  locale?: string,
): Promise<Record<string, Record<string, string>>> {
  return invoke("get_translations", { locale: locale ?? null });
}

// Demo/screenshot commands
export async function rssSeedDemo(): Promise<void> {
  return invoke("rss_seed_demo");
}
