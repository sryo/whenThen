<script lang="ts">
  import {
    ArrowDown,
    ArrowUp,
    Users,
    Play,
    Pause,
    SkipBack,
    SkipForward,
    Subtitles,
    Monitor,
    Unplug,
    Clipboard,
  } from "lucide-svelte";
  import { torrentsState } from "$lib/state/torrents.svelte";
  import { devicesState } from "$lib/state/devices.svelte";
  import { playbackState } from "$lib/state/playback.svelte";
  import { subtitlesState } from "$lib/state/subtitles.svelte";
  import {
    playbackPlay,
    playbackPause,
    playbackStop,
    playbackSeekRelative,
    chromecastDisconnect,
    subtitleLoadFile,
  } from "$lib/services/tauri-commands";
  import { open } from "@tauri-apps/plugin-dialog";
  import { formatSpeed, useContextMenu } from "$lib/utils";
  import { uiState } from "$lib/state/ui.svelte";
  import ContextMenu from "$lib/components/common/ContextMenu.svelte";
  import type { ContextMenuEntry } from "$lib/types/ui";

  let showDeviceMenu = $state(false);
  let showSubtitleMenu = $state(false);

  const titleCtx = useContextMenu();

  function titleMenuItems(): ContextMenuEntry[] {
    const title = playbackState.activeTorrentName ?? playbackState.status?.media_title ?? "Playing";
    return [
      {
        icon: Clipboard,
        label: "Copy Title",
        action: () => {
          navigator.clipboard.writeText(title);
          uiState.addToast("Copied to clipboard", "success");
        },
      },
      { type: "divider" },
      {
        icon: Unplug,
        label: "Disconnect",
        danger: true,
        action: handleDisconnect,
      },
    ];
  }

  async function handleLoadSubtitle() {
    showSubtitleMenu = false;
    try {
      const selected = await open({
        filters: [{ name: "Subtitles", extensions: ["srt", "vtt"] }],
        multiple: false,
      });
      if (selected) {
        const info = await subtitleLoadFile(selected as string);
        subtitlesState.setSubtitle(info);
      }
    } catch (err: any) {
      uiState.addToast(`Could not load subtitle: ${err?.message || err}`, "error");
    }
  }

  function handleClearSubtitle() {
    subtitlesState.clear();
    showSubtitleMenu = false;
  }

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

  async function handlePrev() {
    const deviceId = playbackState.activeDeviceId;
    if (!deviceId) return;
    try {
      await playbackSeekRelative(deviceId, -30);
    } catch (err: any) {
      uiState.addToast(`Could not seek: ${err?.message || err}`, "error");
    }
  }

  async function handleNext() {
    const deviceId = playbackState.activeDeviceId;
    if (!deviceId) return;
    try {
      await playbackSeekRelative(deviceId, 30);
    } catch (err: any) {
      uiState.addToast(`Could not seek: ${err?.message || err}`, "error");
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
    showDeviceMenu = false;
  }

  function toggleDeviceMenu() {
    showDeviceMenu = !showDeviceMenu;
  }

  const isActive = $derived(!playbackState.isIdle);

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      showDeviceMenu = false;
      showSubtitleMenu = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed top-[38px] left-0 right-0 z-40 flex h-[26px] items-center justify-between border-b border-[var(--color-border)] bg-[var(--color-bg-secondary)] px-3 text-xs text-[var(--color-text-muted)]"
>
  {#if isActive}
    <!-- Playing mode: player controls -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="flex items-center gap-1.5 overflow-hidden" oncontextmenu={(e) => titleCtx.open(e)}>
      <span class="truncate text-xs font-medium text-[var(--color-text)]">
        {playbackState.activeTorrentName ?? playbackState.status?.media_title ?? "Playing"}
      </span>
    </div>

    <!-- Center: transport controls -->
    <div class="flex items-center gap-0.5">
      <button
        onclick={handlePrev}
        class="rounded p-0.5 transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
        title="Rewind 30s"
      >
        <SkipBack class="h-3 w-3" />
      </button>
      <button
        onclick={handlePlayPause}
        class="flex h-5 w-5 items-center justify-center rounded-full bg-[var(--color-primary)] text-white transition-colors hover:bg-[var(--color-primary-hover)]"
        title={playbackState.isPlaying ? "Pause" : "Play"}
      >
        {#if playbackState.isPlaying}
          <Pause class="h-2.5 w-2.5" />
        {:else}
          <Play class="h-2.5 w-2.5 translate-x-[1px]" />
        {/if}
      </button>
      <button
        onclick={handleNext}
        class="rounded p-0.5 transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
        title="Skip 30s"
      >
        <SkipForward class="h-3 w-3" />
      </button>
    </div>

    <!-- Right: subtitle, device, disconnect -->
    <div class="relative flex items-center gap-1.5">
      <button
        onclick={() => (showSubtitleMenu = !showSubtitleMenu)}
        class="flex items-center gap-0.5 rounded px-1 py-0.5 transition-colors hover:bg-[var(--color-bg-tertiary)] {subtitlesState.hasSubtitle ? 'text-[var(--color-primary)]' : ''}"
        title={subtitlesState.hasSubtitle ? `Subtitle: ${subtitlesState.current?.name}` : "No subtitles"}
      >
        <Subtitles class="h-3 w-3" />
        {#if subtitlesState.hasSubtitle}
          <span class="max-w-[60px] truncate">{subtitlesState.current?.name}</span>
        {/if}
      </button>

      {#if showSubtitleMenu}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div class="fixed inset-0 z-50" onclick={() => (showSubtitleMenu = false)}></div>
        <div role="menu" class="absolute right-0 top-full z-50 mt-1 min-w-[140px] overflow-hidden rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] shadow-lg">
          {#if subtitlesState.hasSubtitle}
            <div class="flex items-center gap-2 px-3 py-2 text-xs text-[var(--color-text)]">
              <Subtitles class="h-3 w-3 text-[var(--color-primary)]" />
              <span class="truncate">{subtitlesState.current?.name}</span>
            </div>
            <button
              onclick={handleClearSubtitle}
              class="flex w-full items-center gap-2 border-t border-[var(--color-border)] px-3 py-2 text-xs text-[var(--color-error)] hover:bg-[var(--color-bg-tertiary)]"
            >
              Remove subtitle
            </button>
          {/if}
          <button
            onclick={handleLoadSubtitle}
            class="flex w-full items-center gap-2 border-t border-[var(--color-border)] px-3 py-2 text-xs text-[var(--color-text)] hover:bg-[var(--color-bg-tertiary)]"
          >
            Load from file...
          </button>
        </div>
      {/if}

      <button
        onclick={toggleDeviceMenu}
        class="flex items-center gap-0.5 rounded px-1 py-0.5 transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
        title={playbackState.activeDeviceName ?? "Device"}
      >
        <Monitor class="h-3 w-3" />
        <span class="max-w-[80px] truncate">{playbackState.activeDeviceName ?? "Device"}</span>
      </button>

      <button
        onclick={handleDisconnect}
        class="rounded p-0.5 transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-error)]"
        title="Disconnect"
      >
        <Unplug class="h-3 w-3" />
      </button>

      {#if showDeviceMenu}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
          class="fixed inset-0 z-50"
          onclick={() => (showDeviceMenu = false)}
        ></div>
        <div role="menu" class="absolute right-0 top-full z-50 mt-1 min-w-[160px] overflow-hidden rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] shadow-lg">
          {#if devicesState.connectedDevices.length > 0}
            {#each devicesState.connectedDevices as device (device.id)}
              <div class="flex items-center gap-2 px-3 py-2 text-xs text-[var(--color-text)]">
                <Monitor class="h-3 w-3 text-[var(--color-success)]" />
                <span class="truncate">{device.name}</span>
                {#if device.id === playbackState.activeDeviceId}
                  <span class="ml-auto text-[10px] text-[var(--color-primary)]">Active</span>
                {/if}
              </div>
            {/each}
          {:else}
            <div class="px-3 py-2 text-xs text-[var(--color-text-muted)]">No devices on your network</div>
          {/if}
          <button
            onclick={handleDisconnect}
            class="flex w-full items-center gap-2 border-t border-[var(--color-border)] px-3 py-2 text-xs text-[var(--color-error)] hover:bg-[var(--color-bg-tertiary)]"
          >
            <Unplug class="h-3 w-3" />
            Disconnect
          </button>
        </div>
      {/if}
    </div>
  {:else}
    <!-- Idle mode: torrent metrics -->
    <div class="flex items-center gap-3">
      <span class="flex items-center gap-1">
        <ArrowDown class="h-3 w-3" />
        {formatSpeed(torrentsState.totalDownloadSpeed)}
      </span>
      <span class="flex items-center gap-1">
        <ArrowUp class="h-3 w-3" />
        {formatSpeed(torrentsState.totalUploadSpeed)}
      </span>
      <span class="flex items-center gap-1">
        <Users class="h-3 w-3" />
        {torrentsState.totalPeers} connected
      </span>
    </div>
    <div class="flex items-center gap-2">
      {#if devicesState.hasConnectedDevice}
        <span class="flex items-center gap-1">
          <span class="h-1.5 w-1.5 rounded-full bg-[var(--color-success)]"></span>
          <Monitor class="h-3 w-3" />
          {devicesState.connectedDevices[0].name}
          {#if devicesState.connectedDevices.length > 1}
            <span>+{devicesState.connectedDevices.length - 1}</span>
          {/if}
        </span>
      {:else}
        <span class="flex items-center gap-1">
          <span class="h-1.5 w-1.5 rounded-full bg-[var(--color-text-muted)]"></span>
          No device
        </span>
      {/if}
    </div>
  {/if}
</div>

{#if titleCtx.state}
  <ContextMenu x={titleCtx.state.x} y={titleCtx.state.y} items={titleMenuItems()} onclose={titleCtx.close} />
{/if}
