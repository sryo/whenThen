<script lang="ts">
  import { onMount, onDestroy, tick } from "svelte";
  import AppShell from "$lib/components/layout/AppShell.svelte";
  import PlayletsView from "$lib/components/views/PlayletsView.svelte";
  import ActivityView from "$lib/components/views/ActivityView.svelte";
  import SettingsView from "$lib/components/views/SettingsView.svelte";
  import Toast from "$lib/components/common/Toast.svelte";
  import PlayletPickerModal from "$lib/components/common/PlayletPickerModal.svelte";
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
    chromecastStartDiscovery,
    settingsGet,
    torrentSyncRestored,
  } from "$lib/services/tauri-commands";
  import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";

  let contentEl: HTMLElement | undefined = $state();
  let resizeObserver: ResizeObserver | undefined;

  // Trigger the .go class on the next frame after pip mounts
  $effect(() => {
    if (uiState.flyingPip) {
      tick().then(() => {
        requestAnimationFrame(() => {
          document.querySelectorAll('.flying-pip-x, .flying-pip-y').forEach((el) => el.classList.add('go'));
        });
      });
    }
  });

  onMount(async () => {
    await setupEventListeners();
    await setupTriggerWatcher();

    await playletsState.loadPlaylets();
    await tasksState.loadTasks();

    try {
      const config = await settingsGet();
      settingsState.setSettings(config);
    } catch {
      // Use defaults, still apply scheme
      settingsState.applyScheme();
    }

    // Restore persisted torrents (retry until session is ready)
    for (let attempt = 0; attempt < 20; attempt++) {
      try {
        const restored = await torrentSyncRestored();
        if (restored.length > 0) {
          torrentsState.setTorrents(restored);
        }
        break;
      } catch {
        await new Promise((r) => setTimeout(r, 500));
      }
    }

    if (settingsState.settings.auto_discover) {
      try {
        await chromecastStartDiscovery();
      } catch {}

    }

    // Suppress default WebView context menu (component menus use stopPropagation)
    window.addEventListener("contextmenu", suppressContextMenu);

    window.addEventListener("keydown", handleKeyboard);

    // Auto-resize window to fit content (capped to screen height)
    if (contentEl) {
      resizeObserver = new ResizeObserver(() => {
        if (!contentEl) return;
        const height = Math.min(contentEl.scrollHeight, window.screen.availHeight);
        const width = contentEl.offsetWidth;
        getCurrentWindow().setSize(new LogicalSize(width, height));
      });
      resizeObserver.observe(contentEl);
    }
  });

  onDestroy(() => {
    cleanupEventListeners();
    cleanupTriggerWatcher();
    window.removeEventListener("contextmenu", suppressContextMenu);
    window.removeEventListener("keydown", handleKeyboard);
    resizeObserver?.disconnect();
  });

  function suppressContextMenu(e: MouseEvent) {
    e.preventDefault();
  }

  function handleKeyboard(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    if (
      target.tagName === "INPUT" ||
      target.tagName === "TEXTAREA" ||
      target.tagName === "SELECT"
    ) {
      return;
    }

    switch (e.key) {
      case "1":
        uiState.setView("playlets");
        break;
      case "2":
        uiState.setView("activity");
        break;
      case "3":
        uiState.setView("settings");
        break;
    }
  }
</script>

<div bind:this={contentEl} class="h-full">
  <AppShell>
    {#if uiState.activeView === "playlets"}
      <PlayletsView />
    {:else if uiState.activeView === "activity"}
      <ActivityView />
    {:else if uiState.activeView === "settings"}
      <SettingsView />
    {/if}
  </AppShell>
</div>

<!-- Playlet picker modal -->
{#if uiState.pendingTorrent}
  <PlayletPickerModal />
{/if}

<!-- Toast notifications -->
{#if uiState.toasts.length > 0}
  <div class="fixed bottom-10 right-4 z-50 flex flex-col gap-2">
    {#each uiState.toasts as toast (toast.id)}
      <Toast {toast} />
    {/each}
  </div>
{/if}

<!-- Flying pip: animated dot from card to Activity badge -->
{#if uiState.flyingPip}
  {@const pip = uiState.flyingPip}
  {@const dest = document.querySelector('[data-nav="activity"]')?.getBoundingClientRect()}
  {#if dest}
    {@const toX = dest.left + dest.width / 2}
    {@const toY = dest.top + dest.height / 2}
    <div
      class="flying-pip-x"
      style="left:{pip.fromX}px; top:0px; --dx:{toX - pip.fromX}px"
      ontransitionend={() => uiState.clearFlyingPip()}
    >
      <div
        class="flying-pip-y"
        style="top:{pip.fromY}px; --dy:{toY - pip.fromY}px"
      >
        <div class="flying-pip-dot"></div>
      </div>
    </div>
  {/if}
{/if}

<style>
  /* Outer div: horizontal movement with ease-out */
  .flying-pip-x {
    position: fixed;
    z-index: 9999;
    pointer-events: none;
    transform: translateX(0);
    transition: transform 400ms ease-out;
  }
  /* .go is added via JS after mount */
  .flying-pip-x:global(.go) {
    transform: translateX(var(--dx));
  }

  /* Inner div: vertical movement with ease-in */
  .flying-pip-y {
    position: absolute;
    left: 0;
    transform: translateY(0);
    transition: transform 400ms ease-in;
  }
  .flying-pip-y:global(.go) {
    transform: translateY(var(--dy));
  }

  .flying-pip-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--color-primary);
    margin-left: -5px;
    margin-top: -5px;
    transition: transform 400ms ease-in, opacity 400ms ease-in;
  }
  .flying-pip-x:global(.go) .flying-pip-dot {
    transform: scale(0.4);
    opacity: 0.5;
  }
</style>
