<!-- Main app layout with sidebar navigation. -->
<script lang="ts">
  import Sidebar from "./Sidebar.svelte";
  import InboxView from "$lib/components/views/InboxView.svelte";
  import PlayletsView from "$lib/components/views/PlayletsView.svelte";
  import SettingsView from "$lib/components/views/SettingsView.svelte";
  import Toast from "$lib/components/common/Toast.svelte";
  import PlaybackBar from "$lib/components/common/PlaybackBar.svelte";
  import { uiState } from "$lib/state/ui.svelte";
  import { playbackState } from "$lib/state/playback.svelte";

  const showPlaybackBar = $derived(playbackState.isPlaying || playbackState.isPaused || playbackState.activeTorrentName !== null);
</script>

<div class="flex h-full bg-[var(--color-bg)]">
  <Sidebar />

  <div class="flex min-w-0 flex-1 flex-col">
    <!-- Header with drag region -->
    <header class="flex h-14 shrink-0 items-center pl-4 pr-4" data-tauri-drag-region>
      <h1 class="pointer-events-none text-sm font-semibold text-[var(--color-text)]">When</h1>
    </header>

    <!-- Main content area -->
    <main class="min-h-0 flex-1 overflow-y-auto">
      {#if uiState.activeView === "inbox"}
        <InboxView />
      {:else if uiState.activeView === "rules"}
        <PlayletsView />
      {:else if uiState.activeView === "settings"}
        <SettingsView />
      {/if}
    </main>

    {#if showPlaybackBar}
      <PlaybackBar />
    {/if}
  </div>
</div>

<!-- Toast notifications -->
{#if uiState.toasts.length > 0}
  <div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2">
    {#each uiState.toasts as toast (toast.id)}
      <Toast {toast} />
    {/each}
  </div>
{/if}
