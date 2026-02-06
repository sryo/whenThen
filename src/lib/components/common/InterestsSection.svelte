<!-- What section: patterns to watch for. -->
<script lang="ts">
  import { Plus, X, ToggleLeft, ToggleRight, Search, HelpCircle, Check } from "lucide-svelte";
  import { feedsState, type Interest, type FeedFilter } from "$lib/state/feeds.svelte";
  import { i18n } from "$lib/i18n/state.svelte";

  const placeholders: Record<FeedFilter["type"], string> = {
    must_contain: "linux, 1080p, S01",
    must_not_contain: "CAM, HDTS, TELESYNC",
    regex: "S[0-9]{2}E[0-9]{2}",
    size_range: "100-5000",
  };

  let savedRecently = $state(false);
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;

  function showSaved() {
    savedRecently = true;
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => { savedRecently = false; }, 1500);
  }

  async function addInterest() {
    try {
      await feedsState.addInterest({
        name: "",
        enabled: true,
        filters: [{ type: "must_contain", value: "", enabled: true }],
      });
      showSaved();
    } catch (e) {
      console.error("Failed to add interest:", e);
    }
  }

  async function updateInterest(id: string, updates: Partial<Interest>) {
    try {
      await feedsState.updateInterest(id, updates);
      showSaved();
    } catch (e) {
      console.error("Failed to update interest:", e);
    }
  }

  async function removeInterest(id: string) {
    try {
      await feedsState.removeInterest(id);
      showSaved();
    } catch (e) {
      console.error("Failed to remove interest:", e);
    }
  }

  async function toggleInterest(e: Event, interest: Interest) {
    e.stopPropagation();
    try {
      await feedsState.toggleInterest(interest.id, !interest.enabled);
      showSaved();
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
      <h3 class="text-2xl font-black text-[var(--color-warning)]">{i18n.t("interests.what")}</h3>
      <Search class="h-4 w-4 text-[var(--color-warning)]" />
      {#if savedRecently}
        {#key Date.now()}
          <span class="flex items-center gap-1 text-xs text-[var(--color-success)] animate-fade-in-out">
            <Check class="h-3 w-3" />
            {i18n.t("common.saved")}
          </span>
        {/key}
      {/if}
    </div>
    <button
      class="text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
      data-tooltip={i18n.t("interests.whatTooltip")}
      data-tooltip-left
    >
      <HelpCircle class="h-4 w-4" />
    </button>
  </div>

  <div class="mt-3 space-y-3">
    {#each feedsState.interests as interest (interest.id)}
      <!-- Card container for each interest -->
      <div class="rounded-lg bg-[var(--color-warning)]/10 p-3 {!interest.enabled ? 'opacity-50' : ''}">
        <!-- Header row: name + toggle + remove -->
        <div class="flex items-center gap-2">
          <input
            type="text"
            value={interest.name}
            oninput={(e) => updateInterest(interest.id, { name: (e.target as HTMLInputElement).value })}
            placeholder={i18n.t("interests.interestPlaceholder")}
            class="h-7 flex-1 max-w-[240px] rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-warning)]"
          />
          <div class="flex-1"></div>
          <button
            onclick={(e) => toggleInterest(e, interest)}
            class="shrink-0 text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
            title={interest.enabled ? i18n.t("common.disable") : i18n.t("common.enable")}
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
            title={i18n.t("interests.removeInterest")}
          >
            <X class="h-3.5 w-3.5" />
          </button>
        </div>

        <!-- Filters section: indented with left border -->
        <div class="mt-2 pl-4 border-l-2 border-[var(--color-warning)]/30 space-y-2">
          {#each interest.filters as filter, i}
            <div class="flex items-center gap-2">
              {#if i > 0}
                <!-- Logic dropdown for subsequent filters -->
                <select
                  value={interest.filterLogic}
                  onchange={(e) => updateInterest(interest.id, { filterLogic: (e.target as HTMLSelectElement).value as "and" | "or" })}
                  class="w-14 h-7 shrink-0 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-1 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-warning)]"
                >
                  <option value="and">and</option>
                  <option value="or">or</option>
                </select>
              {/if}

              <select
                value={filter.type}
                onchange={(e) => updateFilter(interest, i, { type: (e.target as HTMLSelectElement).value as FeedFilter["type"] })}
                class="h-7 shrink-0 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-1.5 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-warning)]"
              >
                <option value="must_contain">{i18n.t("interests.contains")}</option>
                <option value="must_not_contain">{i18n.t("interests.excludes")}</option>
                <option value="regex">{i18n.t("interests.matches")}</option>
                <option value="size_range">{i18n.t("interests.sizeMb")}</option>
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

              <button
                onclick={() => removeFilter(interest, i)}
                class="shrink-0 rounded p-1 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-error)]"
                title={i18n.t("interests.removeRule")}
              >
                <X class="h-3.5 w-3.5" />
              </button>
            </div>
          {/each}

          <!-- Add rule button -->
          <button
            onclick={() => addFilter(interest)}
            class="flex items-center gap-1.5 rounded-lg px-2 py-1 text-xs text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
          >
            <Plus class="h-3.5 w-3.5" />
            {i18n.t("interests.addRule")}
          </button>
        </div>
      </div>
    {/each}
  </div>

  <button
    onclick={addInterest}
    class="mt-2 flex items-center gap-1.5 rounded-lg px-2 py-1 text-xs text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
  >
    <Plus class="h-3.5 w-3.5" />
    {i18n.t("interests.addInterest")}
  </button>
</div>
