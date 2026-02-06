<script lang="ts">
  import {
    Plus,
    Copy,
    Trash2,
    ToggleLeft,
    ToggleRight,
    X,
    Play,
    HelpCircle,
    GripVertical,
    Check,
  } from "lucide-svelte";
  import { playletsState } from "$lib/state/playlets.svelte";
  import ContextMenu from "$lib/components/common/ContextMenu.svelte";
  import { useContextMenu } from "$lib/utils";
  import type { ContextMenuEntry } from "$lib/types/ui";
  import type { Playlet } from "$lib/types/playlet";
  import { tasksState } from "$lib/state/tasks.svelte";
  import { uiState } from "$lib/state/ui.svelte";
  import { handleDroppedContent, handleDroppedFile } from "$lib/services/drag-drop";
  import { assignTorrentToPlaylet, beginManualDrop } from "$lib/services/playlet-assignment";
  import PlayletEditModal from "$lib/components/common/PlayletEditModal.svelte";
  import SourcesSection from "$lib/components/common/SourcesSection.svelte";
  import InterestsSection from "$lib/components/common/InterestsSection.svelte";
  import { settingsState } from "$lib/state/settings.svelte";
  import type { Action } from "$lib/types/playlet";
  import { getPlayletTemplates, createPlayletFromTemplate } from "$lib/services/playlet-templates";
  import type { PlayletTemplate } from "$lib/services/playlet-templates";
  import { triggerDetails, buildActionPhrases } from "$lib/utils/playlet-display";
  import { i18n } from "$lib/i18n/state.svelte";

  // Convert a template to a pseudo-Playlet so we can reuse the block builders
  function templateAsPlaylet(t: PlayletTemplate): Playlet {
    return {
      id: "",
      name: t.name,
      enabled: true,
      trigger: t.trigger,
      actions: t.actions.map((a) => ({ ...a, id: "" }) as Action),
      conditions: [],
      conditionLogic: "and",
      fileFilter: t.fileFilter,
      createdAt: "",
    };
  }

  function activeTaskCount(playletId: string): number {
    return tasksState.activeTasks.filter((t) => t.playletId === playletId).length;
  }

  let editingPlayletId = $state<string | null>(null);
  let draggingOverPlayletId = $state<string | null>(null);
  let newCardIds = $state<Set<string>>(new Set());
  let removingCardId = $state<string | null>(null);
  let showTemplatePicker = $state(false);
  let closingTemplatePicker = $state(false);
  let savedRecently = $state(false);
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;

  function showSaved() {
    savedRecently = true;
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => { savedRecently = false; }, 1500);
  }

  // Drag-to-reorder state
  let dragFromIndex = $state<number | null>(null);
  let dropTargetIndex = $state<number | null>(null);
  let cardsContainer: HTMLElement | undefined = $state();

  const ctx = useContextMenu<string>();

  function contextMenuItems(playletId: string): ContextMenuEntry[] {
    const p = playletsState.getById(playletId);
    if (!p) return [];
    return [
      {
        icon: p.enabled ? ToggleRight : ToggleLeft,
        label: p.enabled ? i18n.t("common.disable") : i18n.t("common.enable"),
        action: () => playletsState.updatePlaylet(playletId, { enabled: !p.enabled }),
      },
      {
        icon: Copy,
        label: i18n.t("common.duplicate"),
        action: () => {
          const dup = playletsState.duplicatePlaylet(playletId);
          if (dup) {
            newCardIds = new Set([...newCardIds, dup.id]);
            editingPlayletId = dup.id;
          }
        },
      },
      { type: "divider" as const, },
      {
        icon: Trash2,
        label: i18n.t("common.delete"),
        danger: true,
        hidden: playletsState.count <= 1,
        action: () => {
          removingCardId = playletId;
          setTimeout(() => {
            playletsState.removePlaylet(playletId);
            removingCardId = null;
          }, 200);
        },
      },
    ];
  }

  function openPlaylet(id: string) {
    editingPlayletId = id;
  }

  function handleNewPlaylet() {
    if (settingsState.settings.skip_template_picker) {
      createBlankPlaylet();
    } else {
      showTemplatePicker = true;
    }
  }

  function closeTemplatePicker() {
    closingTemplatePicker = true;
    setTimeout(() => {
      showTemplatePicker = false;
      closingTemplatePicker = false;
    }, 200);
  }

  function createFromTemplate(template: PlayletTemplate) {
    closeTemplatePicker();
    const playlet = createPlayletFromTemplate(template);
    playletsState.addPlayletFromData(playlet);
    newCardIds = new Set([...newCardIds, playlet.id]);
    editingPlayletId = playlet.id;
  }

  function createBlankPlaylet() {
    closeTemplatePicker();
    const playlet = playletsState.addPlaylet();
    newCardIds = new Set([...newCardIds, playlet.id]);
    editingPlayletId = playlet.id;
  }

  function closeModal(newPlayletId?: string) {
    // Show saved indicator if we're closing an edit (not deleting an empty playlet)
    const wasEditing = editingPlayletId;
    editingPlayletId = null;
    if (newPlayletId) {
      newCardIds = new Set([...newCardIds, newPlayletId]);
    }
    // Only show saved if the playlet still exists (wasn't removed due to being empty)
    if (wasEditing && playletsState.getById(wasEditing)) {
      showSaved();
    }
  }

  function handleDeletePlaylet(id: string) {
    editingPlayletId = null;
    removingCardId = id;
    setTimeout(() => {
      playletsState.removePlaylet(id);
      removingCardId = null;
    }, 200);
  }


  function handleCardDragOver(e: DragEvent, playletId: string) {
    e.preventDefault();
    e.stopPropagation();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "copy";
    draggingOverPlayletId = playletId;
  }

  function handleCardDragLeave(e: DragEvent) {
    e.preventDefault();
    draggingOverPlayletId = null;
  }

  function pipFromCard(el: EventTarget | null) {
    if (!(el instanceof HTMLElement)) return;
    const rect = el.getBoundingClientRect();
    uiState.triggerFlyingPip(rect.left + rect.width / 2, rect.top + rect.height / 2);
  }

  async function handleCardDrop(e: DragEvent, playletId: string) {
    e.preventDefault();
    e.stopPropagation();
    draggingOverPlayletId = null;
    const dropTarget = e.currentTarget;

    const text = e.dataTransfer?.getData("text/plain");
    if (text) {
      try {
        beginManualDrop();
        const response = await handleDroppedContent(text);
        if (response) {
          const ok = assignTorrentToPlaylet(playletId, response, true);
          if (ok) pipFromCard(dropTarget);
        }
      } catch (err: any) {
        uiState.addToast(`Could not add torrent: ${err?.message || String(err)}`, "error");
      }
      return;
    }
    const files = e.dataTransfer?.files;
    if (files) {
      for (const file of files) {
        try {
          beginManualDrop();
          let response;
          // Tauri 2 / WKWebView: File objects don't have a .path property
          if ("path" in file && typeof (file as any).path === "string") {
            response = await handleDroppedContent((file as any).path);
          } else {
            response = await handleDroppedFile(file);
          }
          if (response) {
            const ok = assignTorrentToPlaylet(playletId, response, true);
            if (ok) pipFromCard(dropTarget);
          }
        } catch (err: any) {
          uiState.addToast(`Could not add torrent: ${err?.message || String(err)}`, "error");
        }
      }
    }
  }

  // Drag-to-reorder handlers
  function handlePlayletDragStart(index: number, e: PointerEvent) {
    dragFromIndex = index;
    cardsContainer?.setPointerCapture(e.pointerId);
  }

  function handlePlayletDragMove(e: PointerEvent) {
    if (dragFromIndex === null || !cardsContainer) return;
    const cards = cardsContainer.querySelectorAll<HTMLElement>("[data-playlet-index]");
    let target: number | null = null;
    for (const card of cards) {
      const rect = card.getBoundingClientRect();
      const midY = rect.top + rect.height / 2;
      const idx = parseInt(card.dataset.playletIndex!, 10);
      if (e.clientY < midY) {
        target = idx;
        break;
      }
      target = idx + 1;
    }
    dropTargetIndex = target;
  }

  function handlePlayletDragEnd() {
    if (dragFromIndex !== null && dropTargetIndex !== null) {
      const to = dropTargetIndex > dragFromIndex ? dropTargetIndex - 1 : dropTargetIndex;
      if (to !== dragFromIndex) {
        playletsState.reorderPlaylet(dragFromIndex, to);
      }
    }
    dragFromIndex = null;
    dropTargetIndex = null;
  }
