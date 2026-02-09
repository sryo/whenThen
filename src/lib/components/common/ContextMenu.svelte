<script lang="ts">
  import type { Component } from "svelte";
  import type { ContextMenuEntry } from "$lib/types/ui";

  let {
    x,
    y,
    items,
    onclose,
  }: {
    x: number;
    y: number;
    items: ContextMenuEntry[];
    onclose: () => void;
  } = $props();

  let menuEl: HTMLElement | undefined = $state();
  let focusIndex = $state(-1);

  const visibleItems = $derived(
    items.filter((item) => item.type === "divider" || !item.hidden),
  );

  const actionableIndices = $derived(
    visibleItems
      .map((item, i) => ({ item, i }))
      .filter((e) => e.item.type !== "divider" && !e.item.disabled)
      .map((e) => e.i),
  );

  const clamped = $derived.by(() => {
    if (!menuEl) return { left: x, top: y };
    const rect = menuEl.getBoundingClientRect();
    let left = x;
    let top = y;
    if (left + rect.width > window.innerWidth) {
      left = window.innerWidth - rect.width - 4;
    }
    if (top + rect.height > window.innerHeight) {
      top = window.innerHeight - rect.height - 4;
    }
    return { left: Math.max(4, left), top: Math.max(4, top) };
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onclose();
      return;
    }
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      if (actionableIndices.length === 0) return;
      const currentPos = actionableIndices.indexOf(focusIndex);
      if (e.key === "ArrowDown") {
        focusIndex = actionableIndices[(currentPos + 1) % actionableIndices.length];
      } else {
        focusIndex = actionableIndices[(currentPos - 1 + actionableIndices.length) % actionableIndices.length];
      }
      return;
    }
    if (e.key === "Enter" && focusIndex >= 0) {
      const item = visibleItems[focusIndex];
      if (item && item.type !== "divider" && !item.disabled) {
        item.action();
        onclose();
      }
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="fixed inset-0 z-50"
  onclick={onclose}
  oncontextmenu={(e) => { e.preventDefault(); onclose(); }}
  onkeydown={handleKeydown}
  tabindex="-1"
>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    bind:this={menuEl}
    class="absolute flex flex-col overflow-hidden rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)]/80 py-1 shadow-xl backdrop-blur-xl"
    style="left: {clamped.left}px; top: {clamped.top}px;"
    onclick={(e) => e.stopPropagation()}
  >
    {#each visibleItems as item, i}
      {#if item.type === "divider"}
        <div class="mx-2 my-1 h-px bg-[var(--color-border)]"></div>
      {:else}
        <button
          onclick={() => { item.action(); onclose(); }}
          disabled={item.disabled}
          class="flex items-center gap-2 px-4 py-2 text-xs transition-colors
            {item.disabled ? 'opacity-30 cursor-not-allowed' : ''}
            {focusIndex === i ? 'bg-[var(--color-bg-tertiary)]' : ''}
            {item.danger
              ? 'text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-error)]'
              : 'text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]'}"
        >
          {#if item.icon}
            {@const Icon = item.icon as Component}
            <Icon class="h-4 w-4" />
          {/if}
          {item.label}
        </button>
      {/if}
    {/each}
  </div>
</div>
