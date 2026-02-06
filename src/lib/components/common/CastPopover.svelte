<!-- Popover for casting torrents to Chromecast devices. Shows file picker for multi-file torrents. -->
<script lang="ts">
  import { Tv, Film, Loader2 } from "lucide-svelte";
  import { devicesState } from "$lib/state/devices.svelte";
  import { torrentFiles, playbackCastTorrent, chromecastConnect } from "$lib/services/tauri-commands";
  import type { TorrentFileInfo } from "$lib/types/torrent";
  import { i18n } from "$lib/i18n/state.svelte";
  import { onMount } from "svelte";

  let {
    torrentId,
    x,
    y,
    onClose,
  }: {
    torrentId: number;
    x: number;
    y: number;
    onClose: () => void;
  } = $props();

  let step = $state<"loading" | "files" | "devices">("loading");
  let selectedFileIndex = $state<number | null>(null);
  let playableFiles = $state<TorrentFileInfo[]>([]);
  let casting = $state(false);

  onMount(async () => {
    try {
      const files = await torrentFiles(torrentId);
      playableFiles = files.filter((f) => f.is_playable);

      if (playableFiles.length === 0) {
        // No playable files, close popover
        onClose();
      } else if (playableFiles.length === 1) {
        // Single playable file, skip to device selection
        selectedFileIndex = playableFiles[0].index;
        step = "devices";
      } else {
        // Multiple playable files, show file picker
        step = "files";
      }
    } catch {
      onClose();
    }
  });

  function selectFile(index: number) {
    selectedFileIndex = index;
    step = "devices";
  }

  async function castToDevice(deviceId: string) {
    if (selectedFileIndex === null || casting) return;
    casting = true;
    try {
      // Connect to device first (if not already connected)
      const device = devicesState.devices.find((d) => d.id === deviceId);
      if (device && device.status !== "connected") {
        await chromecastConnect(deviceId);
        devicesState.updateDeviceStatus(deviceId, "connected");
      }
      await playbackCastTorrent(deviceId, torrentId, selectedFileIndex);
      onClose();
    } catch (e) {
      console.error("Failed to cast:", e);
      casting = false;
    }
  }

  function handleClickOutside(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest(".cast-popover")) {
      onClose();
    }
  }
</script>

<svelte:window onclick={handleClickOutside} />

<div
  class="cast-popover fixed z-50 rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-3 shadow-lg"
  style="left: {x}px; top: {y}px; min-width: 240px;"
>
  {#if step === "loading"}
    <div class="flex items-center justify-center gap-2 py-4">
      <Loader2 class="h-4 w-4 animate-spin text-[var(--color-text-muted)]" />
    </div>
  {:else if step === "files"}
    <div class="mb-2 text-sm font-medium text-[var(--color-text)]">
      {i18n.t("cast.selectFile")}
    </div>
    <div class="max-h-48 space-y-1 overflow-y-auto">
      {#each playableFiles as file}
        <button
          onclick={() => selectFile(file.index)}
          class="flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-left text-sm text-[var(--color-text)] transition-colors hover:bg-[var(--color-bg-secondary)]"
        >
          <Film class="h-4 w-4 shrink-0 text-[var(--color-text-muted)]" />
          <span class="min-w-0 flex-1 truncate">{file.name}</span>
        </button>
      {/each}
    </div>
  {:else}
    <div class="mb-2 text-sm font-medium text-[var(--color-text)]">
      {i18n.t("cast.selectDevice")}
    </div>
    {#if devicesState.devices.length === 0}
      <span class="text-sm text-[var(--color-text-muted)]">
        {i18n.t("actions.noChromecastDevices")}
      </span>
    {:else}
      <div class="grid grid-cols-2 gap-2">
        {#each devicesState.devices as device}
          <button
            onclick={() => castToDevice(device.id)}
            disabled={casting}
            class="flex flex-col items-start gap-0.5 truncate rounded-lg bg-[var(--color-bg-secondary)] px-3 py-2 text-left transition-colors hover:bg-[var(--color-bg-tertiary)] disabled:opacity-50"
          >
            <span class="flex items-center gap-2 text-sm font-medium text-[var(--color-text)]">
              <Tv class="h-4 w-4 shrink-0" />
              {device.name}
            </span>
            {#if device.model}
              <span class="ml-6 text-xs text-[var(--color-text-muted)]">{device.model}</span>
            {/if}
          </button>
        {/each}
      </div>
    {/if}
  {/if}
</div>
