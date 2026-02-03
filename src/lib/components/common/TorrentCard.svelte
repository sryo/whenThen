<script lang="ts">
  import {
    Pause,
    Play,
    Trash2,
    GripVertical,
    ChevronDown,
    ChevronRight,
    Loader2,
  } from "lucide-svelte";
  import type { TorrentSummary } from "$lib/types/torrent";
  import { formatBytes, formatSpeed, formatProgress } from "$lib/utils";
  import {
    torrentPause,
    torrentResume,
    torrentDelete,
    torrentFiles,
  } from "$lib/services/tauri-commands";
  import { torrentsState } from "$lib/state/torrents.svelte";
  import { uiState } from "$lib/state/ui.svelte";
  import TorrentFileBrowser from "./TorrentFileBrowser.svelte";
  import type { TorrentFileInfo } from "$lib/types/torrent";

  let {
    torrent,
    index = 0,
    onDragStart,
    onDragOver,
    onDrop,
  }: {
    torrent: TorrentSummary;
    index?: number;
    onDragStart?: (e: DragEvent, i: number) => void;
    onDragOver?: (e: DragEvent, i: number) => void;
    onDrop?: (e: DragEvent, i: number) => void;
  } = $props();

  let expanded = $state(false);
  let files = $state<TorrentFileInfo[]>([]);
  let loadingFiles = $state(false);
  let actionLoading = $state(false);
  let deleting = $state(false);

  async function toggleExpanded() {
    expanded = !expanded;
    if (expanded && files.length === 0) {
      loadingFiles = true;
      try {
        files = await torrentFiles(torrent.id);
      } catch (err: any) {
        uiState.addToast(`Could not load files: ${err?.message || err}`, "error");
      } finally {
        loadingFiles = false;
      }
    }
  }

  async function handlePauseResume() {
    if (actionLoading) return;
    actionLoading = true;
    try {
      if (torrent.state === "downloading") {
        await torrentPause(torrent.id);
      } else if (torrent.state === "paused") {
        await torrentResume(torrent.id);
      }
    } catch (err: any) {
      uiState.addToast(`Could not update torrent: ${err?.message || err}`, "error");
    } finally {
      actionLoading = false;
    }
  }

  async function handleDelete() {
    if (deleting) return;
    deleting = true;
    try {
      await torrentDelete(torrent.id, false);
      torrentsState.removeTorrent(torrent.id);
    } catch (err: any) {
      uiState.addToast(`Could not update torrent: ${err?.message || err}`, "error");
      deleting = false;
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] transition-colors hover:border-[var(--color-text-muted)] {deleting ? 'opacity-50 pointer-events-none' : ''}"
  draggable="true"
  ondragstart={(e) => onDragStart?.(e, index)}
  ondragover={(e) => onDragOver?.(e, index)}
  ondrop={(e) => onDrop?.(e, index)}
>
  <div class="flex items-center gap-3 px-3 py-2">
    <div class="cursor-grab text-[var(--color-text-muted)] active:cursor-grabbing">
      <GripVertical class="h-4 w-4" />
    </div>
    <button
      onclick={toggleExpanded}
      class="shrink-0 text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
    >
      {#if expanded}
        <ChevronDown class="h-4 w-4" />
      {:else}
        <ChevronRight class="h-4 w-4" />
      {/if}
    </button>
    <div class="min-w-0 flex-1">
      <div class="flex items-center gap-2">
        <h3 class="truncate text-sm font-medium text-[var(--color-text)]">
          {torrent.name}
        </h3>
      </div>
      <div class="mt-1 flex items-center gap-2">
        <!-- Progress bar -->
        <div class="h-1.5 flex-1 overflow-hidden rounded-full bg-[var(--color-bg-tertiary)]">
          <div
            class="h-full rounded-full transition-all {torrent.state === 'completed'
              ? 'bg-[var(--color-success)]'
              : 'bg-[var(--color-primary)]'}"
            style="width: {torrent.progress * 100}%"
          ></div>
        </div>
        <span class="shrink-0 text-xs text-[var(--color-text-muted)]">
          {formatProgress(torrent.progress)}
        </span>
      </div>
      <p class="mt-0.5 text-xs text-[var(--color-text-muted)]">
        {#if torrent.state === "downloading"}
          {formatSpeed(torrent.download_speed)} &middot; {torrent.peers_connected} peers
        {:else if torrent.state === "completed"}
          {formatBytes(torrent.total_bytes)} &middot; Completed
        {:else if torrent.state === "paused"}
          Paused
        {:else}
          {torrent.state}
        {/if}
      </p>
    </div>

    <div class="flex items-center gap-1 shrink-0">
      {#if actionLoading}
        <span class="animate-spin rounded p-1.5 text-[var(--color-text-muted)]">
          <Loader2 class="h-4 w-4" />
        </span>
      {:else if torrent.state === "downloading"}
        <button
          onclick={handlePauseResume}
          class="rounded p-1.5 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
          title="Pause"
        >
          <Pause class="h-4 w-4" />
        </button>
      {:else if torrent.state === "paused"}
        <button
          onclick={handlePauseResume}
          class="rounded p-1.5 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
          title="Resume"
        >
          <Play class="h-4 w-4" />
        </button>
      {/if}
      <button
        onclick={handleDelete}
        disabled={deleting}
        class="rounded p-1.5 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-error)] disabled:opacity-50"
        title="Delete"
      >
        {#if deleting}
          <Loader2 class="h-4 w-4 animate-spin" />
        {:else}
          <Trash2 class="h-4 w-4" />
        {/if}
      </button>
    </div>
  </div>

  <!-- Expandable file browser -->
  {#if expanded}
    <div class="border-t border-[var(--color-border)] px-3 py-2">
      {#if loadingFiles}
        <p class="text-center text-xs text-[var(--color-text-muted)]">Loading files...</p>
      {:else}
        <TorrentFileBrowser {files} torrentId={torrent.id} />
      {/if}
    </div>
  {/if}
</div>
