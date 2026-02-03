<script lang="ts">
  import {
    SkipBack,
    Play,
    Pause,
    SkipForward,
    Volume2,
    Unplug,
  } from "lucide-svelte";
  import { playbackState } from "$lib/state/playback.svelte";
  import {
    playbackPlay,
    playbackPause,
    playbackStop,
    playbackSeek,
    playbackSeekRelative,
    playbackSetVolume,
    chromecastDisconnect,
  } from "$lib/services/tauri-commands";
  import { formatDuration } from "$lib/utils";
  import { uiState } from "$lib/state/ui.svelte";

  let volume = $state(1);
  let isSeeking = $state(false);
  let seekValue = $state(0);

  async function handlePlayPause() {
    const deviceId = playbackState.activeDeviceId;
    if (!deviceId) return;
    try {
      if (playbackState.isPlaying) {
        await playbackPause(deviceId);
      } else {
        await playbackPlay(deviceId);
      }
    } catch (err: any) {
      uiState.addToast(`Could not play: ${err?.message || err}`, "error");
    }
  }

  async function handleDisconnect() {
    const deviceId = playbackState.activeDeviceId;
    if (!deviceId) return;
    try {
      await playbackStop(deviceId);
      await chromecastDisconnect(deviceId);
      playbackState.clear();
    } catch (err: any) {
      uiState.addToast(`Could not disconnect: ${err?.message || err}`, "error");
    }
  }

  async function handleRewind() {
    const deviceId = playbackState.activeDeviceId;
    if (!deviceId) return;
    try {
      await playbackSeekRelative(deviceId, -30);
    } catch (err: any) {
      uiState.addToast(`Could not seek: ${err?.message || err}`, "error");
    }
  }

  async function handleForward() {
    const deviceId = playbackState.activeDeviceId;
    if (!deviceId) return;
    try {
      await playbackSeekRelative(deviceId, 30);
    } catch (err: any) {
      uiState.addToast(`Could not seek: ${err?.message || err}`, "error");
    }
  }

  async function handleVolumeChange(e: Event) {
    const target = e.target as HTMLInputElement;
    volume = parseFloat(target.value);
    const deviceId = playbackState.activeDeviceId;
    if (!deviceId) return;
    try {
      await playbackSetVolume(deviceId, volume);
    } catch {
      // Silently handle volume errors
    }
  }

  function handleSeekStart(e: Event) {
    isSeeking = true;
    seekValue = parseFloat((e.target as HTMLInputElement).value);
  }

  function handleSeekInput(e: Event) {
    seekValue = parseFloat((e.target as HTMLInputElement).value);
  }

  async function handleSeekEnd() {
    isSeeking = false;
    const deviceId = playbackState.activeDeviceId;
    if (!deviceId) return;
    try {
      await playbackSeek(deviceId, seekValue);
    } catch (err: any) {
      uiState.addToast(`Could not seek: ${err?.message || err}`, "error");
    }
  }

  const displayTime = $derived(isSeeking ? seekValue : playbackState.currentTime);
  const seekPercent = $derived(
    playbackState.duration > 0 ? (displayTime / playbackState.duration) * 100 : 0,
  );
</script>

{#if !playbackState.isIdle}
  <div
    class="flex h-16 items-center gap-4 border-t border-[var(--color-border)] bg-[var(--color-bg-secondary)] px-4"
  >
    <!-- Left: media info -->
    <div class="flex min-w-0 flex-1 flex-col">
      <span class="truncate text-sm font-medium text-[var(--color-text)]">
        {playbackState.activeTorrentName ?? playbackState.status?.media_title ?? "Now playing"}
      </span>
      {#if playbackState.activeDeviceName}
        <span class="truncate text-xs text-[var(--color-text-muted)]">
          {playbackState.activeDeviceName}
        </span>
      {/if}
    </div>

    <!-- Center: transport + seek -->
    <div class="flex flex-col items-center gap-1">
      <div class="flex items-center gap-1.5">
        <button
          onclick={handleRewind}
          class="rounded-full p-1 text-[var(--color-text-secondary)] transition-colors hover:text-[var(--color-text)]"
          title="Rewind 30s"
        >
          <SkipBack class="h-3.5 w-3.5" />
        </button>

        <button
          onclick={handlePlayPause}
          class="flex h-8 w-8 items-center justify-center rounded-full bg-[var(--color-primary)] text-white transition-colors hover:bg-[var(--color-primary-hover)]"
          title={playbackState.isPlaying ? "Pause" : "Play"}
        >
          {#if playbackState.isPlaying}
            <Pause class="h-3.5 w-3.5" />
          {:else}
            <Play class="h-3.5 w-3.5 translate-x-0.5" />
          {/if}
        </button>

        <button
          onclick={handleForward}
          class="rounded-full p-1 text-[var(--color-text-secondary)] transition-colors hover:text-[var(--color-text)]"
          title="Skip 30s"
        >
          <SkipForward class="h-3.5 w-3.5" />
        </button>
      </div>
      <div class="flex w-48 items-center gap-1.5">
        <span class="w-8 text-right text-[10px] text-[var(--color-text-muted)]">
          {formatDuration(displayTime)}
        </span>
        <input
          type="range"
          min="0"
          max={playbackState.duration || 1}
          step="1"
          value={displayTime}
          onmousedown={handleSeekStart}
          oninput={handleSeekInput}
          onmouseup={handleSeekEnd}
          class="h-1 flex-1 cursor-pointer accent-[var(--color-primary)]"
        />
        <span class="w-8 text-[10px] text-[var(--color-text-muted)]">
          {formatDuration(playbackState.duration)}
        </span>
      </div>
    </div>

    <!-- Right: volume + disconnect -->
    <div class="flex flex-1 items-center justify-end gap-2">
      <Volume2 class="h-3.5 w-3.5 text-[var(--color-text-muted)]" />
      <input
        type="range"
        min="0"
        max="1"
        step="0.05"
        value={volume}
        oninput={handleVolumeChange}
        class="h-1 w-16 cursor-pointer accent-[var(--color-primary)]"
      />
      <button
        onclick={handleDisconnect}
        class="ml-1 rounded-md p-1.5 text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-error)]"
        title="Disconnect"
      >
        <Unplug class="h-4 w-4" />
      </button>
    </div>
  </div>
{/if}
