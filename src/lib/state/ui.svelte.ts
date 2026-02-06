import type { ViewName } from "$lib/types/ui";
import type { Toast } from "$lib/types/ui";
import type { TorrentAddedResponse } from "$lib/types/torrent";
import { load } from "@tauri-apps/plugin-store";

export type { ViewName, Toast };

let storeInstance: Awaited<ReturnType<typeof load>> | null = null;

async function getStore() {
  if (!storeInstance) {
    storeInstance = await load("ui.json", { autoSave: true, defaults: {} });
  }
  return storeInstance;
}

class UiState {
  activeView = $state<ViewName>("inbox");
  toasts = $state<Toast[]>([]);
  pendingTorrent = $state<TorrentAddedResponse | null>(null);
  flyingPip = $state<{ id: string; fromX: number; fromY: number } | null>(null);
  showSettings = $state(false);
  collapsedSections = $state<Set<string>>(new Set());

  async loadPersistedState() {
    try {
      const store = await getStore();
      const collapsed = await store.get<string[]>("collapsedSections");
      if (collapsed && Array.isArray(collapsed)) {
        this.collapsedSections = new Set(collapsed);
      }
    } catch {}
  }

  async persistCollapsedSections() {
    try {
      const store = await getStore();
      await store.set("collapsedSections", Array.from(this.collapsedSections));
    } catch {}
  }

  toggleSection(name: string) {
    if (this.collapsedSections.has(name)) {
      this.collapsedSections.delete(name);
    } else {
      this.collapsedSections.add(name);
    }
    this.collapsedSections = new Set(this.collapsedSections);
    this.persistCollapsedSections();
  }

  isSectionCollapsed(name: string): boolean {
    return this.collapsedSections.has(name);
  }

  setView(view: ViewName) {
    this.activeView = view;
  }

  addToast(message: string, level: Toast["level"] = "info") {
    const id = crypto.randomUUID();
    this.toasts = [...this.toasts, { id, message, level }];
    setTimeout(() => {
      this.toasts = this.toasts.filter((t) => t.id !== id);
    }, 5000);
  }

  removeToast(id: string) {
    this.toasts = this.toasts.filter((t) => t.id !== id);
  }

  showPlayletPicker(response: TorrentAddedResponse) {
    this.pendingTorrent = response;
  }

  clearPlayletPicker() {
    this.pendingTorrent = null;
  }

  triggerFlyingPip(fromX: number, fromY: number) {
    this.flyingPip = { id: crypto.randomUUID(), fromX, fromY };
  }

  clearFlyingPip() {
    this.flyingPip = null;
  }

  openSettings() {
    this.showSettings = true;
  }

  closeSettings() {
    this.showSettings = false;
  }

  toggleSettings() {
    this.showSettings = !this.showSettings;
  }
}

export const uiState = new UiState();
