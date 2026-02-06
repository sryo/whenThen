<!-- Where section: RSS feed sources. -->
<script lang="ts">
  import { Plus, X, Rss, ToggleLeft, ToggleRight, HelpCircle } from "lucide-svelte";
  import { feedsState, type Source } from "$lib/state/feeds.svelte";

  let editingId = $state<string | null>(null);

  async function addSource() {
    try {
      const source = await feedsState.addSource({
        name: "",
        url: "",
        enabled: true,
        checkIntervalMinutes: 30,
      });
      editingId = source.id;
    } catch (e) {
      console.error("Failed to add source:", e);
    }
  }

  async function updateSource(id: string, updates: Partial<Source>) {
    try {
      await feedsState.updateSource(id, updates);
    } catch (e) {
      console.error("Failed to update source:", e);
    }
  }

  async function removeSource(id: string) {
    try {
      await feedsState.removeSource(id);
    } catch (e) {
      console.error("Failed to remove source:", e);
    }
  }

  async function toggleSource(e: Event, source: Source) {
    e.stopPropagation();
    try {
      await feedsState.toggleSource(source.id, !source.enabled);
    } catch (e) {
      console.error("Failed to toggle source:", e);
    }
  }
</script>

<div class="rounded-xl border border-[var(--color-primary)]/30 bg-[var(--color-primary)]/5 p-4">
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-3">
      <h3 class="text-2xl font-black text-[var(--color-primary)]">Where</h3>
      <Rss class="h-4 w-4 text-[var(--color-primary)]" />
    </div>
    <button
      class="text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
      data-tooltip="RSS feeds to monitor for new torrents."
      data-tooltip-left
    >
      <HelpCircle class="h-4 w-4" />
    </button>
  </div>

  <div class="mt-3 space-y-2">
    {#each feedsState.sources as source, i (source.id)}
      <div class="flex items-center gap-2">
        {#if i === 0}
          <span class="w-10 shrink-0 text-xs font-bold text-[var(--color-primary)]">Feed</span>
        {:else}
          <span class="w-10 shrink-0 text-xs font-bold text-[var(--color-primary)]">and</span>
        {/if}

        <input
          type="text"
          value={source.name}
          oninput={(e) => updateSource(source.id, { name: (e.target as HTMLInputElement).value })}
          placeholder="Name"
          class="h-7 w-20 shrink-0 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-primary)] {!source.enabled ? 'opacity-50' : ''}"
        />

        <input
          type="url"
          value={source.url}
          oninput={(e) => updateSource(source.id, { url: (e.target as HTMLInputElement).value })}
          placeholder="https://..."
          class="h-7 min-w-0 flex-1 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-primary)] {!source.enabled ? 'opacity-50' : ''}"
        />

        <button
          onclick={(e) => toggleSource(e, source)}
          class="shrink-0 text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
          title={source.enabled ? "Disable" : "Enable"}
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
          title="Remove"
        >
          <X class="h-3.5 w-3.5" />
        </button>
      </div>
    {/each}
  </div>

  <button
    onclick={addSource}
    class="mt-2 flex items-center gap-1.5 rounded-lg px-2 py-1 text-xs text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
  >
    <Plus class="h-3.5 w-3.5" />
    Add source
  </button>
</div>
