<!-- Popover for casting torrents to Chromecast devices. Shows file picker for multi-file torrents. -->
<script lang="ts">
  import { Tv, Film, Loader2, ListVideo } from "lucide-svelte";
  import { devicesState } from "$lib/state/devices.svelte";
  import { playbackState } from "$lib/state/playback.svelte";
  import { queueState } from "$lib/state/queue.svelte";
  import { torrentFiles, playbackCastTorrent, chromecastConnect } from "$lib/services/tauri-commands";
  import type { TorrentFileInfo } from "$lib/types/torrent";
  import { i18n } from "$lib/i18n/state.svelte";
  import { onMount } from "svelte";

  let {
    torrentId,
    torrentName,
    x,
    y,
    onClose,
  }: {
    torrentId: number;
    torrentName: string;
    x: number;
    y: number;
    onClose: () => void;
  } = $props();

  let step = $state<"loading" | "files" | "devices">("loading");
  let selectedFileIndex = $state<number | null>(null);
  let playAll = $state(false);
  let playableFiles = $state<TorrentFileInfo[]>([]);
  let casting = $state(false);

  // Skip file picker for single-file torrents, go straight to device selection
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

  function selectFile(e: MouseEvent, index: number) {
    e.stopPropagation();
    selectedFileIndex = index;
    playAll = false;
    step = "devices";
  }

  function selectPlayAll(e: MouseEvent) {
    e.stopPropagation();
    playAll = true;
    selectedFileIndex = playableFiles[0]?.index ?? null;
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

      // Cast the first file
      await playbackCastTorrent(deviceId, torrentId, selectedFileIndex);

      // Set playback context
      const selectedFile = playableFiles.find(f => f.index === selectedFileIndex);
      playbackState.setContext(
        selectedFile?.name ?? torrentName,
        device?.name ?? deviceId,
        torrentId,
        selectedFileIndex
      );

      // If playing all, add remaining files to queue
      if (playAll && playableFiles.length > 1) {
        const remaining = playableFiles
          .filter(f => f.index !== selectedFileIndex)
          .map(f => ({ index: f.index, name: f.name }));
        queueState.addBatch(torrentId, remaining);
      }

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
  style="left: {x}px; top: {y}px; min-width: 240px; transform: translateX(-100%);"
>
  {#if step === "loading"}
    <div class="flex items-center justify-center gap-2 py-4">
      <Loader2 class="h-4 w-4 animate-spin text-[var(--color-text-muted)]" />
    </div>
  {:else if step === "files"}
    <div class="mb-2 text-sm font-medium text-[var(--color-text)]">
      {i18n.t("cast.selectFile")}
    </div>
    <div class="max-h-56 space-y-1 overflow-y-auto">
      <!-- Play All option -->
      <button
        onclick={selectPlayAll}
        class="flex w-full items-center gap-2 rounded-lg bg-[var(--color-primary)]/10 px-2 py-1.5 text-left text-sm text-[var(--color-primary)] transition-colors hover:bg-[var(--color-primary)]/20"
      >
        <ListVideo class="h-4 w-4 shrink-0" />
        <span class="flex-1">{i18n.t("playback.playAll")}</span>
        <span class="text-xs opacity-70">{playableFiles.length}</span>
      </button>
      <div class="my-1.5 border-t border-[var(--color-border)]"></div>
      {#each playableFiles as file}
        <button
          onclick={(e) => selectFile(e, file.index)}
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
      <div class="space-y-1">
        {#each devicesState.devices as device}
          <button
            onclick={() => castToDevice(device.id)}
            disabled={casting}
            class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left transition-colors hover:bg-[var(--color-bg-secondary)] disabled:opacity-50"
          >
            <Tv class="h-4 w-4 shrink-0 text-[var(--color-text-muted)]" />
            <div class="min-w-0 flex-1">
              <div class="text-sm font-medium text-[var(--color-text)]">{device.name}</div>
              {#if device.model}
                <div class="text-xs text-[var(--color-text-muted)]">{device.model}</div>
              {/if}
            </div>
          </button>
        {/each}
      </div>
    {/if}
  {/if}
</div>
