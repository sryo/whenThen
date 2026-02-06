// Playback queue for Chromecast casting.

export interface QueueItem {
  id: string;
  torrentId: number;
  fileIndex: number;
  name: string;
  addedAt: number;
}

let items = $state<QueueItem[]>([]);

export const queueState = {
  get items() {
    return items;
  },
  get isEmpty() {
    return items.length === 0;
  },
  get nextItems() {
    return items.slice(0, 3);
  },

  addToQueue(torrentId: number, fileIndex: number, name: string) {
    const item: QueueItem = {
      id: `${torrentId}-${fileIndex}-${Date.now()}`,
      torrentId,
      fileIndex,
      name,
      addedAt: Date.now(),
    };
    items = [...items, item];
    return item;
  },

  addBatch(torrentId: number, files: { index: number; name: string }[]) {
    const now = Date.now();
    const newItems = files.map((f, i) => ({
      id: `${torrentId}-${f.index}-${now}-${i}`,
      torrentId,
      fileIndex: f.index,
      name: f.name,
      addedAt: now + i,
    }));
    items = [...items, ...newItems];
    return newItems;
  },

  removeFromQueue(id: string) {
    items = items.filter((i) => i.id !== id);
  },

  clearQueue() {
    items = [];
  },

  playNext(): QueueItem | null {
    if (items.length === 0) return null;
    const next = items[0];
    items = items.slice(1);
    return next;
  },
};
