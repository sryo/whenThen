<script lang="ts">
  import { Monitor, Zap, Activity, Settings } from "lucide-svelte";
  import { uiState, type ViewName } from "$lib/state/ui.svelte";
  import { tasksState } from "$lib/state/tasks.svelte";

  const navItems: { view: ViewName; icon: typeof Zap; label: string }[] = [
    { view: "playlets", icon: Zap, label: "Playlets" },
    { view: "activity", icon: Activity, label: "Activity" },
    { view: "settings", icon: Settings, label: "Settings" },
  ];

  // 28px button + 4px gap
  const STEP = 32;

  const activeIndex = $derived(
    navItems.findIndex((item) => item.view === uiState.activeView),
  );

  const activeTaskCount = $derived(tasksState.activeTasks.length);

  let prevTaskCount = $state(0);
  let badgePulse = $state(false);

  $effect(() => {
    if (activeTaskCount > prevTaskCount && prevTaskCount >= 0) {
      badgePulse = true;
    }
    prevTaskCount = activeTaskCount;
  });

  function onBadgeAnimEnd() {
    badgePulse = false;
  }
</script>

<div
  data-tauri-drag-region
  class="fixed top-0 left-0 right-0 z-50 flex h-[38px] items-center justify-between border-b border-[var(--color-border)] bg-[var(--color-bg-secondary)]"
>
  <!-- Left: traffic lights padding + brand -->
  <div class="flex items-center gap-2 pl-[78px]" data-tauri-drag-region>
    <Monitor class="h-4 w-4 text-[var(--color-primary)]" />
    <span class="text-sm font-semibold text-[var(--color-text)]" data-tauri-drag-region>
      whenThen
    </span>
  </div>

  <!-- Right: nav with sliding indicator -->
  <div class="relative flex items-center gap-1 pr-3">
    <!-- Sliding background pill -->
    <div
      class="absolute h-7 w-7 rounded-md bg-[var(--color-primary)] transition-transform duration-200 ease-out"
      style="transform: translateX({activeIndex * STEP}px)"
    ></div>

    {#each navItems as item}
      <button
        onclick={() => uiState.setView(item.view)}
        data-nav={item.view}
        class="relative z-10 flex h-7 w-7 items-center justify-center rounded-md transition-colors duration-150
          {uiState.activeView === item.view
            ? 'text-white'
            : 'text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]'}"
        title={item.label}
      >
        <item.icon class="h-4 w-4" />
        {#if item.view === "activity" && activeTaskCount > 0}
          <span
            class="absolute -top-1 -right-1 flex h-3.5 min-w-3.5 items-center justify-center rounded-full bg-[var(--color-primary)] px-0.5 text-[8px] font-bold text-white {badgePulse ? 'badge-pulse' : ''}"
            onanimationend={onBadgeAnimEnd}
          >
            {activeTaskCount}
          </span>
        {/if}
      </button>
    {/each}
  </div>
</div>

<style>
  .badge-pulse {
    animation: badgePulse 300ms ease-out;
  }

  @keyframes badgePulse {
    0%   { transform: scale(1); }
    50%  { transform: scale(1.5); }
    100% { transform: scale(1); }
  }
</style>
