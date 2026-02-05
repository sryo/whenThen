<script lang="ts">
  import { Trash2, X, Loader2 } from "lucide-svelte";
  import TaskCard from "$lib/components/common/TaskCard.svelte";
  import TorrentFileBrowser from "$lib/components/common/TorrentFileBrowser.svelte";
  import { tasksState } from "$lib/state/tasks.svelte";
  import { uiState } from "$lib/state/ui.svelte";
  import { torrentFiles } from "$lib/services/tauri-commands";
  import type { Task } from "$lib/types/task";
  import type { TorrentFileInfo } from "$lib/types/torrent";

  // Sidebar state
  let sidebarTask = $state<Task | null>(null);
  let sidebarFiles = $state<TorrentFileInfo[]>([]);
  let sidebarLoading = $state(false);
  let sidebarClosing = $state(false);

  const sidebarOpen = $derived(sidebarTask !== null);

  async function openFileSidebar(task: Task) {
    sidebarLoading = true;
    sidebarTask = task;
    sidebarClosing = false;
    try {
      const files = await torrentFiles(task.torrentId);
      sidebarFiles = files;
    } catch (err: any) {
      uiState.addToast(`Could not load files: ${err?.message || err}`, "error");
      sidebarTask = null;
    } finally {
      sidebarLoading = false;
    }
  }

  function closeFileSidebar() {
    sidebarClosing = true;
    setTimeout(() => {
      sidebarTask = null;
      sidebarFiles = [];
      sidebarClosing = false;
    }, 200);
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) closeFileSidebar();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") closeFileSidebar();
  }
</script>

<div class="mx-auto max-w-2xl space-y-4 p-6">
  <!-- Active Tasks -->
  <div>
    {#if tasksState.activeTasks.length > 0}
      <div class="space-y-2">
        {#each tasksState.activeTasks as task (task.id)}
          <TaskCard {task} onShowFiles={openFileSidebar} />
        {/each}
      </div>
    {:else}
      <p class="py-8 text-center text-sm text-[var(--color-text-muted)]">
        Nothing running. Drop a torrent on a playlet to start.
      </p>
    {/if}
  </div>

  <!-- Completed Tasks -->
  {#if tasksState.completedTasks.length > 0}
    <div>
      <div class="mb-3 flex items-center justify-between">
        <h2 class="text-sm font-semibold uppercase tracking-wider text-[var(--color-text-muted)]">
          Done
        </h2>
        <button
          onclick={() => tasksState.clearCompleted()}
          class="flex items-center gap-1 rounded-md px-2 py-1 text-xs text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
        >
          <Trash2 class="h-3 w-3" />
          Clear
        </button>
      </div>
      <div class="space-y-2">
        {#each tasksState.completedTasks as task (task.id)}
          <TaskCard {task} onShowFiles={openFileSidebar} />
        {/each}
      </div>
    </div>
  {/if}
</div>

<!-- File browser sidebar -->
{#if sidebarOpen}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    role="dialog"
    aria-modal="true"
    aria-label="Torrent files"
    tabindex="-1"
    class="fixed inset-0 z-50 flex justify-end {sidebarClosing ? 'backdrop-fade-out' : 'backdrop-fade-in'}"
    onclick={handleBackdropClick}
    onkeydown={handleKeyDown}
  >
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="{sidebarClosing ? 'sidebar-slide-out' : 'sidebar-slide-in'} flex h-full w-full max-w-md flex-col border-l border-[var(--color-border)] bg-[var(--color-bg)] shadow-2xl"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-[var(--color-border)] px-4 py-3">
        <div class="min-w-0">
          <h2 class="text-sm font-bold text-[var(--color-text)]">Files</h2>
          {#if sidebarTask}
            <p class="truncate text-xs text-[var(--color-text-muted)]">{sidebarTask.torrentName}</p>
          {/if}
        </div>
        <button
          onclick={closeFileSidebar}
          class="shrink-0 rounded-lg p-1.5 text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
        >
          <X class="h-4 w-4" />
        </button>
      </div>

      <!-- Body -->
      <div class="flex-1 overflow-y-auto p-3">
        {#if sidebarLoading}
          <div class="flex items-center justify-center py-12">
            <Loader2 class="h-5 w-5 animate-spin text-[var(--color-text-muted)]" />
          </div>
        {:else if sidebarTask}
          <TorrentFileBrowser
            files={sidebarFiles}
            torrentId={sidebarTask.torrentId}
          />
        {/if}
      </div>

    </div>
  </div>
{/if}

<style>
  .sidebar-slide-in {
    animation: slideIn 200ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }
  .sidebar-slide-out {
    animation: slideOut 200ms cubic-bezier(0.7, 0, 0.84, 0) forwards;
  }
  .backdrop-fade-in {
    animation: fadeIn 200ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }
  .backdrop-fade-out {
    animation: fadeOut 200ms cubic-bezier(0.7, 0, 0.84, 0) forwards;
  }

  @keyframes slideIn {
    from { transform: translateX(100%); }
    to   { transform: translateX(0); }
  }
  @keyframes slideOut {
    from { transform: translateX(0); }
    to   { transform: translateX(100%); }
  }
  @keyframes fadeIn {
    from { background-color: transparent; }
    to   { background-color: rgb(0 0 0 / 0.3); }
  }
  @keyframes fadeOut {
    from { background-color: rgb(0 0 0 / 0.3); }
    to   { background-color: transparent; }
  }
</style>