</script>

<div class="mx-auto max-w-2xl p-6">
  <!-- Sources section -->
  <SourcesSection />

  <!-- Connector line -->
  <div class="pointer-events-none flex justify-center">
    <div class="h-4 w-[4px] bg-[var(--color-border)]"></div>
  </div>

  <!-- Interests section -->
  <InterestsSection />

  <!-- Connector line -->
  <div class="pointer-events-none flex justify-center">
    <div class="h-4 w-[4px] bg-[var(--color-border)]"></div>
  </div>

  <!-- Do section -->
  <div class="space-y-3 rounded-xl border border-[var(--color-success)]/30 bg-[var(--color-success)]/5 p-4">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <h3 class="text-2xl font-black text-[var(--color-success)]">{i18n.t("playlets.do")}</h3>
        <Play class="h-4 w-4 text-[var(--color-success)]" />
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
        data-tooltip={i18n.t("playlets.doTooltip")}
        data-tooltip-left
      >
        <HelpCircle class="h-4 w-4" />
      </button>
    </div>

    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="space-y-3"
      bind:this={cardsContainer}
      onpointermove={handlePlayletDragMove}
      onpointerup={handlePlayletDragEnd}
      onpointercancel={handlePlayletDragEnd}
    >
      {#each playletsState.playlets as playlet, index (playlet.id)}
        {@const details = triggerDetails(playlet)}
        {@const phrases = buildActionPhrases(playlet)}
        <!-- Drop indicator above -->
        {#if dropTargetIndex === index && dragFromIndex !== null && dragFromIndex !== index && dragFromIndex !== index - 1}
          <div class="h-0.5 rounded-full bg-[var(--color-primary)]"></div>
        {/if}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          data-playlet-index={index}
          ondragover={(e) => handleCardDragOver(e, playlet.id)}
          ondragleave={handleCardDragLeave}
          ondrop={(e) => handleCardDrop(e, playlet.id)}
          oncontextmenu={(e) => ctx.open(e, playlet.id)}
          onanimationend={() => { newCardIds.delete(playlet.id); newCardIds = newCardIds; }}
          class="group flex flex-col overflow-hidden rounded-xl bg-[var(--color-success)]/10 text-left transition-all duration-150 {draggingOverPlayletId === playlet.id ? 'ring-2 ring-[var(--color-primary)] scale-[1.02] shadow-lg' : 'hover:bg-[var(--color-success)]/15'} {newCardIds.has(playlet.id) ? 'card-enter' : ''} {removingCardId === playlet.id ? 'card-exit' : ''} {!playlet.enabled ? 'opacity-50' : ''} {dragFromIndex === index ? 'opacity-40' : ''}"
        >
          <div class="flex items-center">
            <!-- Drag handle -->
            <button
              onpointerdown={(e) => handlePlayletDragStart(index, e)}
              class="flex shrink-0 cursor-grab items-center justify-center px-2 py-4 text-[var(--color-text-muted)] opacity-0 transition-opacity hover:text-[var(--color-text)] group-hover:opacity-100 {dragFromIndex !== null ? 'cursor-grabbing' : ''}"
            >
              <GripVertical class="h-4 w-4" />
            </button>
            <button
              onclick={() => openPlaylet(playlet.id)}
              ondragover={(e) => e.preventDefault()}
              class="flex flex-1 flex-col text-left"
            >
              <!-- Colored pipeline text -->
              <div class="flex items-center justify-between gap-2 pr-4 py-4">
                <div class="flex flex-wrap items-center gap-x-1.5 text-sm font-bold">
                  <!-- Trigger details -->
                  {#each details as part, i}
                    {#if i > 0}
                      <span class="text-[var(--color-text-muted)]">·</span>
                    {/if}
                    <span style="color: {part.color};">{part.text}</span>
                  {/each}
                  <!-- Arrow to actions -->
                  {#if phrases.length > 0}
                    <span class="text-[var(--color-text-muted)]">→</span>
                  {/if}
                  <!-- Action phrases -->
                  {#each phrases as phrase, i}
                    {#if i > 0}
                      <span class="text-[var(--color-text-muted)]">→</span>
                    {/if}
                    <span style="color: {phrase.color};">{phrase.text}</span>
                  {/each}
                </div>
                <span class="shrink-0 text-lg font-black {activeTaskCount(playlet.id) > 0 ? 'text-[var(--color-primary)]' : 'text-[var(--color-text-muted)]'}">
                  {activeTaskCount(playlet.id)}
                </span>
              </div>
            </button>
          </div>
        </div>
      {/each}
      <!-- Drop indicator at the end -->
      {#if dropTargetIndex === playletsState.playlets.length && dragFromIndex !== null && dragFromIndex !== playletsState.playlets.length - 1}
        <div class="h-0.5 rounded-full bg-[var(--color-primary)]"></div>
      {/if}
    </div>

    <button
      onclick={handleNewPlaylet}
      class="flex items-center gap-1 text-xs font-medium text-[var(--color-text-muted)] transition-colors hover:text-[var(--color-text)]"
    >
      <Plus class="h-3.5 w-3.5" />
      {i18n.t("playlets.newPlaylet")}
    </button>
  </div>

</div>

{#if ctx.state}
  <ContextMenu x={ctx.state.x} y={ctx.state.y} items={contextMenuItems(ctx.state.data)} onclose={ctx.close} />
{/if}

<!-- Template picker sidebar -->
{#if showTemplatePicker}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    role="dialog"
    aria-modal="true"
    aria-label={i18n.t("modal.chooseTemplate")}
    tabindex="-1"
    class="fixed inset-0 z-50 flex justify-end {closingTemplatePicker ? 'backdrop-fade-out' : 'backdrop-fade-in'}"
    onclick={(e) => { if (e.target === e.currentTarget) closeTemplatePicker(); }}
    onkeydown={(e) => { if (e.key === "Escape") closeTemplatePicker(); }}
  >
    <div
      class="{closingTemplatePicker ? 'sidebar-slide-out' : 'sidebar-slide-in'} flex h-full w-full sm:max-w-md flex-col border-l border-[var(--color-border)] bg-[var(--color-bg)] shadow-2xl"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="flex shrink-0 items-center justify-between border-b border-[var(--color-border)] px-5 py-4">
        <span class="text-lg font-bold text-[var(--color-text)]">{i18n.t("playlets.newPlayletTitle")}</span>
        <button
          onclick={closeTemplatePicker}
          class="ml-3 shrink-0 rounded-lg p-1.5 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
        >
          <X class="h-5 w-5" />
        </button>
      </div>

      <!-- Scrollable content -->
      <div class="min-h-0 flex-1 overflow-y-auto p-5">
        <p class="mb-4 text-xs text-[var(--color-text-muted)]">{i18n.t("playlets.playletsAutomate")}</p>
        <div class="space-y-2">
          {#each getPlayletTemplates() as template}
            {@const pseudo = templateAsPlaylet(template)}
            {@const tDetails = triggerDetails(pseudo)}
            {@const tPhrases = buildActionPhrases(pseudo)}
            <button
              onclick={() => createFromTemplate(template)}
              class="flex w-full flex-col rounded-xl bg-[var(--color-bg-secondary)] px-4 py-3 text-left transition-colors hover:bg-[var(--color-bg-tertiary)]"
            >
              <span class="text-sm font-bold text-[var(--color-text)]">{template.name}</span>
              <span class="text-xs text-[var(--color-text-muted)]">{template.description}</span>
              <!-- Colored pipeline text -->
              <div class="mt-2 flex flex-wrap items-center gap-x-1.5 text-sm font-bold">
                {#each tDetails as part, i}
                  {#if i > 0}
                    <span class="text-[var(--color-text-muted)]">·</span>
                  {/if}
                  <span style="color: {part.color};">{part.text}</span>
                {/each}
                {#if tPhrases.length > 0}
                  <span class="text-[var(--color-text-muted)]">→</span>
                {/if}
                {#each tPhrases as phrase, i}
                  {#if i > 0}
                    <span class="text-[var(--color-text-muted)]">→</span>
                  {/if}
                  <span style="color: {phrase.color};">{phrase.text}</span>
                {/each}
              </div>
            </button>
          {/each}
          <button
            onclick={createBlankPlaylet}
            class="flex w-full flex-col rounded-xl bg-[var(--color-bg-secondary)] px-4 py-3 text-left transition-colors hover:bg-[var(--color-bg-tertiary)]"
          >
            <span class="text-sm font-bold text-[var(--color-text)]">{i18n.t("playlets.blankPlaylet")}</span>
            <span class="text-xs text-[var(--color-text-muted)]">{i18n.t("playlets.blankDescription")}</span>
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <label
              class="mt-2 flex items-center gap-2 text-xs text-[var(--color-text-muted)]"
              onclick={(e) => e.stopPropagation()}
            >
              <input
                type="checkbox"
                checked={settingsState.settings.skip_template_picker}
                onchange={(e) => settingsState.updateAndSave({ skip_template_picker: (e.target as HTMLInputElement).checked })}
                class="rounded"
              />
              {i18n.t("playlets.alwaysCreateBlank")}
            </label>
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

{#if editingPlayletId}
  <PlayletEditModal playletId={editingPlayletId} onClose={closeModal} onDelete={handleDeletePlaylet} />
{/if}

<style>
  .card-enter {
    animation: cardIn 200ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
    transition: none;
  }

  .card-exit {
    animation: cardOut 200ms cubic-bezier(0.7, 0, 0.84, 0) forwards;
    transition: none;
    pointer-events: none;
  }

  @keyframes cardIn {
    from { opacity: 0; transform: scale(0.95); }
    to   { opacity: 1; transform: scale(1); }
  }

  @keyframes cardOut {
    from { opacity: 1; transform: scale(1); filter: blur(0); }
    to   { opacity: 0; transform: scale(0.95); filter: blur(4px); }
  }

  .backdrop-fade-in {
    animation: fadeIn 200ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  .backdrop-fade-out {
    animation: fadeOut 200ms cubic-bezier(0.7, 0, 0.84, 0) forwards;
  }

  .sidebar-slide-in {
    animation: slideIn 200ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  .sidebar-slide-out {
    animation: slideOut 200ms cubic-bezier(0.7, 0, 0.84, 0) forwards;
  }

  @keyframes fadeIn {
    from { background-color: transparent; }
    to   { background-color: rgb(0 0 0 / 0.3); }
  }

  @keyframes fadeOut {
    from { background-color: rgb(0 0 0 / 0.3); }
    to   { background-color: transparent; }
  }

  @keyframes slideIn {
    from { transform: translateX(100%); }
    to   { transform: translateX(0); }
  }

  @keyframes slideOut {
    from { transform: translateX(0); }
    to   { transform: translateX(100%); }
  }
</style>
