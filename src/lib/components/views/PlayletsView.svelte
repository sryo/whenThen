<script lang="ts">
  import type { Component } from "svelte";
  import type { IconType } from "$lib/types/ui";
  import {
    Plus,
    Link,
    Filter,
    Files,
    FileVideo,
    Music,
    FileCode,
    CircleCheck,
    FileSearch,
    ArrowUpDown,
    Captions,
    FolderSearch,
    Copy,
    Trash2,
    ToggleLeft,
    ToggleRight,
    X,
  } from "lucide-svelte";
  import { playletsState, derivePlayletName } from "$lib/state/playlets.svelte";
  import ContextMenu from "$lib/components/common/ContextMenu.svelte";
  import { useContextMenu } from "$lib/utils";
  import type { ContextMenuEntry } from "$lib/types/ui";
  import type { Playlet, TriggerType } from "$lib/types/playlet";
  import type { FileFilterCategory } from "$lib/types/playlet";
  import { tasksState } from "$lib/state/tasks.svelte";
  import { uiState } from "$lib/state/ui.svelte";
  import { handleDroppedContent, handleDroppedFile } from "$lib/services/drag-drop";
  import { assignTorrentToPlaylet, beginManualDrop } from "$lib/services/playlet-assignment";
  import PlayletEditModal from "$lib/components/common/PlayletEditModal.svelte";
  import ActionBlock from "$lib/components/common/ActionBlock.svelte";
  import { getActionDef, getActionLabel } from "$lib/services/action-registry";
  import { devicesState } from "$lib/state/devices.svelte";
  import { settingsState } from "$lib/state/settings.svelte";
  import type { Action } from "$lib/types/playlet";
  import { playletTemplates, createPlayletFromTemplate } from "$lib/services/playlet-templates";
  import type { PlayletTemplate } from "$lib/services/playlet-templates";
  import { flowConnector } from "$lib/utils/flow-connector";

  const filterCategoryIcons: Record<FileFilterCategory, typeof Files> = {
    all: Files,
    video: FileVideo,
    audio: Music,
    subtitle: Captions,
    custom: FileCode,
  };

  const triggerTypeIcons: Record<TriggerType, typeof Link> = {
    torrent_added: Link,
    download_complete: CircleCheck,
    metadata_received: FileSearch,
    seeding_ratio: ArrowUpDown,
    folder_watch: FolderSearch,
  };

  interface TriggerIcon {
    icon: IconType;
    color: string;
  }

  function buildTriggerIcons(playlet: Playlet): TriggerIcon[] {
    const triggerType = playlet.trigger?.type ?? "torrent_added";
    const icons: TriggerIcon[] = [
      { icon: triggerTypeIcons[triggerType] ?? Link, color: "var(--color-primary)" },
    ];
    if (playlet.conditions.length > 0) {
      icons.push({ icon: Filter, color: "var(--color-info)" });
    }
    if (playlet.fileFilter) {
      icons.push({ icon: filterCategoryIcons[playlet.fileFilter.category], color: "var(--color-accent)" });
    }
    return icons;
  }

  interface DetailPart {
    text: string;
    color: string;
  }

  function triggerDetails(playlet: Playlet): DetailPart[] {
    const parts: DetailPart[] = [];

    // Show trigger info if non-default
    const triggerType = playlet.trigger?.type ?? "torrent_added";
    if (triggerType === "download_complete") {
      parts.push({ text: "complete", color: "var(--color-primary)" });
    } else if (triggerType === "metadata_received") {
      parts.push({ text: "metadata", color: "var(--color-primary)" });
    } else if (triggerType === "seeding_ratio") {
      const ratio = playlet.trigger.seedingRatio ?? 2.0;
      parts.push({ text: `ratio ${ratio}`, color: "var(--color-primary)" });
    } else if (triggerType === "folder_watch") {
      if (playlet.trigger.watchFolder) {
        const segments = playlet.trigger.watchFolder.replace(/\/+$/, "").split("/");
        parts.push({ text: segments[segments.length - 1] || "folder", color: "var(--color-primary)" });
      } else {
        parts.push({ text: "folder", color: "var(--color-primary)" });
      }
    }

    const vals = playlet.conditions
      .filter((c) => {
        if (c.field === "name") return c.value.trim();
        return c.numericValue !== undefined;
      })
      .map((c) => {
        if (c.field === "name") return c.value.trim();
        if (c.field === "total_size") return `${c.numericValue}MB`;
        return `${c.numericValue} files`;
      });
    if (vals.length > 0) {
      const join = playlet.conditionLogic === "or" ? " | " : " & ";
      parts.push({ text: vals.join(join), color: "var(--color-info)" });
    }

    if (playlet.fileFilter) {
      let text = "";
      switch (playlet.fileFilter.category) {
        case "all": text = "any"; break;
        case "video": text = "video"; break;
        case "audio": text = "audio"; break;
        case "subtitle": text = "subtitle"; break;
        case "custom": {
          const exts = playlet.fileFilter.customExtensions;
          if (exts.length > 0) text = exts.join(", ");
          break;
        }
      }
      if (text) parts.push({ text, color: "var(--color-accent)" });
    }

    if (parts.length === 0) {
      parts.push({ text: "any", color: "var(--color-text-muted)" });
    }

    return parts;
  }

  interface ActionBlock {
    label: string;
    color: string;
    icon: IconType;
  }

  function buildActionBlocks(playlet: Playlet): ActionBlock[] {
    return playlet.actions.map((action) => {
      const def = getActionDef(action.type);
      const configured = getActionLabel(action);
      return {
        label: configured ?? def?.label ?? action.type,
        color: def?.color ?? "var(--color-text-muted)",
        icon: def?.icon ?? Plus,
      };
    });
  }

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

  const ctx = useContextMenu<string>();

  function contextMenuItems(playletId: string): ContextMenuEntry[] {
    const p = playletsState.getById(playletId);
    if (!p) return [];
    return [
      {
        icon: p.enabled ? ToggleRight : ToggleLeft,
        label: p.enabled ? "Disable" : "Enable",
        action: () => playletsState.updatePlaylet(playletId, { enabled: !p.enabled }),
      },
      {
        icon: Copy,
        label: "Duplicate",
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
        label: "Delete",
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
    editingPlayletId = null;
    if (newPlayletId) {
      newCardIds = new Set([...newCardIds, newPlayletId]);
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
</script>

<div class="mx-auto max-w-3xl p-6">
  <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
    {#each playletsState.playlets as playlet (playlet.id)}
      {@const triggerIcons = buildTriggerIcons(playlet)}
      {@const details = triggerDetails(playlet)}
      {@const actions = buildActionBlocks(playlet)}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        ondragover={(e) => handleCardDragOver(e, playlet.id)}
        ondragleave={handleCardDragLeave}
        ondrop={(e) => handleCardDrop(e, playlet.id)}
        oncontextmenu={(e) => ctx.open(e, playlet.id)}
        onanimationend={() => { newCardIds.delete(playlet.id); newCardIds = newCardIds; }}
        class="group flex flex-col overflow-hidden rounded-xl bg-[var(--color-bg-secondary)] text-left transition-all duration-150 {draggingOverPlayletId === playlet.id ? 'ring-2 ring-[var(--color-primary)] scale-[1.02] shadow-lg' : 'hover:bg-[var(--color-bg-tertiary)]'} {newCardIds.has(playlet.id) ? 'card-enter' : ''} {removingCardId === playlet.id ? 'card-exit' : ''} {!playlet.enabled ? 'opacity-50' : ''}"
      >
        <!-- pointer-events-none during drag so the outer div receives drop events -->
        <button
          onclick={() => openPlaylet(playlet.id)}
          ondragover={(e) => e.preventDefault()}
          class="flex flex-1 flex-col text-left"
        >
          <!-- Mini-blocks strip -->
          <div class="relative flex flex-wrap items-end gap-x-3 gap-y-4 px-3 py-3" use:flowConnector>
            <!-- Trigger block (When + Where + With combined) -->
            <div data-flow-block class="flex flex-col items-center gap-1 rounded-lg border border-[var(--color-primary)]/30 bg-[var(--color-primary)]/10 px-3 py-2">
              <div class="flex items-center gap-1.5">
                {#each triggerIcons as ti}
                  {@const Icon = ti.icon as Component}
                  <Icon style="color: {ti.color};" class="h-7 w-7 shrink-0" />
                {/each}
              </div>
              <div class="flex max-w-[10rem] items-center gap-1 truncate text-xs font-bold">
                {#each details as part, i}
                  {#if i > 0}
                    <span class="text-[var(--color-text-muted)]">·</span>
                  {/if}
                  <span style="color: {part.color};">{part.text}</span>
                {/each}
              </div>
            </div>

            <!-- Action blocks -->
            {#each actions as action}
              <div data-flow-block>
                <ActionBlock icon={action.icon} color={action.color} label={action.label} />
              </div>
            {/each}
          </div>

          <!-- Playlet name + task count -->
          <div class="flex items-start justify-between gap-2 px-4 py-4">
            <h3 class="text-base font-bold leading-snug text-[var(--color-text)]">
              {derivePlayletName(playlet)}
            </h3>
            <span class="shrink-0 text-lg font-black {activeTaskCount(playlet.id) > 0 ? 'text-[var(--color-primary)]' : 'text-[var(--color-text-muted)]'}">
              {activeTaskCount(playlet.id)}
            </span>
          </div>
        </button>

      </div>
    {/each}

    <!-- New playlet card -->
    <button
      onclick={handleNewPlaylet}
      class="flex flex-col overflow-hidden rounded-xl bg-[var(--color-bg-secondary)] text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)]"
    >
      <div class="px-3 py-3">
        <div class="flex flex-col items-center gap-1 px-3 py-2">
          <Plus class="h-7 w-7" />
          <span class="text-xs font-black">New playlet</span>
        </div>
      </div>
    </button>
  </div>

  <!-- Drop-target legend -->
  <p class="mt-4 text-center text-xs text-[var(--color-text-muted)]">
    Drop a torrent on a playlet to start
  </p>
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
    aria-label="Choose a template"
    tabindex="-1"
    class="fixed inset-0 z-50 flex justify-end {closingTemplatePicker ? 'backdrop-fade-out' : 'backdrop-fade-in'}"
    onclick={(e) => { if (e.target === e.currentTarget) closeTemplatePicker(); }}
    onkeydown={(e) => { if (e.key === "Escape") closeTemplatePicker(); }}
  >
    <div
      class="{closingTemplatePicker ? 'sidebar-slide-out' : 'sidebar-slide-in'} flex h-full w-full max-w-md flex-col border-l border-[var(--color-border)] bg-[var(--color-bg)] shadow-2xl"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="flex shrink-0 items-center justify-between border-b border-[var(--color-border)] px-5 py-4">
        <span class="text-lg font-bold text-[var(--color-text)]">New playlet</span>
        <button
          onclick={closeTemplatePicker}
          class="ml-3 shrink-0 rounded-lg p-1.5 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
        >
          <X class="h-5 w-5" />
        </button>
      </div>

      <!-- Scrollable content -->
      <div class="min-h-0 flex-1 overflow-y-auto p-5">
        <p class="mb-4 text-xs text-[var(--color-text-muted)]">Playlets are tiny apps that do things with your torrents.</p>
        <div class="space-y-2">
          {#each playletTemplates as template}
            {@const pseudo = templateAsPlaylet(template)}
            {@const tIcons = buildTriggerIcons(pseudo)}
            {@const tDetails = triggerDetails(pseudo)}
            {@const tActions = buildActionBlocks(pseudo)}
            <button
              onclick={() => createFromTemplate(template)}
              class="flex w-full flex-col rounded-xl bg-[var(--color-bg-secondary)] px-4 py-3 text-left transition-colors hover:bg-[var(--color-bg-tertiary)]"
            >
              <!-- Mini-blocks strip -->
              <div class="relative mb-2 flex flex-wrap items-end gap-x-2 gap-y-3" use:flowConnector>
                <div data-flow-block class="flex flex-col items-center gap-1 rounded-lg border border-[var(--color-primary)]/30 bg-[var(--color-primary)]/10 px-2 py-1.5">
                  <div class="flex items-center gap-1">
                    {#each tIcons as ti}
                      {@const Icon = ti.icon as Component}
                      <Icon style="color: {ti.color};" class="h-4 w-4 shrink-0" />
                    {/each}
                  </div>
                  <div class="flex items-center gap-1 text-[10px] font-bold">
                    {#each tDetails as part, i}
                      {#if i > 0}
                        <span class="text-[var(--color-text-muted)]">·</span>
                      {/if}
                      <span style="color: {part.color};">{part.text}</span>
                    {/each}
                  </div>
                </div>
                {#each tActions as act}
                  <div data-flow-block>
                    <ActionBlock icon={act.icon} color={act.color} label={act.label} size="sm" />
                  </div>
                {/each}
              </div>
              <span class="text-sm font-bold text-[var(--color-text)]">{template.name}</span>
              <span class="text-xs text-[var(--color-text-muted)]">{template.description}</span>
            </button>
          {/each}
          <button
            onclick={createBlankPlaylet}
            class="flex w-full flex-col rounded-xl bg-[var(--color-bg-secondary)] px-4 py-3 text-left transition-colors hover:bg-[var(--color-bg-tertiary)]"
          >
            <span class="text-sm font-bold text-[var(--color-text)]">Blank playlet</span>
            <span class="text-xs text-[var(--color-text-muted)]">Start from scratch</span>
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
              Always create blank
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
