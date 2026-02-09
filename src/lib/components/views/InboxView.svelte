<!-- RSS matches awaiting approval and active downloads. -->
<script lang="ts">
  import { Pause, Play, X, ThumbsUp, ThumbsDown, AlertTriangle, Film, FileText, Loader2, ChevronDown, ChevronUp, RefreshCw, Trash2, Cast, Ban, Search, Workflow, FolderOpen, Check, ListPlus, Link } from "lucide-svelte";
  import ContextMenu from "$lib/components/common/ContextMenu.svelte";
  import CastPopover from "$lib/components/common/CastPopover.svelte";
  import TaskHistoryRow from "$lib/components/common/TaskHistoryRow.svelte";
  import { useContextMenu } from "$lib/utils";
  import type { ContextMenuEntry } from "$lib/types/ui";
  import { torrentsState } from "$lib/state/torrents.svelte";
  import { feedsState, type PendingMatch } from "$lib/state/feeds.svelte";
  import { playletsState, derivePlayletName } from "$lib/state/playlets.svelte";
  import { settingsState } from "$lib/state/settings.svelte";
  import { uiState } from "$lib/state/ui.svelte";
  import { playbackState } from "$lib/state/playback.svelte";
  import { queueState } from "$lib/state/queue.svelte";
  import { devicesState } from "$lib/state/devices.svelte";
  import { torrentPause, torrentResume, torrentDelete, torrentRecheck, torrentFiles, runShellCommand, getPlaylistUrl } from "$lib/services/tauri-commands";
  import { tasksState } from "$lib/state/tasks.svelte";
  import { i18n } from "$lib/i18n/state.svelte";
  import { open as openShell } from "@tauri-apps/plugin-shell";
  import { buildActionSummary } from "$lib/utils/playlet-display";

  let expandedMatchId = $state<string | null>(null);
  let loadingMetadata = $state<string | null>(null);
  let approvingId = $state<string | null>(null);
  let refreshing = $state(false);
  let castPopover = $state<{ torrentId: number; name: string; x: number; y: number } | null>(null);
  let playletPicker = $state<{ taskId: string | null; torrentId: number; torrentName: string; x: number; y: number } | null>(null);
  
  const activeDownloads = $derived(torrentsState.activeTorrents);
  const completedTorrents = $derived(torrentsState.completedTorrents);
  const pendingMatches = $derived(feedsState.pendingMatches);
  const completedTasks = $derived(tasksState.completedTasks);
  const allPlaylets = $derived(playletsState.playlets);
  const hasFeedsConfigured = $derived(
    (feedsState.sources.length > 0 || feedsState.scrapers.length > 0) && feedsState.interests.length > 0
  );

  function getPlayletDisplayName(playlet: typeof playletsState.playlets[0]): string {
    if (playlet.name?.trim()) return playlet.name;
    return buildActionSummary(playlet) || "Playlet";
  }

  function getTaskForTorrent(torrentId: number) {
    return tasksState.getByTorrentId(torrentId);
  }

  function getPlayletName(task: ReturnType<typeof getTaskForTorrent>) {
    if (!task) return null;
    if (task.playletName) return task.playletName;
    if (task.playletId) {
      const playlet = playletsState.getById(task.playletId);
      if (!playlet) return null;
      // Use custom name if set, otherwise show verb summary
      if (playlet.name?.trim()) return playlet.name;
      return buildActionSummary(playlet) || null;
    }
    return null;
  }

  function openPlayletPicker(e: MouseEvent, torrentId: number, torrentName: string, taskId: string | null) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    playletPicker = { taskId, torrentId, torrentName, x: rect.right, y: rect.bottom + 4 };
  }

  function selectPlaylet(playletId: string) {
    if (!playletPicker) return;

    if (playletPicker.taskId) {
      // Reassign existing task
      tasksState.reassignTask(playletPicker.taskId, playletId);
    } else {
      // Create new task for torrent without one
      const playlet = playletsState.playlets.find(p => p.id === playletId);
      if (playlet) {
        const name = playlet.name || derivePlayletName(playlet);
        tasksState.createTask(
          playletPicker.torrentId,
          playletPicker.torrentName,
          playlet.id,
          name
        );
        uiState.addToast(i18n.t("toast.playletAssigned", { name }), "success");
      }
    }
    playletPicker = null;
  }

  function getTaskPlayletId(taskId: string | null): string | null {
    if (!taskId) return null;
    const task = tasksState.getById(taskId);
    return task?.playletId ?? null;
  }

  function openTorrentFolder() {
    const dir = settingsState.downloadDirectory;
    if (dir) {
      openShell(dir);
    }
  }

  async function clearCompletedTorrents() {
    for (const torrent of completedTorrents) {
      try {
        await torrentDelete(torrent.id, false);
        torrentsState.removeTorrent(torrent.id);
      } catch {}
    }
  }

  async function refreshPending() {
    refreshing = true;
    try {
      await feedsState.checkFeedsNow();
    } finally {
      refreshing = false;
    }
  }

  const torrentCtx = useContextMenu<{ id: number; state: string }>();
  const completedCtx = useContextMenu<{ id: number; infoHash: string; name: string }>();
  const historyCtx = useContextMenu<{ id: string }>();

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
        label: state === "paused" ? i18n.t("common.resume") : i18n.t("common.pause"),
        action: () => handlePauseResume(id, state),
      },
      {
        icon: RefreshCw,
        label: i18n.t("common.recheck"),
        action: () => handleRecheck(id),
      },
      { type: "divider" },
      {
        icon: X,
        label: i18n.t("common.remove"),
        action: () => handleRemove(id),
      },
      {
        icon: Trash2,
        label: i18n.t("common.removeWithFiles"),
        danger: true,
        action: () => handleDeleteWithFiles(id),
      },
    ];
  }

  async function handleMarkBad(id: number, infoHash: string, name: string, findAlternative: boolean) {
    try {
      const interestLink = feedsState.getTorrentInterest(id);
      const newMatches = await feedsState.markBad(
        infoHash,
        name,
        interestLink?.interestId,
        interestLink?.interestName,
        undefined,
        findAlternative
      );
      feedsState.clearTorrentInterest(id);
      await torrentDelete(id, true);
      torrentsState.removeTorrent(id);
      if (newMatches > 0) {
        await feedsState.loadPending();
      }
    } catch (e) {
      console.error("Failed to mark as bad:", e);
    }
  }

  function completedTorrentContextMenuItems(id: number, infoHash: string, name: string): ContextMenuEntry[] {
    const interestLink = feedsState.getTorrentInterest(id);
    const hasConnectedDevice = devicesState.hasConnectedDevice;
    const items: ContextMenuEntry[] = [
      {
        icon: Cast,
        label: i18n.t("actions.cast.label"),
        action: () => {
          // TODO: open cast popover for completed torrent
        },
      },
      {
        icon: ListPlus,
        label: i18n.t("playback.addAllToQueue"),
        action: () => addAllToQueue(id),
        disabled: !hasConnectedDevice,
      },
      {
        icon: Link,
        label: i18n.t("common.copyPlaylistUrl"),
        action: () => copyPlaylistUrl(id),
      },
      { type: "divider" },
      {
        icon: Ban,
        label: i18n.t("inbox.markAsBad"),
        action: () => handleMarkBad(id, infoHash, name, false),
      },
    ];

    if (interestLink) {
      items.push({
        icon: Search,
        label: i18n.t("inbox.markAsBadFindAlt"),
        action: () => handleMarkBad(id, infoHash, name, true),
      });
    }

    items.push(
      { type: "divider" },
      {
        icon: X,
        label: i18n.t("common.remove"),
        action: () => handleRemove(id),
      },
      {
        icon: Trash2,
        label: i18n.t("common.removeWithFiles"),
        danger: true,
        action: () => handleDeleteWithFiles(id),
      }
    );

    return items;
  }

  function historyContextMenuItems(id: string): ContextMenuEntry[] {
    return [
      {
        icon: X,
        label: i18n.t("common.remove"),
        action: () => tasksState.removeTask(id),
      },
    ];
  }

  async function toggleExpand(match: PendingMatch) {
    if (expandedMatchId === match.id) {
      expandedMatchId = null;
      return;
    }

    expandedMatchId = match.id;

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
    approvingId = matchId;
    try {
      await feedsState.approveMatch(matchId);
      if (expandedMatchId === matchId) expandedMatchId = null;
    } catch (err) {
      console.error("Failed to approve:", err);
    } finally {
      approvingId = null;
    }
  }

  async function rejectMatch(e: Event, matchId: string) {
    e.stopPropagation();
    try {
      await feedsState.rejectMatch(matchId);
      if (expandedMatchId === matchId) expandedMatchId = null;
    } catch (err) {
      console.error("Failed to reject:", err);
    }
  }

  function openCastPopover(e: MouseEvent, torrentId: number, name: string) {
    e.stopPropagation(); // Prevent click from bubbling to window and closing popover
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    castPopover = { torrentId, name, x: rect.right, y: rect.bottom + 4 };
  }

  async function openTorrentFolderById(torrentId: number) {
    const torrent = torrentsState.torrents.find(t => t.id === torrentId);
    const dir = settingsState.downloadDirectory;

    if (!dir) {
      uiState.addToast(i18n.t("settings.notSet"), "warning");
      return;
    }

    if (torrent?.name) {
      const path = `${dir}/${torrent.name}`;
      try {
        await runShellCommand(`open -R "${path}"`);
      } catch {
        // File might be moved/deleted, try opening directory
        try {
          await openShell(dir);
        } catch {
          uiState.addToast(i18n.t("toast.somethingWentWrong", { error: "Could not open folder" }), "error");
        }
      }
    } else {
      try {
        await openShell(dir);
      } catch {
        uiState.addToast(i18n.t("toast.somethingWentWrong", { error: "Could not open folder" }), "error");
      }
    }
  }

  async function clearAllPending() {
    try {
      await feedsState.rejectAllMatches();
      expandedMatchId = null;
    } catch (e) {
      console.error("Failed to clear pending:", e);
    }
  }

  async function addAllToQueue(torrentId: number) {
    try {
      const files = await torrentFiles(torrentId);
      const playableFiles = files.filter((f) => f.is_playable);

      if (playableFiles.length === 0) {
        return;
      }

      if (playableFiles.length === 1) {
        queueState.addToQueue(torrentId, playableFiles[0].index, playableFiles[0].name);
        uiState.addToast(i18n.t("playback.addedToQueue"), "success");
      } else {
        // Add all playable files to queue
        queueState.addBatch(torrentId, playableFiles.map(f => ({ index: f.index, name: f.name })));
        uiState.addToast(i18n.t("playback.addedCountToQueue", { count: playableFiles.length }), "success");
      }
    } catch (e) {
      console.error("Failed to add to queue:", e);
    }
  }

  async function copyPlaylistUrl(torrentId: number) {
    try {
      const url = await getPlaylistUrl(torrentId);
      await navigator.clipboard.writeText(url);
      uiState.addToast(i18n.t("common.copiedToClipboard"), "success");
    } catch (e) {
      console.error("Failed to copy playlist URL:", e);
    }
  }
