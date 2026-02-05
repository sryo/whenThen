<!-- Reusable action block for pipelines, cards, and pickers. -->
<script lang="ts">
  import type { Component } from "svelte";
  import type { IconType } from "$lib/types/ui";
  import { Check, Play, SkipForward, X } from "lucide-svelte";

  let {
    icon,
    color,
    label,
    value,
    size = "md",
    status,
  }: {
    icon: IconType;
    color: string;
    label: string;
    /** Configured value shown next to pip circles, below card blocks. */
    value?: string | null;
    size?: "sm" | "md" | "pip";
    status?: "done" | "running" | "pending" | "failed" | "skipped";
  } = $props();

  const Icon = $derived(icon as Component);
  const active = $derived(!status || status === "done" || status === "running");
  const skipped = $derived(status === "skipped");
</script>

{#if size === "pip"}
  <!-- Compact circle for CI-pipeline strips (TaskCard) -->
  <div
    style="--block-color: {color};"
    class="flex items-center gap-1.5"
    title={label}
  >
    <div class="relative">
      <div
        class="flex h-5 w-5 shrink-0 items-center justify-center rounded-full
          {!status ? 'pip-active' : ''}
          {status === 'done' ? 'pip-active' : ''}
          {status === 'running' ? 'pip-running' : ''}
          {status === 'pending' ? 'pip-pending' : ''}
          {status === 'failed' ? 'pip-failed' : ''}
          {status === 'skipped' ? 'pip-skipped' : ''}"
      >
        <Icon class="h-3 w-3 {active ? 'text-white' : skipped ? 'text-white' : 'text-[var(--color-text-muted)]'}" />
      </div>
      {#if status === "done"}
        <Check class="absolute -bottom-0.5 -right-0.5 h-2.5 w-2.5 rounded-full bg-[var(--color-success)] p-px text-white" />
      {:else if status === "running"}
        <Play class="absolute -bottom-0.5 -right-0.5 h-2.5 w-2.5 rounded-full bg-[var(--block-color)] p-px text-white" />
      {:else if status === "failed"}
        <X class="absolute -bottom-0.5 -right-0.5 h-2.5 w-2.5 rounded-full bg-[var(--color-error)] p-px text-white" />
      {:else if status === "skipped"}
        <SkipForward class="absolute -bottom-0.5 -right-0.5 h-2.5 w-2.5 rounded-full bg-[var(--color-warning)] p-px text-white" />
      {/if}
    </div>
    {#if value}
      <span class="max-w-20 truncate text-[10px] font-medium {status === 'failed' ? 'text-[var(--color-error)]' : skipped ? 'text-[var(--color-warning)]' : active ? 'text-[var(--color-text)]' : 'text-[var(--color-text-muted)]'}">
        {value}
      </span>
    {/if}
  </div>
{:else}
  <!-- Card / picker block (md, sm) -->
  <div
    style="--block-color: {color};"
    class="flex flex-col items-center border
      {size === 'sm' ? 'gap-0.5 rounded-md px-2 py-1.5' : 'gap-1 rounded-lg px-3 py-2'}
      {!status ? 'block-active' : ''}
      {status === 'done' ? 'block-active' : ''}
      {status === 'running' ? 'block-running' : ''}
      {status === 'pending' ? 'block-pending' : ''}
      {status === 'failed' ? 'block-failed' : ''}
      {status === 'skipped' ? 'block-skipped' : ''}"
  >
    {#if status === 'done'}
      <div class="relative">
        <Icon class="{size === 'sm' ? 'h-4 w-4' : 'h-7 w-7'} shrink-0 text-white" />
        <Check class="absolute -bottom-0.5 -right-0.5 h-2.5 w-2.5 rounded-full bg-white" style="color: var(--block-color);" />
      </div>
    {:else}
      <Icon class="{size === 'sm' ? 'h-4 w-4' : 'h-7 w-7'} shrink-0 {active ? 'text-white' : status === 'failed' ? 'text-[var(--color-error)]' : skipped ? 'text-[var(--color-warning)]' : 'text-[var(--color-text-muted)]'}" />
    {/if}
    <span class="{size === 'sm' ? 'text-[10px]' : 'text-xs'} font-black {active ? 'text-white' : status === 'failed' ? 'text-[var(--color-error)]' : skipped ? 'text-[var(--color-warning)]' : 'text-[var(--color-text-muted)]'}">
      {value ?? label}
    </span>
  </div>
{/if}

<style>
  /* Card / picker block styles */
  .block-active {
    background-color: var(--block-color);
    border-color: color-mix(in srgb, var(--block-color) 30%, transparent);
  }

  .block-running {
    background-color: var(--block-color);
    border-color: color-mix(in srgb, var(--block-color) 30%, transparent);
    outline: 2px solid var(--block-color);
    outline-offset: 2px;
  }

  .block-pending {
    background-color: var(--color-bg-tertiary);
    border-color: var(--color-border);
    opacity: 0.4;
  }

  .block-failed {
    background-color: color-mix(in srgb, var(--color-error) 20%, transparent);
    border-color: var(--color-error);
  }

  .block-skipped {
    background-color: color-mix(in srgb, var(--color-warning) 20%, transparent);
    border-color: var(--color-warning);
  }

  /* Pip (circle) styles */
  .pip-active {
    background-color: var(--block-color);
  }

  .pip-running {
    background-color: var(--block-color);
    outline: 2px solid var(--block-color);
    outline-offset: 1px;
  }

  .pip-pending {
    background-color: var(--color-bg-tertiary);
    opacity: 0.4;
  }

  .pip-failed {
    background-color: var(--color-error);
  }

  .pip-skipped {
    background-color: var(--color-warning);
  }

</style>
