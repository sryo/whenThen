<!-- Playlet picker modal shown when a torrent is added and multiple playlets match. -->
<script lang="ts">
  import type { Component } from "svelte";
  import { onMount, onDestroy } from "svelte";
  import ActionBlock from "$lib/components/common/ActionBlock.svelte";
  import { flowConnector } from "$lib/utils/flow-connector";
  import { buildTriggerIcons, triggerDetails, buildActionBlocks } from "$lib/utils/playlet-display";
  import { settingsState } from "$lib/state/settings.svelte";
  import { playletsState, derivePlayletName } from "$lib/state/playlets.svelte";
  import { uiState } from "$lib/state/ui.svelte";
  import { torrentsState } from "$lib/state/torrents.svelte";
  import { tasksState } from "$lib/state/tasks.svelte";
  import { assignTorrentToPlaylet } from "$lib/services/playlet-assignment";
  import { torrentDelete } from "$lib/services/tauri-commands";

  const ANIM_MS = 150;

  const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;
  const modLabel = isMac ? "\u2318" : "Ctrl+";

  let closing = $state(false);

  // Countdown state
  const countdownTotal = $derived(settingsState.settings.picker_countdown_seconds);
  let remaining = $state(countdownTotal);
  let timerHandle: ReturnType<typeof setInterval> | null = null;

  const eligible = $derived(
    playletsState.playlets.filter((p) => p.enabled && p.trigger.type === "torrent_added"),
  );

  function selectPlaylet(playletId: string) {
    const pending = uiState.pendingTorrent;
    if (!pending) return;

    stopTimer();

    const alreadyExists = torrentsState.torrents.some((t) => t.id === pending.id);

    if (alreadyExists) {
      const existingTask = tasksState.getByTorrentId(pending.id);
      if (existingTask) {
        tasksState.reassignTask(existingTask.id, playletId);
        const playlet = playletsState.getById(playletId);
        const name = playlet ? derivePlayletName(playlet) : "playlet";
        uiState.addToast(`Reassigned to "${name}"`, "success");
      } else {
        assignTorrentToPlaylet(playletId, pending);
      }
    } else {
      torrentsState.addTorrent({
        id: pending.id,
        name: pending.name,
        info_hash: pending.info_hash,
        state: "initializing",
        progress: 0,
        download_speed: 0,
        upload_speed: 0,
        peers_connected: 0,
        total_bytes: 0,
        downloaded_bytes: 0,
        uploaded_bytes: 0,
        file_count: pending.files.length,
      });
      assignTorrentToPlaylet(playletId, pending);
    }

    beginClose();
  }

  function dismiss() {
    const pending = uiState.pendingTorrent;
    stopTimer();

    if (pending) {
      const alreadyExists = torrentsState.torrents.some((t) => t.id === pending.id);
      if (!alreadyExists) {
        torrentDelete(pending.id, false).catch(() => {});
      }
    }

    beginClose();
  }

  function beginClose() {
    closing = true;
    setTimeout(() => {
      uiState.clearPlayletPicker();
    }, ANIM_MS);
  }

  function startTimer() {
    if (countdownTotal <= 0) return;
    remaining = countdownTotal;
    timerHandle = setInterval(() => {
      remaining -= 0.1;
      if (remaining <= 0) {
        remaining = 0;
        stopTimer();
        if (eligible.length > 0) {
          selectPlaylet(eligible[0].id);
        } else {
          dismiss();
        }
      }
    }, 100);
  }

  function stopTimer() {
    if (timerHandle !== null) {
      clearInterval(timerHandle);
      timerHandle = null;
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      dismiss();
      return;
    }

    const mod = isMac ? e.metaKey : e.ctrlKey;
    if (!mod) return;

    const num = parseInt(e.key);
    if (num >= 1 && num <= 9) {
      e.preventDefault();
      const playlet = eligible[num - 1];
      if (playlet) {
        selectPlaylet(playlet.id);
      }
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      dismiss();
    }
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeyDown, true);
    startTimer();
  });

  onDestroy(() => {
    window.removeEventListener("keydown", handleKeyDown, true);
    stopTimer();
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  role="dialog"
  aria-modal="true"
  aria-label="Pick a playlet"
  tabindex="-1"
  class="fixed inset-0 z-50 flex items-center justify-center backdrop-blur-sm
    {closing ? 'backdrop-fade-out' : 'backdrop-fade-in'}"
  onclick={handleBackdropClick}
>
  <div
    class="mx-4 flex w-full max-w-lg flex-col overflow-hidden rounded-2xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] shadow-2xl
      {closing ? 'dialog-scale-out' : 'dialog-scale-in'}"
  >
    <!-- Countdown bar -->
    {#if countdownTotal > 0}
      <div class="h-1 w-full bg-[var(--color-bg-tertiary)]">
        <div
          class="h-full bg-[var(--color-primary)] transition-[width] duration-100 ease-linear"
          style="width: {(remaining / countdownTotal) * 100}%;"
        ></div>
      </div>
    {/if}

    <!-- Header -->
    <div class="px-6 pt-5 pb-3">
      {#if uiState.pendingTorrent}
        <p class="mb-1 text-xs font-medium text-[var(--color-text-muted)]">Pick a playlet</p>
        <h2 class="truncate text-sm font-bold text-[var(--color-text)]">
          {uiState.pendingTorrent.name}
        </h2>
      {/if}
    </div>

    <!-- Playlet list -->
    <div class="max-h-80 min-h-0 flex-1 space-y-1.5 overflow-y-auto px-4 pb-4">
      {#each eligible as playlet, i (playlet.id)}
        {@const triggerIcons = buildTriggerIcons(playlet)}
        {@const details = triggerDetails(playlet)}
        {@const actions = buildActionBlocks(playlet)}
        <button
          class="flex w-full flex-col rounded-xl bg-[var(--color-bg)] px-4 py-3 text-left transition-colors hover:bg-[var(--color-bg-tertiary)]"
          onclick={() => selectPlaylet(playlet.id)}
        >
          <div use:flowConnector class="relative mb-2 flex flex-wrap items-end gap-x-2 gap-y-3">
            <div data-flow-block class="flex flex-col items-center gap-1 rounded-lg border border-[var(--color-primary)]/30 bg-[var(--color-primary)]/10 px-2 py-1.5">
              <div class="flex items-center gap-1">
                {#each triggerIcons as ti}
                  {@const Icon = ti.icon as Component}
                  <Icon style="color: {ti.color};" class="h-4 w-4 shrink-0" />
                {/each}
              </div>
              <div class="flex items-center gap-1 text-[10px] font-bold">
                {#each details as part, j}
                  {#if j > 0}
                    <span class="text-[var(--color-text-muted)]">&middot;</span>
                  {/if}
                  <span style="color: {part.color};">{part.text}</span>
                {/each}
              </div>
            </div>
            {#each actions as act}
              <div data-flow-block>
                <ActionBlock icon={act.icon} color={act.color} label={act.label} size="sm" />
              </div>
            {/each}
          </div>
          <div class="flex items-center justify-between gap-2">
            <span class="truncate text-sm font-bold text-[var(--color-text)]">{derivePlayletName(playlet)}</span>
            {#if i < 9}
              <span class="shrink-0 rounded-md bg-[var(--color-bg-tertiary)] px-2 py-1 text-xs font-mono font-medium text-[var(--color-text-muted)]">{modLabel}{i + 1}</span>
            {/if}
          </div>
        </button>
      {/each}
    </div>

    <!-- Dismiss hint -->
    <div class="border-t border-[var(--color-border)] px-6 py-3">
      <p class="text-center text-xs text-[var(--color-text-muted)]">
        Press <kbd class="rounded bg-[var(--color-bg-tertiary)] px-1 py-0.5 font-mono text-[10px]">Esc</kbd> to dismiss
      </p>
    </div>
  </div>
</div>

<style>
  .backdrop-fade-in {
    animation: fadeIn 150ms ease-out forwards;
  }
  .backdrop-fade-out {
    animation: fadeOut 150ms ease-in forwards;
  }
  .dialog-scale-in {
    animation: scaleIn 150ms ease-out forwards;
  }
  .dialog-scale-out {
    animation: scaleOut 150ms ease-in forwards;
  }

  @keyframes fadeIn {
    from { background-color: transparent; }
    to   { background-color: rgb(0 0 0 / 0.3); }
  }
  @keyframes fadeOut {
    from { background-color: rgb(0 0 0 / 0.3); }
    to   { background-color: transparent; }
  }
  @keyframes scaleIn {
    from { opacity: 0; transform: scale(0.95); }
    to   { opacity: 1; transform: scale(1); }
  }
  @keyframes scaleOut {
    from { opacity: 1; transform: scale(1); }
    to   { opacity: 0; transform: scale(0.95); }
  }
</style>
