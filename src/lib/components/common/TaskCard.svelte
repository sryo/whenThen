<script lang="ts">
  import type { Component } from "svelte";
  import type { IconType } from "$lib/types/ui";
  import {
    Check,
    ChevronDown,
    Download,
    FolderOpen,
    RotateCcw,
    Trash2,
    FileX,
    Pause,
    Play,
  } from "lucide-svelte";
  import { open } from "@tauri-apps/plugin-shell";
  import type { Task } from "$lib/types/task";
  import type { Playlet } from "$lib/types/playlet";
  import type { ContextMenuEntry } from "$lib/types/ui";
  import { torrentsState } from "$lib/state/torrents.svelte";
  import { tasksState } from "$lib/state/tasks.svelte";
  import { playletsState, derivePlayletName } from "$lib/state/playlets.svelte";
  import { settingsState } from "$lib/state/settings.svelte";
  import { retryTask } from "$lib/services/execution-pipeline";
  import { torrentDelete, torrentPause, torrentResume } from "$lib/services/tauri-commands";
  import { formatProgress, formatBytes, formatSpeed } from "$lib/utils/format";
  import { getActionDef, getActionLabel } from "$lib/services/action-registry";
  import ActionBlock from "$lib/components/common/ActionBlock.svelte";
  import ContextMenu from "$lib/components/common/ContextMenu.svelte";
  import { useContextMenu } from "$lib/utils";

  let { task }: { task: Task } = $props();

  const torrent = $derived(
    torrentsState.torrents.find((t) => t.id === task.torrentId),
  );

  const statusLabel = $derived.by(() => {
    switch (task.status) {
      case "waiting": {
        if (!torrent || torrent.state === "completed") return "Ready";
        if (torrent.state === "initializing") {
          return torrent.total_bytes === 0 ? "Magnetizing" : "Checking";
        }
        if (torrent.state === "paused") return "Paused";
        if (torrent.state === "error") return "Error";
        return torrent.peers_connected === 0 ? "Connecting" : "Downloading";
      }
      case "executing": return "Running";
      case "completed": return "Done";
      case "failed": return "Failed";
    }
  });

  let showPlayletPicker = $state(false);

  const enabledPlaylets = $derived(
    playletsState.playlets.filter((p) => p.enabled),
  );

  const canReassign = $derived(
    task.status === "waiting" && enabledPlaylets.length > 1,
  );

  function handleReassign(playletId: string) {
    tasksState.reassignTask(task.id, playletId);
    showPlayletPicker = false;
  }

  const ctx = useContextMenu();

  function buildContextItems(): ContextMenuEntry[] {
    const items: ContextMenuEntry[] = [];
    if (isDownloading) {
      items.push({ icon: Pause, label: "Pause", action: () => { if (torrent) torrentPause(torrent.id); } });
    } else if (isPaused) {
      items.push({ icon: Play, label: "Resume", action: () => { if (torrent) torrentResume(torrent.id); } });
    }
    if (task.status === "failed") {
      items.push({ icon: RotateCcw, label: "Retry", action: () => retryTask(task.id) });
    }
    items.push({
      icon: Trash2,
      label: "Remove",
      action: () => tasksState.removeTask(task.id),
    });
    items.push({
      icon: FileX,
      label: "Delete with files",
      danger: true,
      action: async () => {
        try {
          await torrentDelete(task.torrentId, true);
          torrentsState.removeTorrent(task.torrentId);
        } catch {}

        tasksState.removeTask(task.id);
      },
    });
    return items;
  }

  // Torrent is actively downloading or initializing (not completed, not paused)
  const isDownloading = $derived(
    torrent && task.status === "waiting" && (torrent.state === "downloading" || torrent.state === "initializing"),
  );

  const isPaused = $derived(
    torrent && task.status === "waiting" && torrent.state === "paused",
  );

  function handlePause() {
    if (torrent) torrentPause(torrent.id);
  }

  function handleResume() {
    if (torrent) torrentResume(torrent.id);
  }

  function handleOpenFolder() {
    open(settingsState.downloadDirectory);
  }

  const eta = $derived.by(() => {
    if (!torrent || torrent.download_speed <= 0 || torrent.total_bytes <= 0) return null;
    const remaining = torrent.total_bytes - torrent.downloaded_bytes;
    if (remaining <= 0) return null;
    const secs = remaining / torrent.download_speed;
    if (secs > 86400) return ">1d";
    if (secs > 3600) return `${Math.floor(secs / 3600)}h ${Math.floor((secs % 3600) / 60)}m`;
    if (secs > 60) return `${Math.floor(secs / 60)}m`;
    return `${Math.floor(secs)}s`;
  });

  // Step info from action results
  interface StepInfo {
    icon: IconType;
    color: string;
    label: string;
    value: string | null;
    status: "done" | "running" | "pending" | "failed" | "skipped";
    error: string | null;
  }

  // Look up the playlet to get configured action values
  const playlet = $derived(
    task.playletId ? playletsState.getById(task.playletId) : null,
  );

  const steps = $derived.by((): StepInfo[] => {
    // Synthetic download step
    let dlStatus: StepInfo["status"];
    if (task.status === "waiting") {
      if (!torrent || torrent.state === "completed") {
        dlStatus = "done";
      } else if (torrent.state === "error") {
        dlStatus = "failed";
      } else if (torrent.state === "paused") {
        dlStatus = "pending";
      } else {
        dlStatus = "running";
      }
    } else {
      dlStatus = "done";
    }

    const downloadStep: StepInfo = {
      icon: Download,
      color: "var(--color-info)",
      label: "Download",
      value: null,
      status: dlStatus,
      error: null,
    };

    const actionSteps = task.actionResults.map((result) => {
      const def = getActionDef(result.actionType);
      const action = playlet?.actions.find((a) => a.id === result.actionId);
      const configured = action ? getActionLabel(action) : null;
      return {
        icon: def?.icon ?? Check,
        color: def?.color ?? "var(--color-text-muted)",
        label: def?.label ?? result.actionType,
        value: configured,
        status: result.status as StepInfo["status"],
        error: result.error,
      };
    });

    return [downloadStep, ...actionSteps];
  });

  // Build action blocks for a playlet (picker dropdown)
  interface BlockInfo {
    icon: IconType;
    color: string;
    label: string;
    value: string | null;
  }

  function buildPlayletBlocks(p: Playlet): BlockInfo[] {
    return p.actions.map((a) => {
      const def = getActionDef(a.type);
      const configured = getActionLabel(a);
      return {
        icon: def?.icon ?? Check,
        color: def?.color ?? "var(--color-text-muted)",
        label: def?.label ?? a.type,
        value: configured,
      };
    });
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-3"
  oncontextmenu={(e) => ctx.open(e)}
>
  <!-- Header: torrent name + pipeline pips (right-aligned) -->
  <div class="flex items-center justify-between gap-2">
    <h4 class="min-w-0 truncate text-sm font-bold text-[var(--color-text)]">
      {task.torrentName}
    </h4>

    {#if steps.length > 0}
      <div class="relative shrink-0">
        {#if canReassign}
          <button
            onclick={() => { showPlayletPicker = !showPlayletPicker; }}
            class="flex items-center gap-y-1 rounded-lg px-1 py-0.5 transition-colors hover:bg-[var(--color-bg-tertiary)]/50"
          >
            {#each steps as step, i}
              {#if i > 0}
                <div class="h-[2px] w-1.5 rounded-full bg-[var(--color-border)]"></div>
              {/if}
              <ActionBlock icon={step.icon} color={step.color} label={step.label} value={step.value} size="pip" status={step.status} />
            {/each}
            <ChevronDown class="ml-0.5 h-3 w-3 text-[var(--color-text-muted)]" />
          </button>
        {:else}
          <div class="flex items-center gap-y-1 px-1 py-0.5">
            {#each steps as step, i}
              {#if i > 0}
                <div class="h-[2px] w-1.5 rounded-full bg-[var(--color-border)]"></div>
              {/if}
              <ActionBlock icon={step.icon} color={step.color} label={step.label} value={step.value} size="pip" status={step.status} />
            {/each}
          </div>
        {/if}

        <!-- Playlet picker dropdown -->
        {#if showPlayletPicker}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <div
            class="fixed inset-0 z-40"
            onclick={() => { showPlayletPicker = false; }}
          ></div>
          <div class="absolute right-0 top-full z-50 mt-1 max-h-64 overflow-y-auto rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] py-1 shadow-xl">
            {#each enabledPlaylets as playlet}
              {@const blocks = buildPlayletBlocks(playlet)}
              <button
                onclick={() => handleReassign(playlet.id)}
                class="flex w-full flex-col gap-1.5 px-3 py-2 text-left transition-colors hover:bg-[var(--color-bg-tertiary)] {playlet.id === task.playletId ? 'bg-[var(--color-bg-tertiary)]' : ''}"
              >
                <div class="flex flex-nowrap items-center">
                  <ActionBlock icon={Download} color="var(--color-info)" label="Download" size="pip" />
                  {#each blocks as block}
                    <div class="h-[2px] w-2 rounded-full bg-[var(--color-border)]"></div>
                    <ActionBlock icon={block.icon} color={block.color} label={block.label} value={block.value} size="pip" />
                  {/each}
                </div>
                <span class="whitespace-nowrap text-xs {playlet.id === task.playletId ? 'font-bold text-[var(--color-primary)]' : 'text-[var(--color-text-muted)]'}">
                  {derivePlayletName(playlet)}
                </span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Download progress bar + controls -->
  {#if torrent && task.status === "waiting" && torrent.state !== "completed"}
    <div class="mt-2 flex items-center gap-1.5">
      <div class="h-1.5 flex-1 overflow-hidden rounded-full bg-[var(--color-bg-tertiary)]">
        <div
          class="h-full rounded-full bg-[var(--color-primary)] transition-all"
          style="width: {torrent.progress * 100}%"
        ></div>
      </div>
      {#if isDownloading}
        <button
          onclick={handlePause}
          class="shrink-0 rounded p-1 text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
          title="Pause"
        >
          <Pause class="h-3.5 w-3.5" />
        </button>
      {:else if isPaused}
        <button
          onclick={handleResume}
          class="shrink-0 rounded p-1 text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
          title="Resume"
        >
          <Play class="h-3.5 w-3.5" />
        </button>
      {/if}
      <button
        onclick={handleOpenFolder}
        class="shrink-0 rounded p-1 text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
        title="Open folder"
      >
        <FolderOpen class="h-3.5 w-3.5" />
      </button>
    </div>

    <!-- Torrent stats -->
    <div class="mt-1 flex flex-wrap items-center gap-x-2 text-[10px] text-[var(--color-text-muted)]">
      <span>{statusLabel}</span>
      {#if torrent.total_bytes > 0}
        <span>{formatBytes(torrent.downloaded_bytes)} / {formatBytes(torrent.total_bytes)} ({formatProgress(torrent.progress)})</span>
      {/if}
      {#if eta}
        <span>~{eta}</span>
      {/if}
      {#if torrent.peers_connected > 0}
        <span>{torrent.peers_connected} {torrent.peers_connected === 1 ? "peer" : "peers"}</span>
      {/if}
      {#if torrent.download_speed > 0}
        <span>↓ {formatSpeed(torrent.download_speed)}</span>
      {/if}
      {#if torrent.upload_speed > 0}
        <span>↑ {formatSpeed(torrent.upload_speed)}</span>
      {/if}
    </div>
  {/if}

  <!-- Error messages for failed steps -->
  {#if steps.length > 0}
    {#each steps as step}
      {#if step.status === "failed" && step.error}
        <p class="select-text mt-1.5 truncate text-xs text-[var(--color-error)]">
          {step.error}
        </p>
      {/if}
    {/each}
  {/if}
</div>

{#if ctx.state}
  <ContextMenu x={ctx.state.x} y={ctx.state.y} items={buildContextItems()} onclose={ctx.close} />
{/if}
