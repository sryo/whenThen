<!-- Where section: RSS feeds and HTML scrapers. -->
<script lang="ts">
  import { Plus, X, Rss, Globe, ToggleLeft, ToggleRight, HelpCircle, Check, ChevronUp, ChevronDown, AlertCircle, Play, Loader2 } from "lucide-svelte";
  import { feedsState, type Source, type Scraper } from "$lib/state/feeds.svelte";
  import { i18n } from "$lib/i18n/state.svelte";

  let expandedId = $state<string | null>(null);
  let savedRecently = $state(false);
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;
  let testingId = $state<string | null>(null);
  let testResult = $state<{ id: string; count: number; error?: string } | null>(null);

  function toggleExpanded(id: string) {
    expandedId = expandedId === id ? null : id;
  }

  function getBackoffMinutes(retryAfter: string | undefined): number | null {
    if (!retryAfter) return null;
    const retryDate = new Date(retryAfter);
    const now = new Date();
    const diff = Math.max(0, Math.ceil((retryDate.getTime() - now.getTime()) / 60000));
    return diff > 0 ? diff : null;
  }

  function showSaved() {
    savedRecently = true;
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => { savedRecently = false; }, 1500);
  }

  // Source operations
  async function addSource() {
    try {
      await feedsState.addSource({
        name: "",
        url: "",
        enabled: true,
      });
      showSaved();
    } catch (e) {
      console.error("Failed to add source:", e);
    }
  }

  async function updateSource(id: string, updates: Partial<Source>) {
    try {
      await feedsState.updateSource(id, updates);
      showSaved();
    } catch (e) {
      console.error("Failed to update source:", e);
    }
  }

  async function removeSource(id: string) {
    try {
      await feedsState.removeSource(id);
      showSaved();
    } catch (e) {
      console.error("Failed to remove source:", e);
    }
  }

  async function toggleSource(e: Event, source: Source) {
    e.stopPropagation();
    try {
      await feedsState.toggleSource(source.id, !source.enabled);
      showSaved();
    } catch (e) {
      console.error("Failed to toggle source:", e);
    }
  }

  // Scraper operations
  async function addScraper() {
    try {
      const scraper = await feedsState.addScraper({
        name: "",
        baseUrl: "",
        itemSelector: "",
        titleSelector: "",
        linkSelector: "",
        enabled: true,
      });
      expandedId = scraper.id;
      showSaved();
    } catch (e) {
      console.error("Failed to add scraper:", e);
    }
  }

  async function updateScraper(id: string, updates: Partial<Scraper>) {
    try {
      await feedsState.updateScraper(id, updates);
      showSaved();
    } catch (e) {
      console.error("Failed to update scraper:", e);
    }
  }

  async function removeScraper(id: string) {
    try {
      await feedsState.removeScraper(id);
      showSaved();
    } catch (e) {
      console.error("Failed to remove scraper:", e);
    }
  }

  async function toggleScraper(e: Event, scraper: Scraper) {
    e.stopPropagation();
    try {
      await feedsState.toggleScraper(scraper.id, !scraper.enabled);
      showSaved();
    } catch (e) {
      console.error("Failed to toggle scraper:", e);
    }
  }

  async function testScraper(scraper: Scraper) {
    testingId = scraper.id;
    testResult = null;
    try {
      const result = await feedsState.testScraper(scraper);
      testResult = { id: scraper.id, count: result.totalCount };
    } catch (e: any) {
      testResult = { id: scraper.id, count: 0, error: e?.message || String(e) };
    } finally {
      testingId = null;
    }
  }
</script>

