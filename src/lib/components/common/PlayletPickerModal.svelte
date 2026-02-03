<script lang="ts">
  import type { Component } from "svelte";
  import type { IconType } from "$lib/types/ui";
  import { Link, Filter, FileVideo } from "lucide-svelte";
  import { uiState } from "$lib/state/ui.svelte";
  import { playletsState, derivePlayletName } from "$lib/state/playlets.svelte";
  import type { Playlet } from "$lib/types/playlet";
  import { assignTorrentToPlaylet } from "$lib/services/playlet-assignment";
  import { getActionDef } from "$lib/services/action-registry";
  import { onMount, onDestroy } from "svelte";

  interface CardSegment {
    color: string;
    icon: IconType;
  }

  function buildSegments(playlet: Playlet): CardSegment[] {
    const segments: CardSegment[] = [
      { color: "var(--color-primary)", icon: Link },
    ];
    if (playlet.conditions.length > 0) {
      segments.push({ color: "#60a5fa", icon: Filter });
    }
    if (playlet.fileFilter) {
      segments.push({ color: "#2dd4bf", icon: FileVideo });
    }
    for (const type of new Set(playlet.actions.map((a) => a.type))) {
      const def = getActionDef(type);
      if (def) segments.push({ color: def.color, icon: def.icon });
    }
    return segments;
  }

  // Clip-path chevrons: arrow point on right, notch on left
  function segmentStyle(color: string, index: number, total: number): string {
    const d = 8;
    let s = `background-color: ${color};`;
    if (total <= 1) return s;
    if (index > 0) s += ` margin-left: -${d}px;`;
    if (index === 0) {
      s += ` clip-path: polygon(0 0, calc(100% - ${d}px) 0, 100% 50%, calc(100% - ${d}px) 100%, 0 100%);`;
    } else if (index === total - 1) {
      s += ` clip-path: polygon(0 0, 100% 0, 100% 100%, 0 100%, ${d}px 50%);`;
    } else {
      s += ` clip-path: polygon(0 0, calc(100% - ${d}px) 0, 100% 50%, calc(100% - ${d}px) 100%, 0 100%, ${d}px 50%);`;
    }
    return s;
  }

  const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;
  const modLabel = isMac ? "⌘" : "Ctrl+";

  function selectPlaylet(playletId: string) {
    const pending = uiState.pendingTorrent;
    if (!pending) return;
    assignTorrentToPlaylet(playletId, pending);
    uiState.clearPlayletPicker();
  }

  function dismiss() {
    uiState.clearPlayletPicker();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      dismiss();
      return;
    }

    const mod = isMac ? e.metaKey : e.ctrlKey;
    if (!mod) return;

    const num = parseInt(e.key);
    if (num >= 1 && num <= 9) {
      e.preventDefault();
      const playlet = playletsState.playlets[num - 1];
      if (playlet) {
        selectPlaylet(playlet.id);
      }
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      dismiss();
    }
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeyDown, true);
  });

  onDestroy(() => {
    window.removeEventListener("keydown", handleKeyDown, true);
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  role="dialog"
  aria-modal="true"
  aria-label="Pick a playlet"
  tabindex="-1"
  class="fixed inset-0 z-50 flex items-center justify-center bg-[var(--color-bg)]/80 backdrop-blur-sm"
  onclick={handleBackdropClick}
>
  <div
    class="mx-4 w-full max-w-lg rounded-2xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-6 shadow-2xl"
  >
    <!-- Header: torrent name -->
    {#if uiState.pendingTorrent}
      <p class="mb-1 text-xs font-medium text-[var(--color-text-muted)]">Pick a playlet</p>
      <h2 class="mb-5 truncate text-sm font-bold text-[var(--color-text)]">
        {uiState.pendingTorrent.name}
      </h2>
    {/if}

    <!-- Playlet grid (only enabled playlets with torrent_added trigger) -->
    <div class="flex flex-wrap gap-3">
      {#each playletsState.playlets.filter((p) => p.enabled && p.trigger.type === "torrent_added") as playlet, i (playlet.id)}
        {@const pickerSegments = buildSegments(playlet)}
        <button
          onclick={() => selectPlaylet(playlet.id)}
          class="group flex w-28 flex-col items-center gap-2 overflow-hidden rounded-xl border-2 border-[var(--color-border)] transition-colors hover:brightness-95 dark:hover:brightness-110"
        >
          <!-- Fused icon strip with chevron interlocks -->
          <div class="flex h-10 w-full">
            {#each pickerSegments as segment, j}
              {@const Icon = segment.icon as Component}
              <div
                class="flex flex-1 items-center justify-center"
                style={segmentStyle(segment.color, j, pickerSegments.length)}
              >
                <Icon class="h-5 w-5 text-white" />
              </div>
            {/each}
          </div>
          <span class="w-full truncate px-2 text-center text-xs font-medium text-[var(--color-text)]">
            {derivePlayletName(playlet)}
          </span>
          {#if i < 9}
            <span class="mb-2 rounded bg-[var(--color-bg-tertiary)] px-1.5 py-0.5 text-[10px] font-mono text-[var(--color-text-muted)]">
              {modLabel}{i + 1}
            </span>
          {/if}
        </button>
      {/each}
    </div>

    <!-- Dismiss hint -->
    <p class="mt-4 text-center text-xs text-[var(--color-text-muted)]">
      Press <kbd class="rounded bg-[var(--color-bg-tertiary)] px-1 py-0.5 font-mono text-[10px]">Esc</kbd> to dismiss
    </p>
  </div>
</div>
