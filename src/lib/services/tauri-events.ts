import { listen } from "@tauri-apps/api/event";
import { devicesState } from "$lib/state/devices.svelte";
import { torrentsState } from "$lib/state/torrents.svelte";
import { playbackState } from "$lib/state/playback.svelte";
import { uiState } from "$lib/state/ui.svelte";
import { tasksState } from "$lib/state/tasks.svelte";
import { tryExecuteNext } from "./execution-pipeline";
import { assignTorrentToPlaylet, findBestMatch, shouldSkipAutoAssign } from "./playlet-assignment";
import { initNotifications, notifyDownloadComplete, notifyRssMatch } from "./notifications";
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
  // Initialize notifications
  await initNotifications();
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

      // Manual card drop handles its own assignment
      if (shouldSkipAutoAssign()) return;

      const match = findBestMatch(t.name, "torrent_added", undefined, t.files.length);
      if (match) {
        assignTorrentToPlaylet(match.id, t);
      }
    }),
  );

  unlisteners.push(
    await listen<{ old_id: number; new_id: number; name: string }>(
      "torrent:rechecked",
      (event) => {
        const { old_id, new_id, name } = event.payload;
        torrentsState.removeTorrent(old_id);
        torrentsState.addTorrent({
          id: new_id,
          name,
          info_hash: "",
          state: "initializing",
          progress: 0,
          download_speed: 0,
          upload_speed: 0,
          peers_connected: 0,
          total_bytes: 0,
          downloaded_bytes: 0,
          file_count: 0,
        });
        tasksState.updateTorrentId(old_id, new_id);
        uiState.addToast("Rechecking pieces", "info");
      },
    ),
  );

  unlisteners.push(
    await listen<{ old_id: number; new_id: number; name: string }>(
      "torrent:files-updated",
      (event) => {
        const { old_id, new_id, name } = event.payload;
        torrentsState.removeTorrent(old_id);
        torrentsState.addTorrent({
          id: new_id,
          name,
          info_hash: "",
          state: "initializing",
          progress: 0,
          download_speed: 0,
          upload_speed: 0,
          peers_connected: 0,
          total_bytes: 0,
          downloaded_bytes: 0,
          file_count: 0,
        });
        tasksState.updateTorrentId(old_id, new_id);
        uiState.addToast("File selection updated", "info");
      },
    ),
  );

  unlisteners.push(
    await listen<TorrentProgress>("torrent:progress", (event) => {
      torrentsState.updateProgress(event.payload);
    }),
  );

  unlisteners.push(
    await listen<number>("torrent:completed", async (event) => {
      const torrentId = event.payload;
      uiState.addToast("Download finished", "success");

      // Send native notification
      const torrent = torrentsState.torrents.find((t) => t.id === torrentId);
      if (torrent) {
        await notifyDownloadComplete(torrent.name);
      }

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

  // RSS events
  unlisteners.push(
    await listen<{ feed_name: string; title: string; torrent_id: number }>(
      "rss:match",
      async (event) => {
        const { feed_name, title } = event.payload;
        await notifyRssMatch(feed_name, title);
      },
    ),
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