<div class="rounded-xl border border-[var(--color-primary)]/30 bg-[var(--color-primary)]/5 p-4">
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-3">
      <h3 class="text-2xl font-black text-[var(--color-primary)]">{i18n.t("sources.where")}</h3>
      <Rss class="h-4 w-4 text-[var(--color-primary)]" />
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
      data-tooltip={i18n.t("sources.whereTooltip")}
      data-tooltip-left
    >
      <HelpCircle class="h-4 w-4" />
    </button>
  </div>

  <div class="mt-3 space-y-2">
    <!-- RSS Sources -->
    {#each feedsState.sources as source, i (source.id)}
      {@const backoffMins = getBackoffMinutes(source.retryAfter)}
      <div class="flex items-center gap-2 rounded-lg bg-[var(--color-primary)]/10 p-2 {!source.enabled ? 'opacity-50' : ''}">
        <Rss class="h-3.5 w-3.5 shrink-0 text-[var(--color-text-muted)]" />

        <input
          type="text"
          value={source.name}
          oninput={(e) => updateSource(source.id, { name: (e.target as HTMLInputElement).value })}
          placeholder={i18n.t("sources.feedName")}
          class="h-7 w-20 shrink-0 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-primary)]"
        />

        <input
          type="url"
          value={source.url}
          oninput={(e) => updateSource(source.id, { url: (e.target as HTMLInputElement).value })}
          placeholder={i18n.t("sources.feedUrlPlaceholder")}
          class="h-7 min-w-0 flex-1 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-primary)]"
        />

        <!-- Check interval inline -->
        <input
          type="number"
          min="1"
          max="120"
          value={source.checkInterval ?? ""}
          oninput={(e) => {
            const val = parseInt((e.target as HTMLInputElement).value);
            updateSource(source.id, { checkInterval: val > 0 ? val : undefined });
          }}
          placeholder={i18n.t("sources.intervalPlaceholder")}
          class="h-7 w-16 shrink-0 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-primary)]"
        />

        <!-- Backoff indicator -->
        {#if backoffMins}
          <span class="flex items-center gap-1 text-xs text-[var(--color-warning)]" title={i18n.t("sources.inBackoff", { minutes: backoffMins })}>
            <AlertCircle class="h-3.5 w-3.5" />
            {backoffMins}m
          </span>
        {/if}

        <button
          onclick={(e) => toggleSource(e, source)}
          class="shrink-0 text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
          title={source.enabled ? i18n.t("common.disable") : i18n.t("common.enable")}
        >
          {#if source.enabled}
            <ToggleRight class="h-5 w-5 text-[var(--color-success)]" />
          {:else}
            <ToggleLeft class="h-5 w-5" />
          {/if}
        </button>

        <button
          onclick={() => removeSource(source.id)}
          class="shrink-0 rounded p-1 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-error)]"
          title={i18n.t("common.remove")}
        >
          <X class="h-3.5 w-3.5" />
        </button>
      </div>
    {/each}

    <!-- Scrapers -->
    {#each feedsState.scrapers as scraper, i (scraper.id)}
      <div class="rounded-lg bg-[var(--color-primary)]/10 {!scraper.enabled ? 'opacity-50' : ''}">
        <!-- Main row -->
        <div class="flex items-center gap-2 p-2">
          <Globe class="h-3.5 w-3.5 shrink-0 text-[var(--color-text-muted)]" />

          <input
            type="text"
            value={scraper.name}
            oninput={(e) => updateScraper(scraper.id, { name: (e.target as HTMLInputElement).value })}
            placeholder={i18n.t("scrapers.name")}
            class="h-7 w-20 shrink-0 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-primary)]"
          />

          <input
            type="url"
            value={scraper.baseUrl}
            oninput={(e) => updateScraper(scraper.id, { baseUrl: (e.target as HTMLInputElement).value })}
            placeholder={i18n.t("scrapers.baseUrl")}
            class="h-7 min-w-0 flex-1 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-primary)]"
          />

          <!-- Check interval -->
          <input
            type="number"
            min="1"
            max="120"
            value={scraper.checkInterval ?? ""}
            oninput={(e) => {
              const val = parseInt((e.target as HTMLInputElement).value);
              updateScraper(scraper.id, { checkInterval: val > 0 ? val : undefined });
            }}
            placeholder={i18n.t("sources.intervalPlaceholder")}
            class="h-7 w-16 shrink-0 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-primary)]"
          />

          <!-- Expand button -->
          <button
            onclick={() => toggleExpanded(scraper.id)}
            class="shrink-0 rounded p-1 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
          >
            {#if expandedId === scraper.id}
              <ChevronUp class="h-3.5 w-3.5" />
            {:else}
              <ChevronDown class="h-3.5 w-3.5" />
            {/if}
          </button>

          <button
            onclick={(e) => toggleScraper(e, scraper)}
            class="shrink-0 text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
            title={scraper.enabled ? i18n.t("common.disable") : i18n.t("common.enable")}
          >
            {#if scraper.enabled}
              <ToggleRight class="h-5 w-5 text-[var(--color-success)]" />
            {:else}
              <ToggleLeft class="h-5 w-5" />
            {/if}
          </button>

          <button
            onclick={() => removeScraper(scraper.id)}
            class="shrink-0 rounded p-1 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-error)]"
            title={i18n.t("common.remove")}
          >
            <X class="h-3.5 w-3.5" />
          </button>
        </div>

        <!-- Expanded options (CSS selectors) -->
        {#if expandedId === scraper.id}
          <div class="border-t border-[var(--color-primary)]/20 px-3 py-2 space-y-2">
            <!-- Item selector -->
            <div class="flex items-center gap-2">
              <span class="w-20 shrink-0 text-xs text-[var(--color-text-muted)]">{i18n.t("scrapers.itemSelector")}</span>
              <input
                type="text"
                value={scraper.itemSelector}
                oninput={(e) => updateScraper(scraper.id, { itemSelector: (e.target as HTMLInputElement).value })}
                placeholder=".torrent-row"
                class="h-6 min-w-0 flex-1 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-primary)]"
              />
            </div>

            <!-- Title selector -->
            <div class="flex items-center gap-2">
              <span class="w-20 shrink-0 text-xs text-[var(--color-text-muted)]">{i18n.t("scrapers.titleSelector")}</span>
              <input
                type="text"
                value={scraper.titleSelector}
                oninput={(e) => updateScraper(scraper.id, { titleSelector: (e.target as HTMLInputElement).value })}
                placeholder=".title a"
                class="h-6 min-w-0 flex-1 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-primary)]"
              />
            </div>

            <!-- Link selector -->
            <div class="flex items-center gap-2">
              <span class="w-20 shrink-0 text-xs text-[var(--color-text-muted)]">{i18n.t("scrapers.linkSelector")}</span>
              <input
                type="text"
                value={scraper.linkSelector}
                oninput={(e) => updateScraper(scraper.id, { linkSelector: (e.target as HTMLInputElement).value })}
                placeholder="a[href^='magnet:']"
                class="h-6 min-w-0 flex-1 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-primary)]"
              />
            </div>

            <!-- Size selector (optional) -->
            <div class="flex items-center gap-2">
              <span class="w-20 shrink-0 text-xs text-[var(--color-text-muted)]">{i18n.t("scrapers.sizeSelector")}</span>
              <input
                type="text"
                value={scraper.sizeSelector ?? ""}
                oninput={(e) => {
                  const val = (e.target as HTMLInputElement).value;
                  updateScraper(scraper.id, { sizeSelector: val || undefined });
                }}
                placeholder=".size"
                class="h-6 min-w-0 flex-1 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-primary)]"
              />
            </div>

            <!-- Search URL template (optional) -->
            <div class="flex items-center gap-2">
              <span class="w-20 shrink-0 text-xs text-[var(--color-text-muted)]">{i18n.t("scrapers.searchUrl")}</span>
              <input
                type="text"
                value={scraper.searchUrlTemplate ?? ""}
                oninput={(e) => {
                  const val = (e.target as HTMLInputElement).value;
                  updateScraper(scraper.id, { searchUrlTemplate: val || undefined });
                }}
                placeholder="/search?q=..."
                class="h-6 min-w-0 flex-1 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-primary)]"
              />
            </div>

            <!-- Test button and result -->
            <div class="flex items-center gap-2 pt-1">
              <button
                onclick={() => testScraper(scraper)}
                disabled={testingId === scraper.id || !scraper.baseUrl || !scraper.itemSelector}
                class="flex items-center gap-1.5 rounded px-2 py-1 text-xs font-medium text-[var(--color-primary)] transition-colors hover:bg-[var(--color-primary)]/20 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {#if testingId === scraper.id}
                  <Loader2 class="h-3 w-3 animate-spin" />
                {:else}
                  <Play class="h-3 w-3" />
                {/if}
                {i18n.t("scrapers.test")}
              </button>
              {#if testResult?.id === scraper.id}
                {#if testResult.error}
                  <span class="text-xs text-[var(--color-error)]">{testResult.error}</span>
                {:else if testResult.count > 0}
                  <span class="text-xs text-[var(--color-success)]">{i18n.t("scrapers.testSuccess", { count: testResult.count })}</span>
                {:else}
                  <span class="text-xs text-[var(--color-warning)]">{i18n.t("scrapers.testFailed")}</span>
                {/if}
              {/if}
            </div>
          </div>
        {/if}
      </div>
    {/each}
  </div>

  <div class="mt-2 flex items-center gap-2">
    <button
      onclick={addSource}
      class="flex items-center gap-1.5 rounded-lg px-2 py-1 text-xs text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
    >
      <Plus class="h-3.5 w-3.5" />
      {i18n.t("sources.rss")}
    </button>
    <button
      onclick={addScraper}
      class="flex items-center gap-1.5 rounded-lg px-2 py-1 text-xs text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
    >
      <Plus class="h-3.5 w-3.5" />
      {i18n.t("sources.scraper")}
    </button>
  </div>
</div>
