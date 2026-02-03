<script lang="ts">
  import { Tv, Wifi, WifiOff, Loader2 } from "lucide-svelte";
  import type { ChromecastDeviceInfo } from "$lib/types/device";
  import {
    chromecastConnect,
    chromecastDisconnect,
  } from "$lib/services/tauri-commands";
  import { uiState } from "$lib/state/ui.svelte";

  let { device }: { device: ChromecastDeviceInfo } = $props();

  let loading = $state(false);

  async function handleToggle() {
    loading = true;
    try {
      if (device.status === "connected") {
        await chromecastDisconnect(device.id);
      } else {
        await chromecastConnect(device.id);
      }
    } catch (err: any) {
      uiState.addToast(`Could not reach device: ${err?.message || err}`, "error");
    } finally {
      loading = false;
    }
  }
</script>

<div
  class="flex items-center justify-between rounded-lg px-3 py-2 transition-colors hover:bg-[var(--color-bg-tertiary)]"
>
  <div class="flex items-center gap-3 min-w-0">
    <Tv class="h-4 w-4 shrink-0 {device.status === 'connected' ? 'text-[var(--color-success)]' : 'text-[var(--color-text-muted)]'}" />
    <div class="min-w-0">
      <p class="truncate text-sm font-medium text-[var(--color-text)]">
        {device.name}
      </p>
      <p class="text-xs text-[var(--color-text-muted)]">{device.model}</p>
    </div>
  </div>
  <button
    onclick={handleToggle}
    disabled={loading || device.status === "connecting"}
    class="flex items-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-medium transition-colors
      {device.status === 'connected'
        ? 'bg-[var(--color-success)]/15 text-[var(--color-success)] hover:bg-[var(--color-success)]/25'
        : 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)] hover:bg-[var(--color-border)]'}
      disabled:opacity-50 disabled:cursor-not-allowed"
  >
    {#if loading || device.status === "connecting"}
      <Loader2 class="h-3 w-3 animate-spin" />
    {:else if device.status === "connected"}
      <Wifi class="h-3 w-3" />
      Connected
    {:else}
      <WifiOff class="h-3 w-3" />
      Connect
    {/if}
  </button>
</div>
