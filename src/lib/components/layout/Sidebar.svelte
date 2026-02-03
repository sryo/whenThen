<script lang="ts">
  import { Zap, Activity, Settings } from "lucide-svelte";
  import { uiState, type ViewName } from "$lib/state/ui.svelte";
  import { tasksState } from "$lib/state/tasks.svelte";

  const navItems: { view: ViewName; icon: typeof Zap; label: string }[] = [
    { view: "playlets", icon: Zap, label: "Playlets" },
    { view: "activity", icon: Activity, label: "Activity" },
    { view: "settings", icon: Settings, label: "Settings" },
  ];

  const activeIndex = $derived(
    navItems.findIndex((item) => item.view === uiState.activeView),
  );

  const activeTaskCount = $derived(tasksState.activeTasks.length);
</script>

<nav
  class="relative flex w-14 flex-col items-center gap-1 border-r border-[var(--color-border)] bg-[var(--color-bg-secondary)] pt-14 pb-3"
>
  <!-- Sliding highlight behind the active button -->
  <div
    class="absolute left-[7px] h-10 w-10 rounded-lg bg-[var(--color-primary)] transition-transform duration-200 ease-out"
    style="transform: translateY({activeIndex * 44}px)"
  ></div>

  {#each navItems as item, i}
    <button
      onclick={() => uiState.setView(item.view)}
      class="relative z-10 flex h-10 w-10 items-center justify-center rounded-lg transition-colors duration-150
        {uiState.activeView === item.view
          ? 'text-white'
          : 'text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]'}"
      title={item.label}
    >
      <item.icon class="h-5 w-5" />
      {#if item.view === "activity" && activeTaskCount > 0}
        <span class="absolute -top-0.5 -right-0.5 flex h-4 min-w-4 items-center justify-center rounded-full bg-[var(--color-primary)] px-1 text-[9px] font-bold text-white">
          {activeTaskCount}
        </span>
      {/if}
    </button>
  {/each}
</nav>