</script>

<div class="mx-auto max-w-2xl space-y-6 p-6">
  <!-- Pending Matches Section (only if RSS sources or interests configured) -->
  {#if hasFeedsConfigured}
  <section>
    <div class="mb-3 flex items-center justify-between">
      <button onclick={() => uiState.toggleSection("pending")} class="flex items-center gap-2">
        {#if uiState.isSectionCollapsed("pending")}
          <ChevronDown class="h-4 w-4 text-[var(--color-text-muted)]" />
        {:else}
          <ChevronUp class="h-4 w-4 text-[var(--color-text-muted)]" />
        {/if}
        <h2 class="text-lg font-bold text-[var(--color-text)]">{i18n.t("inbox.pending")}</h2>
        {#if pendingMatches.length > 0}
          <span class="rounded-full bg-[var(--color-error)] px-2 py-0.5 text-xs font-medium text-white">
            {pendingMatches.length}
          </span>
        {/if}
      </button>
      <div class="flex items-center gap-1">
        <button
          onclick={refreshPending}
          disabled={refreshing}
          class="rounded-lg p-1.5 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)] disabled:opacity-50"
          title={i18n.t("inbox.refresh")}
        >
          <RefreshCw class="h-4 w-4 {refreshing ? 'animate-spin-ease' : ''}" />
        </button>
        {#if pendingMatches.length > 0}
          <button
            onclick={clearAllPending}
            class="rounded-lg p-1.5 text-[var(--color-text-muted)] hover:bg-[var(--color-error)]/10 hover:text-[var(--color-error)]"
            title={i18n.t("inbox.clearAll")}
          >
            <Trash2 class="h-4 w-4" />
          </button>
        {/if}
      </div>
    </div>

    <div class="expand-container {uiState.collapsedSections.has('pending') ? '' : 'expanded'}">
      <div class="expand-content">
    {#if pendingMatches.length === 0}
      <div class="py-12 text-center">
        <p class="text-sm text-[var(--color-text-muted)]">{i18n.t("inbox.feedsQuiet")}</p>
        <p class="mt-1 text-xs text-[var(--color-text-muted)]">{i18n.t("inbox.feedsQuietDescription")}</p>
      </div>
    {:else}
      <div class="divide-y divide-[var(--color-border)] overflow-hidden rounded-xl border border-[var(--color-border)]">
        {#each pendingMatches as match (match.id)}
          {@const isExpanded = expandedMatchId === match.id}
          {@const isLoading = loadingMetadata === match.id}
          {@const isApproving = approvingId === match.id}
          <div class="group bg-[var(--color-bg)]">
            <div class="flex items-start gap-3 p-4">
              <!-- Approve/Reject buttons -->
              <div class="flex shrink-0 items-center gap-1">
                <button
                  onclick={(e) => approveMatch(e, match.id)}
                  disabled={isApproving}
                  class="rounded-lg p-2 text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-success)]/10 hover:text-[var(--color-success)] disabled:opacity-50"
                  title={i18n.t("inbox.download")}
                >
                  {#if isApproving}
                    <Loader2 class="h-5 w-5 animate-spin" />
                  {:else}
                    <ThumbsUp class="h-5 w-5" />
                  {/if}
                </button>
                <button
                  onclick={(e) => rejectMatch(e, match.id)}
                  class="rounded-lg p-2 text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-error)]/10 hover:text-[var(--color-error)]"
                  title={i18n.t("inbox.skip")}
                >
                  <ThumbsDown class="h-5 w-5" />
                </button>
              </div>

              <!-- Content -->
              <button
                onclick={() => toggleExpand(match)}
                class="flex min-w-0 flex-1 flex-col gap-1 text-left"
              >
                <div class="flex items-center gap-2">
                  <span class="rounded bg-[var(--color-primary)]/10 px-2 py-0.5 text-xs font-medium text-[var(--color-primary)]">
                    {match.interestName}
                  </span>
                  <span class="text-xs text-[var(--color-text-muted)]">
                    {i18n.t("inbox.via", { source: match.sourceName })}
                  </span>
                  {#if isExpanded}
                    <ChevronUp class="ml-auto h-4 w-4 text-[var(--color-text-muted)]" />
                  {:else}
                    <ChevronDown class="ml-auto h-4 w-4 text-[var(--color-text-muted)]" />
                  {/if}
                </div>
                <span class="line-clamp-2 text-sm font-medium text-[var(--color-text)]">
                  {match.title}
                </span>
              </button>
            </div>

            <!-- Expanded file preview -->
            <div class="expand-container {isExpanded ? 'expanded' : ''}">
              <div class="expand-content">
                <div class="border-t border-[var(--color-border)] bg-[var(--color-bg-secondary)] px-4 py-3">
                  {#if isLoading}
                  <div class="flex items-center justify-center gap-2 py-6">
                    <Loader2 class="h-4 w-4 animate-spin text-[var(--color-text-muted)]" />
                    <span class="text-sm text-[var(--color-text-muted)]">{i18n.t("inbox.fetchingFileList")}</span>
                  </div>
                {:else if match.metadata}
                  {@const meta = match.metadata}
                  {@const hasSuspicious = meta.files.some((f) => f.isSuspicious)}
                  {@const videoCount = meta.files.filter((f) => f.isVideo).length}

                  <div class="mb-3 flex items-center gap-4 text-sm">
                    <span class="text-[var(--color-text-muted)]">{i18n.t("inbox.filesCount", { count: meta.fileCount })}</span>
                    <span class="text-[var(--color-text-muted)]">{formatSize(meta.totalSize)}</span>
                    {#if videoCount > 0}
                      <span class="flex items-center gap-1.5 text-[var(--color-success)]">
                        <Film class="h-4 w-4" />
                        {videoCount !== 1 ? i18n.t("inbox.videosCount", { count: videoCount }) : i18n.t("inbox.videoCount", { count: videoCount })}
                      </span>
                    {/if}
                    {#if hasSuspicious}
                      <span class="flex items-center gap-1.5 text-[var(--color-error)]">
                        <AlertTriangle class="h-4 w-4" />
                        {i18n.t("inbox.suspiciousFiles")}
                      </span>
                    {/if}
                  </div>

                  <div class="max-h-48 space-y-1 overflow-y-auto">
                    {#each meta.files.slice(0, 20) as file}
                      <div class="flex items-center gap-2 text-sm">
                        {#if file.isSuspicious}
                          <AlertTriangle class="h-4 w-4 shrink-0 text-[var(--color-error)]" />
                        {:else if file.isVideo}
                          <Film class="h-4 w-4 shrink-0 text-[var(--color-success)]" />
                        {:else}
                          <FileText class="h-4 w-4 shrink-0 text-[var(--color-text-muted)]" />
                        {/if}
                        <span class="min-w-0 flex-1 truncate {file.isSuspicious ? 'text-[var(--color-error)]' : 'text-[var(--color-text)]'}">
                          {file.name}
                        </span>
                        <span class="shrink-0 text-xs text-[var(--color-text-muted)]">{formatSize(file.size)}</span>
                      </div>
                    {/each}
                    {#if meta.files.length > 20}
                      <div class="pt-1 text-sm text-[var(--color-text-muted)]">
                        {i18n.t("inbox.moreFiles", { count: meta.files.length - 20 })}
                      </div>
                    {/if}
                  </div>
                  {:else}
                    <div class="py-4 text-center text-sm text-[var(--color-text-muted)]">
                      {i18n.t("inbox.couldntPreviewFiles")}
                    </div>
                  {/if}
                </div>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
      </div>
    </div>
  </section>
  {/if}

  <!-- Downloads Section -->
  <section>
    <div class="mb-3 flex items-center justify-between">
      <button onclick={() => uiState.toggleSection("downloads")} class="flex items-center gap-2">
        {#if uiState.isSectionCollapsed("downloads")}
          <ChevronDown class="h-4 w-4 text-[var(--color-text-muted)]" />
        {:else}
          <ChevronUp class="h-4 w-4 text-[var(--color-text-muted)]" />
        {/if}
        <h2 class="text-lg font-bold text-[var(--color-text)]">{i18n.t("inbox.downloads")}</h2>
      </button>
      {#if activeDownloads.length > 0}
        <span class="text-sm text-[var(--color-text-muted)]">
          {i18n.t("inbox.activeCount", { count: activeDownloads.length })}
        </span>
      {/if}
    </div>

    <div class="expand-container {uiState.collapsedSections.has('downloads') ? '' : 'expanded'}">
      <div class="expand-content">
    {#if activeDownloads.length === 0}
      <div class="py-12 text-center">
        <p class="text-sm text-[var(--color-text-muted)]">{i18n.t("inbox.noActiveDownloads")}</p>
      </div>
    {:else}
      <div class="space-y-3">
        {#each activeDownloads as torrent (torrent.id)}
          {@const task = getTaskForTorrent(torrent.id)}
          {@const playletName = getPlayletName(task)}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="rounded-xl bg-[var(--color-bg-secondary)] p-4"
            oncontextmenu={(e) => torrentCtx.open(e, { id: torrent.id, state: torrent.state })}
          >
            <div class="mb-2 flex items-start justify-between gap-3">
              <span class="min-w-0 flex-1 truncate font-medium text-[var(--color-text)]">
                {torrent.name}
              </span>
              <div class="flex shrink-0 items-center gap-1">
                <button
                  onclick={(e) => openCastPopover(e, torrent.id, torrent.name)}
                  class="rounded-lg p-1.5 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
                  title={i18n.t("actions.cast.label")}
                >
                  <Cast class="h-4 w-4" />
                </button>
                <button
                  onclick={() => handlePauseResume(torrent.id, torrent.state)}
                  class="rounded-lg p-1.5 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
                  title={torrent.state === "paused" ? i18n.t("common.resume") : i18n.t("common.pause")}
                >
                  {#if torrent.state === "paused"}
                    <Play class="h-4 w-4" />
                  {:else}
                    <Pause class="h-4 w-4" />
                  {/if}
                </button>
              </div>
            </div>
            <div class="mb-2 h-2 overflow-hidden rounded-full bg-[var(--color-bg-tertiary)]">
              <div
                class="h-full rounded-full transition-all duration-300 {torrent.state === 'paused' ? 'bg-[var(--color-warning)]' : 'bg-[var(--color-primary)]'}"
                style="width: {torrent.progress * 100}%"
              ></div>
            </div>
            <div class="flex items-center justify-between text-sm text-[var(--color-text-muted)]">
              <div class="flex items-center gap-3">
                <span class="font-medium">{formatPercent(torrent.progress)}</span>
                {#if torrent.state !== "paused"}
                  <span>{formatSpeed(torrent.download_speed)}</span>
                  {#if torrent.peers_connected > 0 || torrent.queued_peers > 0 || torrent.connecting_peers > 0}
                    {@const total = torrent.peers_connected + (torrent.queued_peers ?? 0) + (torrent.connecting_peers ?? 0)}
                    <span title="{torrent.peers_connected} {i18n.t("common.live")}, {torrent.connecting_peers ?? 0} {i18n.t("common.connecting")}, {torrent.queued_peers ?? 0} {i18n.t("common.queued")}">• {total} {total === 1 ? i18n.t("common.peer") : i18n.t("common.peers")}</span>
                  {/if}
                {:else}
                  <span class="text-[var(--color-warning)]">{i18n.t("inbox.paused")}</span>
                {/if}
              </div>
              <button
                onclick={(e) => openPlayletPicker(e, torrent.id, torrent.name, task?.id ?? null)}
                class="flex items-center gap-1 text-xs {task ? 'text-[var(--color-primary)]' : 'text-[var(--color-text-muted)]'} hover:underline"
              >
                <Workflow class="h-3 w-3" />
                <span>{playletName ?? i18n.t("inbox.assignPlaylet")}</span>
                <ChevronDown class="h-3 w-3" />
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
      </div>
    </div>
  </section>

  <!-- Completed Section -->
  {#if completedTorrents.length > 0}
    <section>
      <div class="mb-3 flex items-center justify-between">
        <button onclick={() => uiState.toggleSection("completed")} class="flex items-center gap-2">
          {#if uiState.isSectionCollapsed("completed")}
            <ChevronDown class="h-4 w-4 text-[var(--color-text-muted)]" />
          {:else}
            <ChevronUp class="h-4 w-4 text-[var(--color-text-muted)]" />
          {/if}
          <h2 class="text-lg font-bold text-[var(--color-text)]">{i18n.t("inbox.completed")}</h2>
        </button>
        <button
          onclick={clearCompletedTorrents}
          class="rounded-lg p-1.5 text-[var(--color-text-muted)] hover:bg-[var(--color-error)]/10 hover:text-[var(--color-error)]"
          title={i18n.t("inbox.clearCompleted")}
        >
          <Trash2 class="h-4 w-4" />
        </button>
      </div>

      <div class="expand-container {uiState.collapsedSections.has('completed') ? '' : 'expanded'}">
        <div class="expand-content">
          <div class="space-y-2">
        {#each completedTorrents as torrent (torrent.id)}
          {@const isNowPlaying = playbackState.activeTorrentId === torrent.id}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="flex items-center justify-between gap-3 rounded-xl p-3 {isNowPlaying ? 'ring-2 ring-[var(--color-primary)] bg-[var(--color-primary)]/5' : 'bg-[var(--color-bg-secondary)]'}"
            oncontextmenu={(e) => completedCtx.open(e, { id: torrent.id, infoHash: torrent.info_hash, name: torrent.name })}
          >
            <div class="flex min-w-0 flex-1 items-center gap-2">
              {#if isNowPlaying}
                <Cast class="h-4 w-4 shrink-0 text-[var(--color-primary)] animate-pulse" />
              {/if}
              <span class="min-w-0 flex-1 truncate text-sm text-[var(--color-text)]">
                {torrent.name}
              </span>
            </div>
            <div class="flex shrink-0 items-center gap-1">
              <button
                onclick={(e) => openCastPopover(e, torrent.id, torrent.name)}
                class="rounded-lg p-1.5 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
                title={i18n.t("actions.cast.label")}
              >
                <Cast class="h-4 w-4" />
              </button>
              <button
                onclick={() => openTorrentFolderById(torrent.id)}
                class="rounded-lg p-1.5 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
                title={i18n.t("common.revealInFinder")}
              >
                <FolderOpen class="h-4 w-4" />
              </button>
            </div>
          </div>
        {/each}
          </div>
        </div>
      </div>
    </section>
  {/if}

  <!-- History Section -->
  {#if completedTasks.length > 0}
    <section>
      <div class="mb-3 flex items-center justify-between">
        <button onclick={() => uiState.toggleSection("history")} class="flex items-center gap-2">
          {#if uiState.isSectionCollapsed("history")}
            <ChevronDown class="h-4 w-4 text-[var(--color-text-muted)]" />
          {:else}
            <ChevronUp class="h-4 w-4 text-[var(--color-text-muted)]" />
          {/if}
          <h2 class="text-lg font-bold text-[var(--color-text)]">{i18n.t("inbox.history")}</h2>
        </button>
        <button
          onclick={() => tasksState.clearCompleted()}
          class="rounded-lg p-1.5 text-[var(--color-text-muted)] hover:bg-[var(--color-error)]/10 hover:text-[var(--color-error)]"
          title={i18n.t("inbox.clearHistory")}
        >
          <Trash2 class="h-4 w-4" />
        </button>
      </div>

      <div class="expand-container {uiState.collapsedSections.has('history') ? '' : 'expanded'}">
        <div class="expand-content">
          <div class="space-y-2">
            {#each completedTasks as task (task.id)}
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div oncontextmenu={(e) => historyCtx.open(e, { id: task.id })}>
                <TaskHistoryRow {task} />
              </div>
            {/each}
          </div>
        </div>
      </div>
    </section>
  {/if}
</div>

{#if torrentCtx.state}
  <ContextMenu x={torrentCtx.state.x} y={torrentCtx.state.y} items={torrentContextMenuItems(torrentCtx.state.data.id, torrentCtx.state.data.state)} onclose={torrentCtx.close} />
{/if}

{#if completedCtx.state}
  <ContextMenu x={completedCtx.state.x} y={completedCtx.state.y} items={completedTorrentContextMenuItems(completedCtx.state.data.id, completedCtx.state.data.infoHash, completedCtx.state.data.name)} onclose={completedCtx.close} />
{/if}

{#if historyCtx.state}
  <ContextMenu x={historyCtx.state.x} y={historyCtx.state.y} items={historyContextMenuItems(historyCtx.state.data.id)} onclose={historyCtx.close} />
{/if}

{#if castPopover}
  <CastPopover
    torrentId={castPopover.torrentId}
    torrentName={castPopover.name}
    x={castPopover.x}
    y={castPopover.y}
    onClose={() => castPopover = null}
  />
{/if}

{#if playletPicker}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-50"
    onclick={() => playletPicker = null}
    onkeydown={(e) => { if (e.key === "Escape") playletPicker = null; }}
  >
    <div
      class="absolute rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] py-1 shadow-lg"
      style="left: {playletPicker.x}px; top: {playletPicker.y}px; min-width: 280px; max-width: 400px; transform: translateX(-100%);"
      onclick={(e) => e.stopPropagation()}
    >
      {#if allPlaylets.length === 0}
        <div class="px-3 py-2 text-sm text-[var(--color-text-muted)]">
          {i18n.t("inbox.noPlaylets")}
        </div>
      {:else}
        {@const currentPlayletId = getTaskPlayletId(playletPicker.taskId)}
        {#each allPlaylets as playlet}
          {@const isSelected = playlet.id === currentPlayletId}
          <button
            onclick={() => selectPlaylet(playlet.id)}
            class="flex w-full items-center gap-2 px-3 py-2 text-left text-sm hover:bg-[var(--color-bg-secondary)] {isSelected ? 'text-[var(--color-primary)]' : 'text-[var(--color-text)]'}"
          >
            {#if isSelected}
              <Check class="h-4 w-4 text-[var(--color-primary)]" />
            {:else}
              <Workflow class="h-4 w-4 text-[var(--color-text-muted)]" />
            {/if}
            {getPlayletDisplayName(playlet)}
          </button>
        {/each}
      {/if}
    </div>
  </div>
{/if}
