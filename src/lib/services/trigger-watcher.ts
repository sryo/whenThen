// Watches Tauri events and fires playlets based on non-default triggers.
import { listen } from "@tauri-apps/api/event";
import { playletsState, derivePlayletName } from "$lib/state/playlets.svelte";
import { tasksState } from "$lib/state/tasks.svelte";
import { torrentsState } from "$lib/state/torrents.svelte";
import { executePipeline } from "./execution-pipeline";
import type { TorrentProgress } from "$lib/types/torrent";

let unlisteners: (() => void)[] = [];

// Track which (playletId, torrentId) pairs have already fired for seeding ratio
const firedRatio = new Set<string>();

function ratioKey(playletId: string, torrentId: number): string {
  return `${playletId}:${torrentId}`;
}

function findMatchingPlaylets(triggerType: string, torrentName: string, totalBytes?: number, fileCount?: number) {
  return playletsState.playlets.filter((p) => {
    if (!p.enabled) return false;
    if (p.trigger.type !== triggerType) return false;
    return playletsState.matchesConditions(p, torrentName, totalBytes, fileCount);
  });
}

function createAndExecuteTask(playletId: string, torrentId: number, torrentName: string) {
  const playlet = playletsState.getById(playletId);
  if (!playlet) return;
  const task = tasksState.createTask(torrentId, torrentName, playlet.id, derivePlayletName(playlet));
  executePipeline(task.id);
}

export async function setupTriggerWatcher() {
  // Folder watch trigger — fired by the Rust file watcher service
  unlisteners.push(
    await listen<{ path: string; torrent_id: number; torrent_name: string }>(
      "folder_watch:torrent_detected",
      (event) => {
        const { path, torrent_id, torrent_name } = event.payload;

        const matches = playletsState.playlets.filter((p) => {
          if (!p.enabled) return false;
          if (p.trigger.type !== "folder_watch") return false;
          // If playlet specifies a watchFolder, only match files from that folder
          if (p.trigger.watchFolder) {
            if (!path.startsWith(p.trigger.watchFolder)) return false;
          }
          return playletsState.matchesConditions(p, torrent_name);
        });

        for (const playlet of matches) {
          createAndExecuteTask(playlet.id, torrent_id, torrent_name);
        }
      },
    ),
  );

  // Download complete trigger
  unlisteners.push(
    await listen<number>("torrent:completed", (event) => {
      const torrentId = event.payload;
      const torrent = torrentsState.torrents.find((t) => t.id === torrentId);
      if (!torrent) return;

      const matches = findMatchingPlaylets(
        "download_complete",
        torrent.name,
        torrent.total_bytes,
        torrent.file_count,
      );
      for (const playlet of matches) {
        createAndExecuteTask(playlet.id, torrentId, torrent.name);
      }
    }),
  );

  // Metadata received trigger
  unlisteners.push(
    await listen<{ id: number; name: string }>("torrent:metadata", (event) => {
      const { id, name } = event.payload;
      const torrent = torrentsState.torrents.find((t) => t.id === id);

      const matches = findMatchingPlaylets(
        "metadata_received",
        name,
        torrent?.total_bytes,
        torrent?.file_count,
      );
      for (const playlet of matches) {
        createAndExecuteTask(playlet.id, id, name);
      }
    }),
  );

  // Seeding ratio trigger — piggyback on progress updates
  unlisteners.push(
    await listen<TorrentProgress>("torrent:progress", (event) => {
      const progress = event.payload;
      if (progress.state !== "completed") return;
      if (progress.total_bytes === 0) return;

      const uploadedBytes = (progress as any).uploaded_bytes ?? 0;
      if (uploadedBytes === 0) return;

      const ratio = uploadedBytes / progress.total_bytes;

      const torrent = torrentsState.torrents.find((t) => t.id === progress.id);
      if (!torrent) return;

      const candidates = playletsState.playlets.filter((p) => {
        if (!p.enabled) return false;
        if (p.trigger.type !== "seeding_ratio") return false;
        const targetRatio = p.trigger.seedingRatio ?? 1.0;
        if (ratio < targetRatio) return false;
        if (firedRatio.has(ratioKey(p.id, progress.id))) return false;
        return playletsState.matchesConditions(p, torrent.name, torrent.total_bytes, torrent.file_count);
      });

      for (const playlet of candidates) {
        firedRatio.add(ratioKey(playlet.id, progress.id));
        createAndExecuteTask(playlet.id, progress.id, torrent.name);
      }
    }),
  );
}

export function cleanupTriggerWatcher() {
  unlisteners.forEach((unlisten) => unlisten());
  unlisteners = [];
  firedRatio.clear();
}
