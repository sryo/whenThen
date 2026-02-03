import type { ViewName } from "$lib/types/ui";
import type { Toast } from "$lib/types/ui";
import type { TorrentAddedResponse } from "$lib/types/torrent";

export type { ViewName, Toast };

let activeView = $state<ViewName>("playlets");
let toasts = $state<Toast[]>([]);
let pendingTorrent = $state<TorrentAddedResponse | null>(null);
let flyingPip = $state<{ id: string; fromX: number; fromY: number } | null>(null);

export const uiState = {
  get activeView() {
    return activeView;
  },
  get toasts() {
    return toasts;
  },
  get pendingTorrent() {
    return pendingTorrent;
  },
  get flyingPip() {
    return flyingPip;
  },

  setView(view: ViewName) {
    activeView = view;
  },

  addToast(message: string, level: Toast["level"] = "info") {
    const id = crypto.randomUUID();
    toasts = [...toasts, { id, message, level }];
    setTimeout(() => {
      toasts = toasts.filter((t) => t.id !== id);
    }, 5000);
  },

  removeToast(id: string) {
    toasts = toasts.filter((t) => t.id !== id);
  },

  showPlayletPicker(response: TorrentAddedResponse) {
    pendingTorrent = response;
  },

  clearPlayletPicker() {
    pendingTorrent = null;
  },

  triggerFlyingPip(fromX: number, fromY: number) {
    flyingPip = { id: crypto.randomUUID(), fromX, fromY };
  },

  clearFlyingPip() {
    flyingPip = null;
  },
};
