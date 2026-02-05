<!-- Static TitleBar variants for showcase. -->
<script lang="ts">
  import { Monitor, Activity, Zap, Settings } from "lucide-svelte";

  const navItems = [
    { view: "activity", icon: Activity, label: "Activity" },
    { view: "playlets", icon: Zap, label: "Playlets" },
    { view: "settings", icon: Settings, label: "Settings" },
  ];

  const STEP = 32;

  function indexFor(view: string): number {
    return navItems.findIndex((item) => item.view === view);
  }

  const variants = [
    { label: "Activity selected", activeView: "activity", badgeCount: 0 },
    { label: "Playlets selected", activeView: "playlets", badgeCount: 0 },
    { label: "Settings selected", activeView: "settings", badgeCount: 0 },
    { label: "Badge pulse", activeView: "activity", badgeCount: 3 },
  ];
</script>

<div class="space-y-3">
  {#each variants as variant}
    <div>
      <span class="mb-1 block text-xs font-medium text-[var(--color-text-muted)]">{variant.label}</span>
      <div class="overflow-hidden rounded-lg border border-[var(--color-border)]">
        <div
          class="flex h-[38px] items-center justify-between border-b border-[var(--color-border)] bg-[var(--color-bg-secondary)]"
        >
          <!-- Left: brand -->
          <div class="flex items-center gap-2 pl-4">
            <Monitor class="h-4 w-4 text-[var(--color-primary)]" />
            <span class="text-sm font-semibold text-[var(--color-text)]">whenThen</span>
          </div>

          <!-- Right: nav with sliding indicator -->
          <div class="relative flex items-center gap-1 pr-3">
            <div
              class="absolute h-7 w-7 rounded-md bg-[var(--color-primary)] transition-transform duration-200 ease-out"
              style="transform: translateX({indexFor(variant.activeView) * STEP}px)"
            ></div>

            {#each navItems as item}
              <div
                class="relative z-10 flex h-7 w-7 items-center justify-center rounded-md
                  {variant.activeView === item.view
                    ? 'text-white'
                    : 'text-[var(--color-text-muted)]'}"
              >
                <item.icon class="h-4 w-4" />
                {#if item.view === "activity" && variant.badgeCount > 0}
                  <span
                    class="absolute -top-1 -right-1 flex h-3.5 min-w-3.5 items-center justify-center rounded-full bg-[var(--color-primary)] px-0.5 text-[8px] font-bold text-white badge-pulse"
                  >
                    {variant.badgeCount}
                  </span>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      </div>
    </div>
  {/each}
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
