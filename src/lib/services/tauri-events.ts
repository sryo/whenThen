import { listen } from "@tauri-apps/api/event";
import { devicesState } from "$lib/state/devices.svelte";
import { torrentsState } from "$lib/state/torrents.svelte";
import { playbackState } from "$lib/state/playback.svelte";
import { uiState } from "$lib/state/ui.svelte";
import { tasksState } from "$lib/state/tasks.svelte";
import { playletsState } from "$lib/state/playlets.svelte";
import { executePipeline, tryExecuteNext } from "./execution-pipeline";
import { assignTorrentToPlaylet, shouldSkipAutoAssign } from "./playlet-assignment";
import type {
  DeviceFoundEvent,
  DeviceLostEvent,
  DeviceConnectedEvent,
  DeviceDisconnectedEvent,
  TorrentProgress,
} from "$lib/types";
import type { PlaybackStatusResponse } from "$lib/types/playback";
import type { TorrentAddedResponse } from "$lib/types/torrent";

let unlisteners: (() => void)[] = [];

export async function setupEventListeners() {
  // Chromecast events
  unlisteners.push(
    await listen<DeviceFoundEvent>("chromecast:device-found", (event) => {
      devicesState.addDevice({
        ...event.payload,
        status: "discovered",
      });
    }),
  );

  unlisteners.push(
    await listen<DeviceLostEvent>("chromecast:device-lost", (event) => {
      devicesState.removeDevice(event.payload.id);
    }),
  );

  unlisteners.push(
    await listen<DeviceConnectedEvent>("chromecast:connected", (event) => {
      devicesState.updateDeviceStatus(event.payload.id, "connected");
      uiState.addToast(`Connected to ${event.payload.name}`, "success");
    }),
  );

  unlisteners.push(
    await listen<DeviceDisconnectedEvent>(
      "chromecast:disconnected",
      (event) => {
        devicesState.updateDeviceStatus(event.payload.id, "discovered");
        uiState.addToast(`Device disconnected`, "info");
      },
    ),
  );

  unlisteners.push(
    await listen<{ id: string; error: string }>(
      "chromecast:connection-error",
      (event) => {
        devicesState.updateDeviceStatus(event.payload.id, "error");
        uiState.addToast(`Couldn't connect: ${event.payload.error}`, "error");
      },
    ),
  );

  // Torrent events
  unlisteners.push(
    await listen<TorrentAddedResponse>("torrent:added", (event) => {
      const t = event.payload;
      torrentsState.addTorrent({
        id: t.id,
        name: t.name,
        info_hash: t.info_hash,
        state: "initializing",
        progress: 0,
        download_speed: 0,
        upload_speed: 0,
        peers_connected: 0,
        total_bytes: 0,
        downloaded_bytes: 0,
        file_count: t.files.length,
      });
      uiState.addToast(`${t.name} added`, "success");

      // Skip auto-assignment if a manual card drop is in flight
      if (shouldSkipAutoAssign()) return;

      // Auto-assign or show playlet picker (only enabled playlets with torrent_added trigger)
      const eligible = playletsState.playlets.filter(
        (p) => p.enabled && p.trigger.type === "torrent_added"
      );
      if (eligible.length === 1) {
        assignTorrentToPlaylet(eligible[0].id, t);
      } else if (eligible.length >= 2) {
        uiState.showPlayletPicker(t);
      }
    }),
  );

  unlisteners.push(
    await listen<TorrentProgress>("torrent:progress", (event) => {
      torrentsState.updateProgress(event.payload);
    }),
  );

  unlisteners.push(
    await listen<number>("torrent:completed", (event) => {
      const torrentId = event.payload;
      uiState.addToast("Download finished", "success");

      // Trigger execution pipeline if a task exists for this torrent
      const task = tasksState.getByTorrentId(torrentId);
      if (task && task.playletId) {
        tryExecuteNext(task.id);
      }
    }),
  );

  unlisteners.push(
    await listen<{ id: number; error: string }>("torrent:error", (event) => {
      uiState.addToast(`Download failed: ${event.payload.error}`, "error");
    }),
  );

  // Playback events
  unlisteners.push(
    await listen<PlaybackStatusResponse>(
      "playback:status-changed",
      (event) => {
        playbackState.setStatus(event.payload);
      },
    ),
  );

  unlisteners.push(
    await listen<{ device_id: string }>("playback:finished", (_event) => {
      playbackState.clear();
      uiState.addToast("Done playing", "info");
    }),
  );

  unlisteners.push(
    await listen<{ device_id: string; error: string }>(
      "playback:error",
      (event) => {
        uiState.addToast(`Couldn't play: ${event.payload.error}`, "error");
      },
    ),
  );
}

export function cleanupEventListeners() {
  unlisteners.forEach((unlisten) => unlisten());
  unlisteners = [];
}
