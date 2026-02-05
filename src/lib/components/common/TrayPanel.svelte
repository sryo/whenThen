<!-- Tray panel: floating playlet list shown below the menu bar icon. -->
<script lang="ts">
  import type { Component } from "svelte";
  import { onMount, onDestroy } from "svelte";
  import { playletsState, derivePlayletName } from "$lib/state/playlets.svelte";
  import { settingsState } from "$lib/state/settings.svelte";
  import { uiState } from "$lib/state/ui.svelte";
  import { tasksState } from "$lib/state/tasks.svelte";
  import ActionBlock from "$lib/components/common/ActionBlock.svelte";
  import { buildTriggerIcons, triggerDetails, buildActionBlocks } from "$lib/utils/playlet-display";
  import { handleDroppedContent, handleDroppedFile } from "$lib/services/drag-drop";
  import { assignTorrentToPlaylet, beginManualDrop } from "$lib/services/playlet-assignment";
  import {
    setupTrayPanelListeners,
    cleanupTrayPanelListeners,
    setCallbacks,
    type TrayDropPayload,
  } from "$lib/services/tray-panel";
  import {
    setupEventListeners,
    cleanupEventListeners,
  } from "$lib/services/tauri-events";
  import {
    setupTriggerWatcher,
    cleanupTriggerWatcher,
  } from "$lib/services/trigger-watcher";
  import { settingsGet, torrentSyncRestored } from "$lib/services/tauri-commands";
  import { torrentsState } from "$lib/state/torrents.svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { emit } from "@tauri-apps/api/event";

  let visible = $state(false);
  let closing = $state(false);
  let draggingOverPlayletId = $state<string | null>(null);
  let successPlayletId = $state<string | null>(null);
  let pendingDrop = $state<TrayDropPayload | null>(null);

  const enabledPlaylets = $derived(playletsState.playlets.filter((p) => p.enabled));

  function doShow() {
    closing = false;
    visible = true;
  }

  function doHide() {
    if (!visible) return;
    closing = true;
    setTimeout(() => {
      visible = false;
      closing = false;
      pendingDrop = null;
      draggingOverPlayletId = null;
      successPlayletId = null;
      getCurrentWindow().hide();
    }, 150);
  }

  function handleTrayDrop(payload: TrayDropPayload) {
    pendingDrop = payload;
    doShow();
  }

  onMount(async () => {
    setCallbacks({
      onShow: doShow,
      onHide: doHide,
      onDrop: handleTrayDrop,
    });
    await setupTrayPanelListeners();
    await setupEventListeners();
    await setupTriggerWatcher();
    await playletsState.loadPlaylets();
    await tasksState.loadTasks();

    try {
      const config = await settingsGet();
      settingsState.setSettings(config);
    } catch {
      settingsState.applyScheme();
    }

    // Restore persisted torrents
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

    // Handle keyboard
    window.addEventListener("keydown", handleKeydown);
  });

  onDestroy(() => {
    cleanupTrayPanelListeners();
    cleanupEventListeners();
    cleanupTriggerWatcher();
    window.removeEventListener("keydown", handleKeydown);
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      doHide();
    }
  }

  function handleCardDragOver(e: DragEvent, playletId: string) {
    e.preventDefault();
    e.stopPropagation();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "copy";
    draggingOverPlayletId = playletId;
  }

  function handleCardDragLeave(e: DragEvent) {
    e.preventDefault();
    draggingOverPlayletId = null;
  }

  async function handleCardDrop(e: DragEvent, playletId: string) {
    e.preventDefault();
    e.stopPropagation();
    draggingOverPlayletId = null;

    const text = e.dataTransfer?.getData("text/plain");
    if (text) {
      await processContent(playletId, text);
      return;
    }

    const files = e.dataTransfer?.files;
    if (files) {
      for (const file of files) {
        try {
          beginManualDrop();
          let response;
          if ("path" in file && typeof (file as any).path === "string") {
            response = await handleDroppedContent((file as any).path);
          } else {
            response = await handleDroppedFile(file);
          }
          if (response) {
            const ok = assignTorrentToPlaylet(playletId, response, true);
            if (ok) showSuccess(playletId);
          }
        } catch (err: any) {
          uiState.addToast(`Could not add torrent: ${err?.message || String(err)}`, "error");
        }
      }
    }
  }

  async function handleCardClick(playletId: string) {
    if (!pendingDrop) return;
    const drop = pendingDrop;
    pendingDrop = null;

    for (const item of drop.data) {
      await processContent(playletId, item);
    }
  }

  async function processContent(playletId: string, content: string) {
    try {
      beginManualDrop();
      const response = await handleDroppedContent(content);
      if (response) {
        const ok = assignTorrentToPlaylet(playletId, response, true);
        if (ok) showSuccess(playletId);
      }
    } catch (err: any) {
      uiState.addToast(`Could not add torrent: ${err?.message || String(err)}`, "error");
    }
  }

  function showSuccess(playletId: string) {
    successPlayletId = playletId;
    setTimeout(() => {
      successPlayletId = null;
      doHide();
    }, 600);
  }

  function openMainWindow() {
    emit("tray:show-main");
  }
