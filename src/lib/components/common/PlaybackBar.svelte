<!-- Persistent playback toolbar for Chromecast control. -->
<script lang="ts">
  import { Cast, Play, Pause, SkipForward, SkipBack, ChevronDown, Tv, X } from "lucide-svelte";
  import { playbackState } from "$lib/state/playback.svelte";
  import { queueState } from "$lib/state/queue.svelte";
  import { devicesState } from "$lib/state/devices.svelte";
  import { playbackPlay, playbackPause, playbackSeek, playbackCastTorrent, playbackStop } from "$lib/services/tauri-commands";
  import { i18n } from "$lib/i18n/state.svelte";

  let seeking = $state(false);
  let seekValue = $state(0);
  let devicePickerOpen = $state(false);

  const currentTime = $derived(playbackState.currentTime);
  const duration = $derived(playbackState.duration);
  const isPlaying = $derived(playbackState.isPlaying);
  const isPaused = $derived(playbackState.isPaused);
  const deviceId = $derived(playbackState.activeDeviceId);
  const torrentName = $derived(playbackState.activeTorrentName);
  const deviceName = $derived(playbackState.activeDeviceName);
  const torrentId = $derived(playbackState.activeTorrentId);
  const fileIndex = $derived(playbackState.activeFileIndex);
  const nextItems = $derived(queueState.nextItems);
  const hasQueue = $derived(!queueState.isEmpty);
  const hasActivePlayback = $derived(isPlaying || isPaused || torrentName !== null);
  const availableDevices = $derived(devicesState.devices);

  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  }

  async function togglePlayPause() {
    if (!deviceId) return;
    if (isPlaying) {
      await playbackPause(deviceId);
    } else {
      await playbackPlay(deviceId);
    }
  }

  async function stop() {
    playbackState.clear();
    if (deviceId) {
      await playbackStop(deviceId);
    }
  }

  async function skipNext() {
    const next = queueState.playNext();
    if (next && deviceId) {
      await playbackCastTorrent(deviceId, next.torrentId, next.fileIndex);
      playbackState.setContext(next.name, deviceName, next.torrentId, next.fileIndex);
    }
  }

  function onSeekStart() {
    seeking = true;
    seekValue = currentTime;
  }

  function onSeekInput(e: Event) {
    const target = e.target as HTMLInputElement;
    seekValue = parseFloat(target.value);
  }

  async function onSeekEnd() {
    if (deviceId) {
      await playbackSeek(deviceId, seekValue);
    }
    seeking = false;
  }

  async function switchDevice(newDeviceId: string) {
    devicePickerOpen = false;
    if (newDeviceId === deviceId) return;

    const device = availableDevices.find(d => d.id === newDeviceId);
    if (!device || torrentId === null || fileIndex === null) return;

    // Stop current playback and start on new device
    if (deviceId) {
      await playbackStop(deviceId);
    }
    await playbackCastTorrent(newDeviceId, torrentId, fileIndex);
    playbackState.setContext(torrentName, device.name, torrentId, fileIndex);
  }

  async function disconnect() {
    devicePickerOpen = false;
    if (deviceId) {
      await playbackStop(deviceId);
    }
    playbackState.clear();
    queueState.clearQueue();
  }

  const displayTime = $derived(seeking ? seekValue : currentTime);
  const progress = $derived(duration > 0 ? (displayTime / duration) * 100 : 0);
</script>

<svelte:window onclick={() => devicePickerOpen = false} />

