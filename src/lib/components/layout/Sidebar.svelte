<!-- Left navigation sidebar for main app views. -->
<script lang="ts">
  import { Inbox, Workflow, Settings } from "lucide-svelte";
  import { uiState } from "$lib/state/ui.svelte";
  import { feedsState } from "$lib/state/feeds.svelte";
  import { i18n } from "$lib/i18n/state.svelte";
  import type { ViewName } from "$lib/types/ui";

  const navItems: { view: ViewName; icon: typeof Inbox; labelKey: string }[] = [
    { view: "inbox", icon: Inbox, labelKey: "nav.inbox" },
    { view: "rules", icon: Workflow, labelKey: "nav.playlets" },
    { view: "settings", icon: Settings, labelKey: "nav.settings" },
  ];

  const pendingCount = $derived(feedsState.pendingCount);
  const activeIndex = $derived(navItems.findIndex((item) => item.view === uiState.activeView));
</script>

<nav class="flex h-full w-[78px] shrink-0 flex-col border-r border-[var(--color-border)] bg-[var(--color-bg-secondary)]">
  <!-- Traffic light spacer / drag region -->
  <div class="h-14 shrink-0" data-tauri-drag-region></div>
  <div class="relative flex flex-1 flex-col items-center gap-1">
    <!-- Sliding indicator -->
    <div
      class="nav-indicator absolute left-2 right-2 h-12 rounded-xl bg-[var(--color-primary)]"
      style="top: calc({activeIndex} * 52px);"
    ></div>

    {#each navItems as item, index}
      {@const isActive = uiState.activeView === item.view}
      <button
        onclick={() => uiState.setView(item.view)}
        class="group relative z-10 flex h-12 w-14 flex-col items-center justify-center rounded-xl transition-colors {isActive ? 'text-white' : 'text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]'}"
        title={i18n.t(item.labelKey)}
      >
        <item.icon class="h-5 w-5" />
        <span class="mt-0.5 text-[9px] font-medium">{i18n.t(item.labelKey)}</span>

        <!-- Badge for inbox pending count -->
        {#if item.view === "inbox" && pendingCount > 0 && !isActive}
          <span class="absolute -right-0.5 -top-0.5 flex h-4 min-w-4 items-center justify-center rounded-full bg-[var(--color-error)] px-1 text-[10px] font-bold text-white">
            {pendingCount > 99 ? "99+" : pendingCount}
          </span>
        {/if}
      </button>
    {/each}
  </div>
</nav>

<style>
  .nav-indicator {
    transition: top 200ms cubic-bezier(0.16, 1, 0.3, 1);
    pointer-events: none;
  }
</style>