</script>

{#if visible || closing}
  <div class="tray-panel {closing ? 'panel-slide-up' : 'panel-slide-down'}">
    <!-- Header -->
    <div class="px-3 py-2 text-xs font-bold text-[var(--color-text-muted)]">
      whenThen
    </div>

    <!-- Playlet list -->
    <div class="flex-1 overflow-y-auto px-2 pb-2">
      {#if enabledPlaylets.length === 0}
        <div class="flex flex-col items-center gap-2 py-8 text-center">
          <p class="text-sm text-[var(--color-text-muted)]">No playlets.</p>
          <button
            onclick={openMainWindow}
            class="text-xs font-medium text-[var(--color-primary)] hover:underline"
          >
            Open whenThen to create one
          </button>
        </div>
      {:else}
        {#each enabledPlaylets as playlet (playlet.id)}
          {@const icons = buildTriggerIcons(playlet)}
          {@const details = triggerDetails(playlet)}
          {@const actions = buildActionBlocks(playlet)}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <button
            class="tray-card group mb-1 flex w-full items-center gap-2 rounded-lg px-2.5 py-2 text-left transition-all duration-100
              {draggingOverPlayletId === playlet.id ? 'ring-2 ring-[var(--color-primary)] scale-[1.02] bg-[var(--color-bg-tertiary)]' : 'hover:bg-[var(--color-bg-tertiary)]'}
              {successPlayletId === playlet.id ? 'bg-[var(--color-success)]/10 ring-2 ring-[var(--color-success)]' : ''}
              {pendingDrop ? 'cursor-pointer' : ''}"
            ondragover={(e) => handleCardDragOver(e, playlet.id)}
            ondragleave={handleCardDragLeave}
            ondrop={(e) => handleCardDrop(e, playlet.id)}
            onclick={() => handleCardClick(playlet.id)}
          >
            <!-- Trigger icons (compact) -->
            <div class="flex shrink-0 items-center gap-0.5">
              {#each icons as ti}
                {@const Icon = ti.icon as Component}
                <Icon style="color: {ti.color};" class="h-4 w-4" />
              {/each}
            </div>

            <!-- Name + details -->
            <div class="min-w-0 flex-1">
              <div class="truncate text-sm font-semibold text-[var(--color-text)]">
                {derivePlayletName(playlet)}
              </div>
              <div class="flex items-center gap-1 text-[10px]">
                {#each details as part, i}
                  {#if i > 0}
                    <span class="text-[var(--color-text-muted)]">&middot;</span>
                  {/if}
                  <span style="color: {part.color};">{part.text}</span>
                {/each}
              </div>
            </div>

            <!-- Action pips -->
            <div class="flex shrink-0 items-center gap-1">
              {#each actions as action}
                <ActionBlock icon={action.icon} color={action.color} label={action.label} size="sm" />
              {/each}
            </div>
          </button>
        {/each}
      {/if}
    </div>

    <!-- Footer hint -->
    <div class="border-t border-[var(--color-border)] px-3 py-1.5 text-center text-[10px] text-[var(--color-text-muted)]">
      {#if pendingDrop}
        Pick a playlet
      {:else}
        Drop content on a playlet
      {/if}
    </div>
  </div>
{/if}

<style>
  .tray-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    max-height: 100vh;
    background-color: var(--color-bg);
    border-radius: 10px;
    border: 1px solid var(--color-border);
    overflow: hidden;
  }

  .panel-slide-down {
    animation: slideDown 150ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  .panel-slide-up {
    animation: slideUp 150ms cubic-bezier(0.7, 0, 0.84, 0) forwards;
  }

  @keyframes slideDown {
    from { opacity: 0; transform: translateY(-8px) scale(0.96); }
    to   { opacity: 1; transform: translateY(0) scale(1); }
  }

  @keyframes slideUp {
    from { opacity: 1; transform: translateY(0) scale(1); }
    to   { opacity: 0; transform: translateY(-8px) scale(0.96); }
  }
</style>