<div class="playback-bar-enter flex h-14 shrink-0 items-center gap-3 border-t border-[var(--color-border)] bg-[var(--color-bg-secondary)] px-4">
  <!-- Left: Controls -->
  <div class="flex shrink-0 items-center gap-1">
    <button
      disabled
      class="rounded-full p-1.5 text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)] disabled:opacity-40"
      title="Previous"
    >
      <SkipBack class="h-4 w-4" />
    </button>
    <button
      onclick={togglePlayPause}
      disabled={!hasActivePlayback}
      class="rounded-full p-2 text-[var(--color-text)] transition-colors hover:bg-[var(--color-bg-tertiary)] disabled:opacity-40"
      title={isPlaying ? i18n.t("common.pause") : i18n.t("common.resume")}
    >
      {#if isPlaying}
        <Pause class="h-5 w-5" />
      {:else}
        <Play class="h-5 w-5" />
      {/if}
    </button>
    <button
      onclick={skipNext}
      disabled={!hasQueue}
      class="rounded-full p-1.5 text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)] disabled:opacity-40"
      title={i18n.t("playback.skipNext")}
    >
      <SkipForward class="h-4 w-4" />
    </button>
  </div>

  <!-- Center: File name + seek bar -->
  <div class="flex min-w-0 flex-1 flex-col gap-0.5">
    <div class="truncate text-sm font-medium text-[var(--color-text)]">
      {torrentName ?? i18n.t("playback.nowPlaying")}
    </div>
    <div class="flex items-center gap-2">
      <span class="w-8 text-right text-xs text-[var(--color-text-muted)]">
        {formatTime(displayTime)}
      </span>
      <input
        type="range"
        min="0"
        max={duration || 1}
        step="1"
        value={displayTime}
        onmousedown={onSeekStart}
        ontouchstart={onSeekStart}
        oninput={onSeekInput}
        onmouseup={onSeekEnd}
        ontouchend={onSeekEnd}
        disabled={duration === 0}
        class="h-1 flex-1 cursor-pointer appearance-none rounded-full bg-[var(--color-bg-tertiary)] disabled:cursor-not-allowed disabled:opacity-40 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-[var(--color-primary)]"
        style="background: linear-gradient(to right, var(--color-primary) {progress}%, var(--color-bg-tertiary) {progress}%);"
      />
      <span class="w-8 text-xs text-[var(--color-text-muted)]">
        {formatTime(duration)}
      </span>
    </div>
  </div>

  <!-- Right: Device selector + queue count -->
  <div class="flex shrink-0 items-center gap-3">
    <!-- Queue indicator -->
    {#if hasQueue}
      <div class="rounded-full bg-[var(--color-primary)]/10 px-2 py-0.5 text-xs font-medium text-[var(--color-primary)]">
        {nextItems.length} {i18n.t("playback.upNext").toLowerCase()}
      </div>
    {/if}

    <!-- Device selector -->
    <div class="relative">
      <button
        onclick={(e) => { e.stopPropagation(); devicePickerOpen = !devicePickerOpen; }}
        class="flex items-center gap-1.5 rounded-lg px-2 py-1 text-sm text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
      >
        <Cast class="h-4 w-4" />
        <span class="max-w-32 truncate">{deviceName ?? "Device"}</span>
        <ChevronDown class="h-3 w-3" />
      </button>

      {#if devicePickerOpen}
        <div
          class="absolute bottom-full right-0 mb-2 min-w-48 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] py-1 shadow-lg"
          onclick={(e) => e.stopPropagation()}
        >
          {#if availableDevices.length === 0}
            <div class="px-3 py-2 text-sm text-[var(--color-text-muted)]">
              {i18n.t("actions.noChromecastDevices")}
            </div>
          {:else}
            {#each availableDevices as device}
              <button
                onclick={() => switchDevice(device.id)}
                class="flex w-full items-center gap-2 px-3 py-2 text-left text-sm hover:bg-[var(--color-bg-secondary)] {device.id === deviceId ? 'text-[var(--color-primary)]' : 'text-[var(--color-text)]'}"
              >
                <Tv class="h-4 w-4 {device.id === deviceId ? 'text-[var(--color-primary)]' : 'text-[var(--color-text-muted)]'}" />
                <span>{device.name}</span>
              </button>
            {/each}
            <div class="my-1 border-t border-[var(--color-border)]"></div>
            <button
              onclick={disconnect}
              class="flex w-full items-center gap-2 px-3 py-2 text-left text-sm text-[var(--color-error)] hover:bg-[var(--color-error)]/10"
            >
              <X class="h-4 w-4" />
              <span>{i18n.t("playback.disconnect")}</span>
            </button>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</div>
