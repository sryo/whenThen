<!-- What section: patterns to watch for. -->
<script lang="ts">
  import { Plus, X, ToggleLeft, ToggleRight, Search, HelpCircle } from "lucide-svelte";
  import { feedsState, type Interest, type FeedFilter } from "$lib/state/feeds.svelte";

  const placeholders: Record<FeedFilter["type"], string> = {
    must_contain: "Severance, 1080p",
    must_not_contain: "CAM, 720p, HDTV",
    regex: "S\\d{2}E\\d{2}",
    size_range: "500-3000",
    episode: "S01, S01E01",
    resolution: "1080p, 2160p, 4K",
    source: "BluRay, WEB-DL, REMUX",
    codec: "x265, HEVC, x264",
    audio: "DTS, TrueHD, Atmos",
    hdr: "HDR, HDR10+, DV",
  };

  async function addInterest() {
    try {
      await feedsState.addInterest({
        name: "",
        enabled: true,
        filters: [{ type: "must_contain", value: "", enabled: true }],
      });
    } catch (e) {
      console.error("Failed to add interest:", e);
    }
  }

  async function updateInterest(id: string, updates: Partial<Interest>) {
    try {
      await feedsState.updateInterest(id, updates);
    } catch (e) {
      console.error("Failed to update interest:", e);
    }
  }

  async function removeInterest(id: string) {
    try {
      await feedsState.removeInterest(id);
    } catch (e) {
      console.error("Failed to remove interest:", e);
    }
  }

  async function toggleInterest(e: Event, interest: Interest) {
    e.stopPropagation();
    try {
      await feedsState.toggleInterest(interest.id, !interest.enabled);
    } catch (e) {
      console.error("Failed to toggle interest:", e);
    }
  }

  function updateFilter(interest: Interest, filterIndex: number, updates: Partial<FeedFilter>) {
    const newFilters = interest.filters.map((f, i) =>
      i === filterIndex ? { ...f, ...updates } : f
    );
    updateInterest(interest.id, { filters: newFilters });
  }

  function addFilter(interest: Interest) {
    const newFilters = [...interest.filters, { type: "must_contain" as const, value: "", enabled: true }];
    updateInterest(interest.id, { filters: newFilters });
  }

  function removeFilter(interest: Interest, filterIndex: number) {
    const newFilters = interest.filters.filter((_, i) => i !== filterIndex);
    updateInterest(interest.id, { filters: newFilters });
  }
</script>

<div class="rounded-xl border border-[var(--color-warning)]/30 bg-[var(--color-warning)]/5 p-4">
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-3">
      <h3 class="text-2xl font-black text-[var(--color-warning)]">What</h3>
      <Search class="h-4 w-4 text-[var(--color-warning)]" />
    </div>
    <button
      class="text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
      data-tooltip="Patterns to match against feed item titles."
      data-tooltip-left
    >
      <HelpCircle class="h-4 w-4" />
    </button>
  </div>

  <div class="mt-3 space-y-3">
    {#each feedsState.interests as interest (interest.id)}
      <div class="space-y-2 {!interest.enabled ? 'opacity-50' : ''}">
        {#each interest.filters as filter, i}
          <div class="flex items-center gap-2">
            {#if i === 0}
              <!-- First filter row: Interest label + name field -->
              <span class="w-14 shrink-0 text-xs font-bold text-[var(--color-warning)]">Interest</span>
              <input
                type="text"
                value={interest.name}
                oninput={(e) => updateInterest(interest.id, { name: (e.target as HTMLInputElement).value })}
                placeholder="name"
                class="h-7 w-24 shrink-0 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-warning)]"
              />
            {:else}
              <!-- Subsequent filter rows: and/or dropdown + spacer -->
              <select
                value={interest.filterLogic}
                onchange={(e) => updateInterest(interest.id, { filterLogic: (e.target as HTMLSelectElement).value as "and" | "or" })}
                class="w-14 h-7 shrink-0 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-1 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-warning)]"
              >
                <option value="and">and</option>
                <option value="or">or</option>
              </select>
              <div class="w-24 shrink-0"></div>
            {/if}

            <select
              value={filter.type}
              onchange={(e) => updateFilter(interest, i, { type: (e.target as HTMLSelectElement).value as FeedFilter["type"] })}
              class="h-7 shrink-0 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-1.5 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-warning)]"
            >
              <option value="must_contain">contains</option>
              <option value="must_not_contain">excludes</option>
              <option value="regex">matches</option>
              <option value="size_range">size (MB)</option>
              <option value="episode">episode</option>
              <option value="resolution">resolution</option>
              <option value="source">source</option>
              <option value="codec">codec</option>
              <option value="audio">audio</option>
              <option value="hdr">HDR</option>
            </select>

            <input
              type="text"
              value={filter.value}
              oninput={(e) => updateFilter(interest, i, { value: (e.target as HTMLInputElement).value })}
              placeholder={placeholders[filter.type]}
              autocapitalize="off"
              spellcheck={false}
              class="h-7 min-w-0 flex-1 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-warning)]"
            />

            {#if i === 0}
              <!-- First row: toggle + remove interest -->
              <button
                onclick={(e) => toggleInterest(e, interest)}
                class="shrink-0 text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
                title={interest.enabled ? "Disable" : "Enable"}
              >
                {#if interest.enabled}
                  <ToggleRight class="h-5 w-5 text-[var(--color-success)]" />
                {:else}
                  <ToggleLeft class="h-5 w-5" />
                {/if}
              </button>
              <button
                onclick={() => removeInterest(interest.id)}
                class="shrink-0 rounded p-1 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-error)]"
                title="Remove interest"
              >
                <X class="h-3.5 w-3.5" />
              </button>
            {:else}
              <!-- Subsequent rows: just remove filter -->
              <div class="w-5 shrink-0"></div>
              <button
                onclick={() => removeFilter(interest, i)}
                class="shrink-0 rounded p-1 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-error)]"
                title="Remove rule"
              >
                <X class="h-3.5 w-3.5" />
              </button>
            {/if}
          </div>
        {/each}

        <!-- Add rule button (indented) -->
        <button
          onclick={() => addFilter(interest)}
          class="ml-[calc(3.5rem+0.5rem)] flex items-center gap-1.5 rounded-lg px-2 py-1 text-xs text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
        >
          <Plus class="h-3.5 w-3.5" />
          Add rule
        </button>
      </div>

      <!-- Separator between interests -->
      {#if feedsState.interests.indexOf(interest) < feedsState.interests.length - 1}
        <div class="border-t border-[var(--color-warning)]/20 my-2"></div>
      {/if}
    {/each}
  </div>

  <button
    onclick={addInterest}
    class="mt-2 flex items-center gap-1.5 rounded-lg px-2 py-1 text-xs text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
  >
    <Plus class="h-3.5 w-3.5" />
    Add interest
  </button>
</div>
