<!-- Tray panel: screener inbox + active downloads. -->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Pause, Play, X, Settings, ThumbsUp, ThumbsDown, AlertTriangle, Film, FileText, Loader2, ChevronDown, ChevronUp, RefreshCw, Trash2 } from "lucide-svelte";
  import ContextMenu from "$lib/components/common/ContextMenu.svelte";
  import { useContextMenu } from "$lib/utils";
  import type { ContextMenuEntry } from "$lib/types/ui";
  import { torrentsState } from "$lib/state/torrents.svelte";
  import { settingsState } from "$lib/state/settings.svelte";
  import { feedsState, type PendingMatch, type TorrentMetadata } from "$lib/state/feeds.svelte";
  import { playletsState } from "$lib/state/playlets.svelte";
  import { tasksState } from "$lib/state/tasks.svelte";
  import {
    setupTrayPanelListeners,
    cleanupTrayPanelListeners,
    setCallbacks,
  } from "$lib/services/tray-panel";
  import {
    setupEventListeners,
    cleanupEventListeners,
  } from "$lib/services/tauri-events";
  import {
    setupTriggerWatcher,
    cleanupTriggerWatcher,
  } from "$lib/services/trigger-watcher";
  import { settingsGet, torrentSyncRestored, torrentPause, torrentResume, torrentDelete, torrentRecheck } from "$lib/services/tauri-commands";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { emit, listen } from "@tauri-apps/api/event";

  let visible = $state(false);
  let closing = $state(false);
  let activeTab = $state<"screener" | "downloads">("downloads");
  let expandedMatchId = $state<string | null>(null);
  let loadingMetadata = $state<string | null>(null);

  const activeDownloads = $derived(torrentsState.activeTorrents);
  const pendingMatches = $derived(feedsState.pendingMatches);

  const torrentCtx = useContextMenu<{ id: number; state: string }>();

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
      getCurrentWindow().hide();
    }, 150);
  }

  onMount(async () => {
    setCallbacks({
      onShow: doShow,
      onHide: doHide,
      onDrop: () => {},
    });
    await setupTrayPanelListeners();
    await setupEventListeners();
    await setupTriggerWatcher();
    await playletsState.loadPlaylets();
    await tasksState.loadTasks();
    await feedsState.loadFeeds();
    await feedsState.loadPending();

    try {
      const config = await settingsGet();
      settingsState.setSettings(config);
    } catch {
      settingsState.applyScheme();
    }

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

    // Fail any orphaned tasks whose torrents no longer exist
    const validTorrentIds = new Set(torrentsState.torrents.map((t) => t.id));
    tasksState.reconcileWithTorrents(validTorrentIds);

    // Listen for pending count updates
    const unlisten = await listen<number>("rss:pending-count", (event) => {
      feedsState.updatePendingCount(event.payload);
    });

    window.addEventListener("keydown", handleKeydown);

    return () => {
      unlisten();
    };
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

  function formatSpeed(bytesPerSecond: number): string {
    if (bytesPerSecond < 1024) return `${bytesPerSecond} B/s`;
    if (bytesPerSecond < 1024 * 1024) return `${(bytesPerSecond / 1024).toFixed(1)} KB/s`;
    return `${(bytesPerSecond / (1024 * 1024)).toFixed(1)} MB/s`;
  }

  function formatPercent(progress: number): string {
    return `${Math.round(progress * 100)}%`;
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(0)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  }

  async function handlePauseResume(id: number, state: string) {
    try {
      if (state === "paused") {
        await torrentResume(id);
      } else {
        await torrentPause(id);
      }
    } catch {}
  }

  async function handleRemove(id: number) {
    try {
      await torrentDelete(id, false);
      torrentsState.removeTorrent(id);
      tasksState.failTasksForTorrent(id);
    } catch (e) {
      console.error("Failed to remove torrent:", e);
    }
  }

  async function handleRecheck(id: number) {
    try {
      await torrentRecheck(id);
    } catch (e) {
      console.error("Failed to recheck torrent:", e);
    }
  }

  async function handleDeleteWithFiles(id: number) {
    try {
      await torrentDelete(id, true);
      torrentsState.removeTorrent(id);
      tasksState.failTasksForTorrent(id);
    } catch (e) {
      console.error("Failed to delete torrent with files:", e);
    }
  }

  function torrentContextMenuItems(id: number, state: string): ContextMenuEntry[] {
    return [
      {
        icon: state === "paused" ? Play : Pause,
        label: state === "paused" ? "Resume" : "Pause",
        action: () => handlePauseResume(id, state),
      },
      {
        icon: RefreshCw,
        label: "Recheck",
        action: () => handleRecheck(id),
      },
      { type: "divider" },
      {
        icon: X,
        label: "Remove",
        action: () => handleRemove(id),
      },
      {
        icon: Trash2,
        label: "Remove with files",
        danger: true,
        action: () => handleDeleteWithFiles(id),
      },
    ];
  }

  async function toggleExpand(match: PendingMatch) {
    if (expandedMatchId === match.id) {
      expandedMatchId = null;
      return;
    }

    expandedMatchId = match.id;

    // Fetch metadata if not already loaded
    if (!match.metadata) {
      loadingMetadata = match.id;
      try {
        await feedsState.fetchMetadata(match.id);
      } catch (e) {
        console.error("Failed to fetch metadata:", e);
      } finally {
        loadingMetadata = null;
      }
    }
  }

  async function approveMatch(e: Event, matchId: string) {
    e.stopPropagation();
    try {
      await feedsState.approveMatch(matchId);
      if (expandedMatchId === matchId) expandedMatchId = null;
    } catch (e) {
      console.error("Failed to approve:", e);
    }
  }

  async function rejectMatch(e: Event, matchId: string) {
    e.stopPropagation();
    try {
      await feedsState.rejectMatch(matchId);
      if (expandedMatchId === matchId) expandedMatchId = null;
    } catch (e) {
      console.error("Failed to reject:", e);
    }
  }

  function openEditor() {
    emit("tray:show-main");
  }

  function quitApp() {
    emit("tray:quit");
  }
</script>

{#if visible || closing}
  <div class="tray-panel {closing ? 'panel-slide-up' : 'panel-slide-down'}">
    <!-- Header with tabs -->
    <div class="flex items-center justify-between border-b border-[var(--color-border)] px-3 py-2">
      <div class="relative flex items-center gap-1 rounded-lg bg-[var(--color-bg-secondary)] p-0.5">
        <!-- Sliding indicator -->
        <div
          class="absolute h-[calc(100%-4px)] rounded-md bg-[var(--color-primary)] tab-indicator"
          style="width: calc(50% - 2px); left: {activeTab === 'screener' ? '2px' : 'calc(50% + 0px)'};"
        ></div>
        <button
          onclick={() => activeTab = "screener"}
          class="relative z-10 rounded-md px-2 py-1 text-xs font-medium transition-colors {activeTab === 'screener' ? 'text-white' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'}"
        >
          Screener
          {#if pendingMatches.length > 0}
            <span class="ml-1 rounded-full {activeTab === 'screener' ? 'bg-white/20' : 'bg-[var(--color-error)]'} px-1.5 text-[10px] text-white">{pendingMatches.length}</span>
          {/if}
        </button>
        <button
          onclick={() => activeTab = "downloads"}
          class="relative z-10 rounded-md px-2 py-1 text-xs font-medium transition-colors {activeTab === 'downloads' ? 'text-white' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'}"
        >
          Downloads
          {#if activeDownloads.length > 0}
            <span class="ml-1 text-[10px] {activeTab === 'downloads' ? 'text-white/60' : 'opacity-60'}">{activeDownloads.length}</span>
          {/if}
        </button>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto">
      {#if activeTab === "screener"}
        <!-- Screener inbox -->
        {#if pendingMatches.length === 0}
          <div class="flex flex-col items-center justify-center py-12 text-center">
            <p class="text-sm text-[var(--color-text-muted)]">No items to screen</p>
            <p class="mt-1 text-xs text-[var(--color-text-muted)]">RSS matches will appear here</p>
          </div>
        {:else}
          <div class="divide-y divide-[var(--color-border)]">
            {#each pendingMatches as match (match.id)}
              {@const isExpanded = expandedMatchId === match.id}
              {@const isLoading = loadingMetadata === match.id}
              <div class="group">
                <!-- Main row: Yes/No + Content -->
                <div class="flex items-stretch">
                  <!-- Yes/No buttons (HEY style) -->
                  <div class="flex shrink-0 flex-col border-r border-[var(--color-border)]">
                    <button
                      onclick={(e) => approveMatch(e, match.id)}
                      class="flex flex-1 items-center justify-center px-3 text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-success)]/10 hover:text-[var(--color-success)]"
                      title="Download"
                    >
                      <ThumbsUp class="h-4 w-4" />
                    </button>
                    <div class="border-t border-[var(--color-border)]"></div>
                    <button
                      onclick={(e) => rejectMatch(e, match.id)}
                      class="flex flex-1 items-center justify-center px-3 text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-error)]/10 hover:text-[var(--color-error)]"
                      title="Skip"
                    >
                      <ThumbsDown class="h-4 w-4" />
                    </button>
                  </div>

                  <!-- Content -->
                  <button
                    onclick={() => toggleExpand(match)}
                    class="flex min-w-0 flex-1 flex-col gap-0.5 px-3 py-2 text-left transition-colors hover:bg-[var(--color-bg-secondary)]"
                  >
                    <div class="flex items-center gap-2">
                      <span class="rounded bg-[var(--color-primary)]/10 px-1.5 py-0.5 text-[10px] font-medium text-[var(--color-primary)]">
                        {match.interestName}
                      </span>
                      <span class="text-[10px] text-[var(--color-text-muted)]">
                        via {match.sourceName}
                      </span>
                      {#if isExpanded}
                        <ChevronUp class="ml-auto h-3 w-3 text-[var(--color-text-muted)]" />
                      {:else}
                        <ChevronDown class="ml-auto h-3 w-3 text-[var(--color-text-muted)]" />
                      {/if}
                    </div>
                    <span class="line-clamp-2 text-sm font-medium text-[var(--color-text)]">
                      {match.title}
                    </span>
                  </button>
                </div>

                <!-- Expanded: file list preview -->
                {#if isExpanded}
                  <div class="border-t border-[var(--color-border)] bg-[var(--color-bg-secondary)] px-3 py-2">
                    {#if isLoading}
                      <div class="flex items-center justify-center gap-2 py-4">
                        <Loader2 class="h-4 w-4 animate-spin text-[var(--color-text-muted)]" />
                        <span class="text-xs text-[var(--color-text-muted)]">Loading contents...</span>
                      </div>
                    {:else if match.metadata}
                      {@const meta = match.metadata}
                      {@const hasSuspicious = meta.files.some((f) => f.isSuspicious)}
                      {@const videoCount = meta.files.filter((f) => f.isVideo).length}

                      <!-- Summary -->
                      <div class="mb-2 flex items-center gap-3 text-xs">
                        <span class="text-[var(--color-text-muted)]">{meta.fileCount} files</span>
                        <span class="text-[var(--color-text-muted)]">{formatSize(meta.totalSize)}</span>
                        {#if videoCount > 0}
                          <span class="flex items-center gap-1 text-[var(--color-success)]">
                            <Film class="h-3 w-3" />
                            {videoCount} video{videoCount !== 1 ? "s" : ""}
                          </span>
                        {/if}
                        {#if hasSuspicious}
                          <span class="flex items-center gap-1 text-[var(--color-error)]">
                            <AlertTriangle class="h-3 w-3" />
                            Suspicious files
                          </span>
                        {/if}
                      </div>

                      <!-- File list -->
                      <div class="max-h-32 space-y-0.5 overflow-y-auto">
                        {#each meta.files.slice(0, 15) as file}
                          <div class="flex items-center gap-2 text-[10px]">
                            {#if file.isSuspicious}
                              <AlertTriangle class="h-3 w-3 shrink-0 text-[var(--color-error)]" />
                            {:else if file.isVideo}
                              <Film class="h-3 w-3 shrink-0 text-[var(--color-success)]" />
                            {:else}
                              <FileText class="h-3 w-3 shrink-0 text-[var(--color-text-muted)]" />
                            {/if}
                            <span class="min-w-0 flex-1 truncate {file.isSuspicious ? 'text-[var(--color-error)]' : 'text-[var(--color-text)]'}">
                              {file.name}
                            </span>
                            <span class="shrink-0 text-[var(--color-text-muted)]">{formatSize(file.size)}</span>
                          </div>
                        {/each}
                        {#if meta.files.length > 15}
                          <div class="text-[10px] text-[var(--color-text-muted)]">
                            +{meta.files.length - 15} more files
                          </div>
                        {/if}
                      </div>
                    {:else}
                      <div class="py-2 text-center text-xs text-[var(--color-text-muted)]">
                        Couldn't load contents
                      </div>
                    {/if}
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        {/if}

      {:else}
        <!-- Downloads list -->
        <div class="px-2 py-2">
          {#if activeDownloads.length === 0}
            <div class="flex flex-col items-center justify-center py-12 text-center">
              <p class="text-sm text-[var(--color-text-muted)]">No active downloads</p>
            </div>
          {:else}
            {#each activeDownloads as torrent (torrent.id)}
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="mb-2 rounded-lg bg-[var(--color-bg-secondary)] p-2.5"
                oncontextmenu={(e) => torrentCtx.open(e, { id: torrent.id, state: torrent.state })}
              >
                <div class="mb-1.5 truncate text-sm font-medium text-[var(--color-text)]">
                  {torrent.name}
                </div>
                <div class="mb-1.5 h-1.5 overflow-hidden rounded-full bg-[var(--color-bg-tertiary)]">
                  <div
                    class="h-full rounded-full transition-all duration-300 {torrent.state === 'paused' ? 'bg-[var(--color-warning)]' : 'bg-[var(--color-primary)]'}"
                    style="width: {torrent.progress * 100}%"
                  ></div>
                </div>
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2 text-[10px] text-[var(--color-text-muted)]">
                    <span class="font-medium">{formatPercent(torrent.progress)}</span>
                    {#if torrent.state !== "paused"}
                      <span>↓ {formatSpeed(torrent.download_speed)}</span>
                    {:else}
                      <span class="text-[var(--color-warning)]">Paused</span>
                    {/if}
                  </div>
                  <div class="flex items-center gap-1">
                    <button
                      onclick={() => handlePauseResume(torrent.id, torrent.state)}
                      class="rounded p-1 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
                      title={torrent.state === "paused" ? "Resume" : "Pause"}
                    >
                      {#if torrent.state === "paused"}
                        <Play class="h-3.5 w-3.5" />
                      {:else}
                        <Pause class="h-3.5 w-3.5" />
                      {/if}
                    </button>
                    <button
                      onclick={() => handleRemove(torrent.id)}
                      class="rounded p-1 text-[var(--color-text-muted)] hover:bg-[var(--color-error)]/10 hover:text-[var(--color-error)]"
                      title="Remove"
                    >
                      <X class="h-3.5 w-3.5" />
                    </button>
                  </div>
                </div>
              </div>
            {/each}
          {/if}
        </div>
      {/if}
    </div>

    <!-- Footer -->
    <div class="flex items-center justify-between border-t border-[var(--color-border)] px-3 py-2">
      <button
        onclick={openEditor}
        class="text-xs font-medium text-[var(--color-primary)] hover:underline"
      >
        Open whenThen
      </button>
      <button
        onclick={quitApp}
        class="text-xs text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
      >
        Quit
      </button>
    </div>
  </div>
{/if}

{#if torrentCtx.state}
  <ContextMenu x={torrentCtx.state.x} y={torrentCtx.state.y} items={torrentContextMenuItems(torrentCtx.state.data.id, torrentCtx.state.data.state)} onclose={torrentCtx.close} />
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

  .tab-indicator {
    transition: left 200ms cubic-bezier(0.16, 1, 0.3, 1);
  }
</style>
