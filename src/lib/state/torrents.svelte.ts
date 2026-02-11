import type {
  TorrentSummary,
  TorrentProgress,
} from "$lib/types/torrent";

let torrents = $state<TorrentSummary[]>([]);

export const torrentsState = {
  get torrents() {
    return torrents;
  },
  get activeTorrents() {
    return torrents.filter((t) => t.state !== "completed");
  },
  get completedTorrents() {
    return torrents.filter((t) => t.state === "completed");
  },
  get totalDownloadSpeed() {
    return torrents.reduce((sum, t) => sum + t.download_speed, 0);
  },
  get totalUploadSpeed() {
    return torrents.reduce((sum, t) => sum + t.upload_speed, 0);
  },
  get totalPeers() {
    return torrents.reduce((sum, t) => sum + t.peers_connected, 0);
  },

  addTorrent(torrent: TorrentSummary) {
    const existing = torrents.findIndex((t) => t.id === torrent.id);
    if (existing >= 0) {
      torrents[existing] = torrent;
    } else {
      torrents = [...torrents, torrent];
    }
  },

  // Update a pending torrent (negative ID) to a real torrent, keeping its position
  promotePending(infoHash: string, torrent: TorrentSummary) {
    const idx = torrents.findIndex((t) => t.info_hash === infoHash && t.id < 0);
    if (idx >= 0) {
      torrents[idx] = torrent;
      return true;
    }
    return false;
  },

  removeTorrent(id: number) {
    torrents = torrents.filter((t) => t.id !== id);
  },

  updateProgress(progress: TorrentProgress) {
    const idx = torrents.findIndex((t) => t.id === progress.id);
    if (idx >= 0) {
      torrents[idx] = {
        ...torrents[idx],
        progress: progress.progress,
        download_speed: progress.download_speed,
        upload_speed: progress.upload_speed,
        peers_connected: progress.peers_connected,
        queued_peers: progress.queued_peers ?? 0,
        connecting_peers: progress.connecting_peers ?? 0,
        downloaded_bytes: progress.downloaded_bytes,
        uploaded_bytes: progress.uploaded_bytes ?? 0,
        total_bytes: progress.total_bytes,
        state: progress.state,
      };
    }
  },

  setTorrents(newTorrents: TorrentSummary[]) {
    torrents = newTorrents;
  },

  reorder(fromIndex: number, toIndex: number) {
    const copy = [...torrents];
    const [moved] = copy.splice(fromIndex, 1);
    copy.splice(toIndex, 0, moved);
    torrents = copy;
  },

  clear() {
    torrents = [];
  },
};
