<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Settings } from "lucide-svelte";
  import PlayletsView from "$lib/components/views/PlayletsView.svelte";
  import Toast from "$lib/components/common/Toast.svelte";
  import TrayPanel from "$lib/components/common/TrayPanel.svelte";
  import SettingsSidebar from "$lib/components/common/SettingsSidebar.svelte";
  import { uiState } from "$lib/state/ui.svelte";
  import { settingsState } from "$lib/state/settings.svelte";
  import { torrentsState } from "$lib/state/torrents.svelte";
  import { playletsState } from "$lib/state/playlets.svelte";
  import { tasksState } from "$lib/state/tasks.svelte";
  import {
    setupEventListeners,
    cleanupEventListeners,
  } from "$lib/services/tauri-events";
  import {
    setupTriggerWatcher,
    cleanupTriggerWatcher,
  } from "$lib/services/trigger-watcher";
  import {
    checkOpenedViaUrl,
    chromecastStartDiscovery,
    settingsGet,
    torrentSyncRestored,
  } from "$lib/services/tauri-commands";
  import { findBestMatch, assignTorrentToPlaylet } from "$lib/services/playlet-assignment";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  const panelParam = new URLSearchParams(window.location.search).get("panel");
  const isTrayPanel = panelParam === "tray";
  const isEditorWindow = panelParam === "editor";
  const isPanel = isTrayPanel;

  // Panel windows need transparent backgrounds for rounded corners
  if (isPanel) {
    document.documentElement.classList.add("panel-window");
  }

  // Panel windows handle their own lifecycle; skip main window setup
  onMount(async () => {
    if (isTrayPanel) return; // TrayPanel runs its own init

    // Load state before registering event listeners so handlers see
    // the full playlet/task list (matters for cold-start via file association).
    await playletsState.loadPlaylets();
    await tasksState.loadTasks();

    try {
      const config = await settingsGet();
      settingsState.setSettings(config);
    } catch {
      // Use defaults, still apply scheme
      settingsState.applyScheme();
    }

    await setupEventListeners();
    await setupTriggerWatcher();

    // Restore persisted torrents (retry until session is ready)
    const hasPendingTasks = tasksState.activeTasks.length > 0;
    for (let attempt = 0; attempt < 20; attempt++) {
      try {
        const restored = await torrentSyncRestored();
        if (restored.length > 0) {
          torrentsState.setTorrents(restored);
          break;
        }
        // Session returned empty — retry if tasks expect torrents
        if (!hasPendingTasks) break;
        await new Promise((r) => setTimeout(r, 500));
      } catch {
        await new Promise((r) => setTimeout(r, 500));
      }
    }

    // Auto-assign unmatched restored torrents to best-matching playlet
    for (const torrent of torrentsState.torrents) {
      if (tasksState.getByTorrentId(torrent.id)) continue;
      const match = findBestMatch(
        torrent.name,
        "torrent_added",
        torrent.total_bytes || undefined,
        torrent.file_count || undefined,
      );
      if (match) {
        assignTorrentToPlaylet(match.id, {
          id: torrent.id,
          name: torrent.name,
          info_hash: torrent.info_hash,
          files: [],
        });
      }
    }

    // Editor window: stay hidden until explicitly shown
    if (isEditorWindow) {
      return;
    }

    // Main window: show only if not launched via file association
    const openedViaUrl = await checkOpenedViaUrl();
    if (!openedViaUrl) {
      // In headless mode, the main window stays hidden by default
      // It's only shown when user clicks "Open whenThen" from tray
    }

    if (settingsState.settings.auto_discover) {
      try {
        await chromecastStartDiscovery();
      } catch {}
    }

    // Suppress default WebView context menu
    window.addEventListener("contextmenu", suppressContextMenu);
  });

  onDestroy(() => {
    if (isTrayPanel) return;
    cleanupEventListeners();
    cleanupTriggerWatcher();
    window.removeEventListener("contextmenu", suppressContextMenu);
  });

  function suppressContextMenu(e: MouseEvent) {
    e.preventDefault();
  }
</script>

{#if isTrayPanel}
  <TrayPanel />
{:else if isEditorWindow}
  <!-- Editor window: subscriptions + playlets -->
  <div class="flex h-full flex-col bg-[var(--color-bg)]">
    <header class="flex shrink-0 items-center justify-between py-2 pl-20 pr-4" data-tauri-drag-region>
      <h1 class="pointer-events-none text-sm font-semibold text-[var(--color-text)]">whenThen</h1>
      <button
        onclick={() => uiState.toggleSettings()}
        class="rounded-lg p-1.5 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
        title="Settings"
      >
        <Settings class="h-4 w-4" />
      </button>
    </header>
    <main class="min-h-0 flex-1 overflow-y-auto">
      <PlayletsView />
    </main>
  </div>

  {#if uiState.showSettings}
    <SettingsSidebar />
  {/if}

  <!-- Toast notifications -->
  {#if uiState.toasts.length > 0}
    <div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2">
      {#each uiState.toasts as toast (toast.id)}
        <Toast {toast} />
      {/each}
    </div>
  {/if}
{:else}
  <!-- Main window: hidden by default, headless operation -->
  <div class="flex h-full flex-col bg-[var(--color-bg)]">
    <header class="flex shrink-0 items-center justify-between py-2 pl-20 pr-4" data-tauri-drag-region>
      <h1 class="pointer-events-none text-sm font-semibold text-[var(--color-text)]">whenThen</h1>
      <button
        onclick={() => uiState.toggleSettings()}
        class="rounded-lg p-1.5 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
        title="Settings"
      >
        <Settings class="h-4 w-4" />
      </button>
    </header>
    <main class="min-h-0 flex-1 overflow-y-auto">
      <PlayletsView />
    </main>
  </div>

  {#if uiState.showSettings}
    <SettingsSidebar />
  {/if}

  <!-- Toast notifications -->
  {#if uiState.toasts.length > 0}
    <div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2">
      {#each uiState.toasts as toast (toast.id)}
        <Toast {toast} />
      {/each}
    </div>
  {/if}
{/if}
