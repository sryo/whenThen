<!-- Where section: RSS feed sources. -->
<script lang="ts">
  import { Plus, X, Rss, ToggleLeft, ToggleRight, HelpCircle, Check, Clock, ChevronDown, ChevronUp, AlertCircle } from "lucide-svelte";
  import { feedsState, type Source } from "$lib/state/feeds.svelte";
  import { i18n } from "$lib/i18n/state.svelte";

  let expandedId = $state<string | null>(null);
  let savedRecently = $state(false);
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;

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

  async function addSource() {
    try {
      const source = await feedsState.addSource({
        name: "",
        url: "",
        enabled: true,
      });
      editingId = source.id;
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
    {#each feedsState.sources as source, i (source.id)}
      {@const backoffMins = getBackoffMinutes(source.retryAfter)}
      <div class="rounded-lg bg-[var(--color-primary)]/10 {!source.enabled ? 'opacity-50' : ''}">
        <!-- Main row -->
        <div class="flex items-center gap-2 p-2">
          {#if i === 0}
            <span class="w-10 shrink-0 text-xs font-bold text-[var(--color-primary)]">{i18n.t("sources.feed")}</span>
          {:else}
            <span class="w-10 shrink-0 text-xs font-bold text-[var(--color-primary)]">{i18n.t("sources.and")}</span>
          {/if}

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

          <!-- Backoff indicator -->
          {#if backoffMins}
            <span class="flex items-center gap-1 text-xs text-[var(--color-warning)]" title={i18n.t("sources.inBackoff", { minutes: backoffMins })}>
              <AlertCircle class="h-3.5 w-3.5" />
              {backoffMins}m
            </span>
          {/if}

          <!-- Expand button -->
          <button
            onclick={() => toggleExpanded(source.id)}
            class="shrink-0 rounded p-1 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
          >
            {#if expandedId === source.id}
              <ChevronUp class="h-3.5 w-3.5" />
            {:else}
              <ChevronDown class="h-3.5 w-3.5" />
            {/if}
          </button>

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

        <!-- Expanded options -->
        {#if expandedId === source.id}
          <div class="border-t border-[var(--color-primary)]/20 px-3 py-2 space-y-2">
            <!-- Check interval -->
            <div class="flex items-center gap-2">
              <Clock class="h-3.5 w-3.5 shrink-0 text-[var(--color-text-muted)]" />
              <span class="text-xs text-[var(--color-text-muted)]">{i18n.t("sources.checkInterval")}</span>
              <input
                type="number"
                min="1"
                max="120"
                value={source.checkInterval ?? ""}
                oninput={(e) => {
                  const val = parseInt((e.target as HTMLInputElement).value);
                  updateSource(source.id, { checkInterval: val > 0 ? val : undefined });
                }}
                placeholder={i18n.t("sources.useGlobalInterval")}
                class="h-6 w-20 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-primary)]"
              />
              <span class="text-xs text-[var(--color-text-muted)]">min</span>
            </div>

            <!-- GUID dedup -->
            <div class="flex items-center gap-2">
              <span class="text-xs text-[var(--color-text-muted)] flex-1">{i18n.t("sources.useGuidDedup")}</span>
              <button
                onclick={() => updateSource(source.id, { useGuidDedup: !source.useGuidDedup })}
                class="shrink-0 text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
              >
                {#if source.useGuidDedup}
                  <ToggleRight class="h-4 w-4 text-[var(--color-success)]" />
                {:else}
                  <ToggleLeft class="h-4 w-4" />
                {/if}
              </button>
            </div>
          </div>
        {/if}
      </div>
    {/each}
  </div>

  <button
    onclick={addSource}
    class="mt-2 flex items-center gap-1.5 rounded-lg px-2 py-1 text-xs text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
  >
    <Plus class="h-3.5 w-3.5" />
    {i18n.t("sources.addSource")}
  </button>
</div>
