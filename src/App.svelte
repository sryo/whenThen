<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import AppShell from "$lib/components/layout/AppShell.svelte";
  import { settingsState } from "$lib/state/settings.svelte";
  import { i18n } from "$lib/i18n/state.svelte";
  import { torrentsState } from "$lib/state/torrents.svelte";
  import { playletsState } from "$lib/state/playlets.svelte";
  import { tasksState } from "$lib/state/tasks.svelte";
  import { feedsState } from "$lib/state/feeds.svelte";
  import { uiState } from "$lib/state/ui.svelte";
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
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import type { ViewName } from "$lib/types/ui";

  let unlistenPending: UnlistenFn | null = null;
  let unlistenNavigate: UnlistenFn | null = null;
  let unlistenMagnet: UnlistenFn | null = null;

  onMount(async () => {
    await playletsState.loadPlaylets();
    await tasksState.loadTasks();
    await feedsState.loadFeeds();
    await feedsState.loadPending();
    await uiState.loadPersistedState();

    try {
      const config = await settingsGet();
      settingsState.setSettings(config);
      await i18n.setLocale(settingsState.settings.locale);
    } catch {
      // Use defaults, still apply scheme
      settingsState.applyScheme();
      await i18n.setLocale(settingsState.settings.locale);
    }

    await setupEventListeners();
    await setupTriggerWatcher();

    const hasPendingTasks = tasksState.activeTasks.length > 0;
    for (let attempt = 0; attempt < 20; attempt++) {
      try {
        const restored = await torrentSyncRestored();
        if (restored.length > 0) {
          torrentsState.setTorrents(restored);
          break;
        }
        if (!hasPendingTasks) break;
        await new Promise((r) => setTimeout(r, 500));
      } catch {
        await new Promise((r) => setTimeout(r, 500));
      }
    }

    const validTorrentIds = new Set(torrentsState.torrents.map((t) => t.id));
    tasksState.reconcileWithTorrents(validTorrentIds);

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

    await checkOpenedViaUrl();

    if (settingsState.settings.auto_discover) {
      try {
        await chromecastStartDiscovery();
      } catch {}
    }

    unlistenPending = await listen<number>("rss:pending-count", (event) => {
      feedsState.updatePendingCount(event.payload);
    });

    unlistenNavigate = await listen<string>("menu:navigate", (event) => {
      const view = event.payload as ViewName;
      uiState.setView(view);
    });

    unlistenMagnet = await listen("menu:add-magnet", () => {
      const magnet = window.prompt("Enter magnet link:");
      if (magnet && magnet.startsWith("magnet:")) {
        import("$lib/services/tauri-commands").then(({ torrentAddMagnet }) => {
          torrentAddMagnet(magnet).catch((err) => {
            uiState.addToast(`Failed to add magnet: ${err?.message || err}`, "error");
          });
        });
      } else if (magnet) {
        uiState.addToast("Invalid magnet link format", "error");
      }
    });

    window.addEventListener("contextmenu", suppressContextMenu);
  });

  onDestroy(() => {
    cleanupEventListeners();
    cleanupTriggerWatcher();
    unlistenPending?.();
    unlistenNavigate?.();
    unlistenMagnet?.();
    window.removeEventListener("contextmenu", suppressContextMenu);
  });

  function suppressContextMenu(e: MouseEvent) {
    e.preventDefault();
  }
</script>

<AppShell />
