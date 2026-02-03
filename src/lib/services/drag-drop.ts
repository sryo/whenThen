import {
  torrentAddMagnet,
  torrentAddFile,
  torrentAddBytes,
  subtitleLoadFile,
  playbackCastLocalFile,
} from "./tauri-commands";
import { devicesState } from "$lib/state/devices.svelte";
import { subtitlesState } from "$lib/state/subtitles.svelte";
import { uiState } from "$lib/state/ui.svelte";
import type { TorrentAddedResponse } from "$lib/types/torrent";

export type DropContentType =
  | "magnet"
  | "torrent"
  | "media"
  | "subtitle"
  | "unknown";

const MEDIA_EXTENSIONS = [
  "mp4",
  "m4v",
  "mkv",
  "avi",
  "mov",
  "mp3",
  "m4a",
  "flac",
  "webm",
  "ogg",
];
const SUBTITLE_EXTENSIONS = ["srt", "vtt"];
const TORRENT_EXTENSION = "torrent";

export function classifyContent(input: string): DropContentType {
  const trimmed = input.trim();

  if (trimmed.toLowerCase().startsWith("magnet:")) {
    return "magnet";
  }

  const ext = trimmed.split(".").pop()?.toLowerCase() ?? "";

  if (ext === TORRENT_EXTENSION) {
    return "torrent";
  }

  if (SUBTITLE_EXTENSIONS.includes(ext)) {
    return "subtitle";
  }

  if (MEDIA_EXTENSIONS.includes(ext)) {
    return "media";
  }

  // Check if it's an HTTP link to a torrent
  if (
    trimmed.toLowerCase().startsWith("http") &&
    trimmed.toLowerCase().includes("torrent")
  ) {
    return "torrent";
  }

  return "unknown";
}

export async function handleDroppedContent(input: string): Promise<TorrentAddedResponse | null> {
  const type = classifyContent(input);

  try {
    switch (type) {
      case "magnet": {
        const response = await torrentAddMagnet(input);
        return response;
      }

      case "torrent": {
        if (input.toLowerCase().startsWith("http")) {
          // HTTP link to torrent - try as magnet first
          const response = await torrentAddMagnet(input);
          return response;
        } else {
          const response = await torrentAddFile(input);
          return response;
        }
      }

      case "subtitle": {
        const info = await subtitleLoadFile(input);
        subtitlesState.setSubtitle(info);
        uiState.addToast(`Loaded subtitle: ${info.name}`, "success");
        return null;
      }

      case "media": {
        const connected = devicesState.connectedDevices;
        if (connected.length > 0) {
          await playbackCastLocalFile(connected[0].id, input);
          uiState.addToast("Playing on device", "info");
        } else {
          uiState.addToast("No device connected", "warning");
        }
        return null;
      }

      case "unknown":
        uiState.addToast("Can't use this file type", "warning");
        return null;
    }
  } catch (err: any) {
    const msg = err?.message || String(err);
    uiState.addToast(`Something went wrong: ${msg}`, "error");
    return null;
  }
}

/** Read a File object's bytes and add it as a torrent. Only handles .torrent files. */
export async function handleDroppedFile(file: File): Promise<TorrentAddedResponse | null> {
  if (!file.name.toLowerCase().endsWith(".torrent")) {
    uiState.addToast("Can't use this file type", "warning");
    return null;
  }

  try {
    const buffer = await file.arrayBuffer();
    const bytes = Array.from(new Uint8Array(buffer));
    return await torrentAddBytes(bytes);
  } catch (err: any) {
    const msg = err?.message || String(err);
    uiState.addToast(`Could not add torrent: ${msg}`, "error");
    return null;
  }
}
