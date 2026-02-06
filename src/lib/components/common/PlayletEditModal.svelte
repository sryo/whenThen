<script lang="ts">
  import type { Component } from "svelte";
  import {
    Plus,
    Copy,
    Trash2,
    X,
    Link,
    FileVideo,
    Files,
    Music,
    FileCode,
    CircleCheck,
    FileSearch,
    ArrowUpDown,
    Captions,
    FolderSearch,
  } from "lucide-svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import ActionRow from "./ActionRow.svelte";
  import { playletsState, derivePlayletName } from "$lib/state/playlets.svelte";
  import type {
    ActionType,
    FileFilterCategory,
    TriggerType,
  } from "$lib/types/playlet";
  import { getActionDef } from "$lib/services/action-registry";
  import { i18n } from "$lib/i18n/state.svelte";

  const thenGridItems: { type: string; label?: string }[] = [
    { type: "cast" },
    { type: "move" },
    { type: "notify" },
    { type: "play" },
    { type: "subtitle" },
    { type: "automation" },
    { type: "delay" },
  ];

  // Types that allow only one instance. "move" covers the Files group (move/rename).
  const SINGLETON_TYPES: Record<string, string[]> = {
    move: ["move"],
    notify: ["notify"],
    play: ["play"],
    subtitle: ["subtitle"],
  };

  function isActionTaken(gridType: string): boolean {
    if (!playlet) return false;
    const check = SINGLETON_TYPES[gridType];
    if (!check) return false;
    return playlet.actions.some((a) => check.includes(a.type));
  }

  const filterCategoryKeys: Record<FileFilterCategory, string> = {
    all: "playlets.filterAll",
    video: "playlets.filterVideo",
    audio: "playlets.filterAudio",
    subtitle: "playlets.filterSubtitle",
    custom: "playlets.filterCustom",
  };

  const filterCategoryIcons: Record<FileFilterCategory, typeof Files> = {
    all: Files,
    video: FileVideo,
    audio: Music,
    subtitle: Captions,
    custom: FileCode,
  };

  const triggerTypeKeys: Record<TriggerType, string> = {
    torrent_added: "playlets.triggerTorrentAdded",
    download_complete: "playlets.triggerDownloadComplete",
    metadata_received: "playlets.triggerMetadataReceived",
    seeding_ratio: "playlets.triggerSeedingRatio",
  };

  const triggerTypeIcons: Record<TriggerType, typeof Link> = {
    torrent_added: Link,
    download_complete: CircleCheck,
    metadata_received: FileSearch,
    seeding_ratio: ArrowUpDown,
  };

  let {
    playletId,
    onClose,
    onDelete,
  }: {
    playletId: string;
    onClose: (newPlayletId?: string) => void;
    onDelete: (playletId: string) => void;
  } = $props();

  let customExtInput = $state("");
  let closing = $state(false);

  let pendingNewPlayletId: string | undefined;

  function requestClose(newPlayletId?: string) {
    if (playlet && playlet.actions.length === 0) {
      playletsState.removePlaylet(playlet.id);
    }
    pendingNewPlayletId = newPlayletId;
    closing = true;
    setTimeout(() => onClose(pendingNewPlayletId), 200);
  }

  const playlet = $derived(playletsState.getById(playletId));
  const activeFileCategory = $derived(playlet?.fileFilter?.category ?? "all");
  const derivedName = $derived(playlet ? derivePlayletName(playlet) : "");
  const activeTriggerType = $derived(playlet?.trigger?.type ?? "torrent_added");
  const ActiveTriggerIcon = $derived(triggerTypeIcons[activeTriggerType] as unknown as Component);

  function toggleEnabled() {
    if (!playlet) return;
    playletsState.updatePlaylet(playlet.id, { enabled: !playlet.enabled });
  }

  // Sync custom extensions input when playlet changes
  $effect(() => {
    if (playlet?.fileFilter?.category === "custom") {
      customExtInput = playlet.fileFilter.customExtensions.join(", ");
    }
  });

  function addAction(type: ActionType) {
    if (!playlet) return;
    playletsState.addAction(playlet.id, type);
  }

  function handleDuplicate() {
    if (!playlet) return;
    const newPlaylet = playletsState.duplicatePlaylet(playlet.id);
    if (newPlaylet) {
      requestClose(newPlaylet.id);
    }
  }

  function handleDelete() {
    if (!playlet) return;
    onDelete(playlet.id);
  }

  function setTriggerType(type: TriggerType) {
    if (!playlet) return;
    playletsState.updatePlaylet(playlet.id, {
      trigger: { ...playlet.trigger, type },
    });
  }

  function setSeedingRatio(value: string) {
    if (!playlet) return;
    const ratio = parseFloat(value);
    playletsState.updatePlaylet(playlet.id, {
      trigger: { ...playlet.trigger, seedingRatio: isNaN(ratio) ? undefined : ratio },
    });
  }

  async function pickWatchFolder() {
    if (!playlet) return;
    const dir = await open({ directory: true, multiple: false });
    if (dir) {
      playletsState.updatePlaylet(playlet.id, {
        trigger: { ...playlet.trigger, watchFolder: dir as string },
      });
    }
  }

  function clearWatchFolder() {
    if (!playlet) return;
    playletsState.updatePlaylet(playlet.id, {
      trigger: { ...playlet.trigger, watchFolder: undefined },
    });
  }

  function setFilterCategory(category: FileFilterCategory) {
    if (!playlet) return;
    const current = playlet.fileFilter;
    playletsState.setFileFilter(playlet.id, {
      category,
      customExtensions: current?.customExtensions ?? [],
      selectLargest: current?.selectLargest,
      minSizeMb: current?.minSizeMb,
      namePattern: current?.namePattern,
    });
  }

  function handleCustomExtChange(value: string) {
    if (!playlet) return;
    customExtInput = value;
    const exts = value.split(",").map((s) => s.trim()).filter(Boolean);
    playletsState.setFileFilter(playlet.id, {
      ...playlet.fileFilter!,
      category: "custom",
      customExtensions: exts,
    });
  }

  function toggleSelectLargest() {
    if (!playlet) return;
    const filter = playlet.fileFilter ?? { category: "all" as const, customExtensions: [] };
    playletsState.setFileFilter(playlet.id, {
      ...filter,
      selectLargest: !filter.selectLargest,
    });
  }

  function setMinSizeMb(value: string) {
    if (!playlet) return;
    const filter = playlet.fileFilter ?? { category: "all" as const, customExtensions: [] };
    const num = parseFloat(value);
    playletsState.setFileFilter(playlet.id, {
      ...filter,
      minSizeMb: isNaN(num) ? undefined : num,
    });
  }


  let dragFromIndex = $state<number | null>(null);
  let dropTargetIndex = $state<number | null>(null);
  let actionsContainer: HTMLElement | undefined = $state();

  function handleActionDragStart(index: number, e: PointerEvent) {
    dragFromIndex = index;
    actionsContainer?.setPointerCapture(e.pointerId);
  }

  function handleDragMove(e: PointerEvent) {
    if (dragFromIndex === null || !actionsContainer || !playlet) return;

    const actionEls = actionsContainer.querySelectorAll<HTMLElement>("[data-action-index]");
    const pointerY = e.clientY;
    let closest = 0;
    let closestDist = Infinity;

    actionEls.forEach((el) => {
      const rect = el.getBoundingClientRect();
      const mid = rect.top + rect.height / 2;
      const dist = Math.abs(pointerY - mid);
      const idx = parseInt(el.dataset.actionIndex!);
      if (pointerY < mid && dist < closestDist) {
        closest = idx;
        closestDist = dist;
      } else if (pointerY >= mid && dist < closestDist) {
        closest = idx + 1;
        closestDist = dist;
      }
    });

    dropTargetIndex = closest;
  }

  function handleDragEnd() {
    if (dragFromIndex === null) return;
    if (dropTargetIndex !== null && playlet) {
      const to = dropTargetIndex > dragFromIndex ? dropTargetIndex - 1 : dropTargetIndex;
      if (to !== dragFromIndex) {
        playletsState.reorderActions(playlet.id, dragFromIndex, to);
      }
    }
    dragFromIndex = null;
    dropTargetIndex = null;
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      requestClose();
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      requestClose();
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  role="dialog"
  aria-modal="true"
  aria-label={i18n.t("playlets.editPlaylet")}
  tabindex="-1"
  class="fixed inset-0 z-50 flex justify-end {closing ? 'backdrop-fade-out' : 'backdrop-fade-in'}"
  onclick={handleBackdropClick}
  onkeydown={handleKeyDown}
>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    class="{closing ? 'sidebar-slide-out' : 'sidebar-slide-in'} flex h-full w-full max-w-md flex-col border-l border-[var(--color-border)] bg-[var(--color-bg)] shadow-2xl"
    onclick={(e) => e.stopPropagation()}
  >
    {#if playlet}
      <!-- Header -->
      <div class="flex shrink-0 items-center justify-between border-b border-[var(--color-border)] px-5 py-4">
        <span class="min-w-0 truncate text-lg font-bold text-[var(--color-text)] {!playlet.enabled ? 'opacity-50' : ''}">
          {derivedName}
        </span>
        <button
          onclick={() => requestClose()}
          class="ml-3 shrink-0 rounded-lg p-1.5 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
        >
          <X class="h-5 w-5" />
        </button>
      </div>

      <!-- Scrollable content -->
      <div class="min-h-0 flex-1 overflow-y-auto">
        <div class="px-5 py-4">
          <!-- WHEN block (Trigger type selector) -->
          <div class="rounded-xl border border-[var(--color-primary)]/30 bg-[var(--color-primary)]/5 p-4">
            <div class="flex items-center gap-3">
              <h3 class="text-2xl font-black text-[var(--color-primary)]">{i18n.t("playlets.when")}</h3>
              <ActiveTriggerIcon class="h-4 w-4 text-[var(--color-primary)]" />
            </div>

            <div class="mt-3 grid grid-cols-2 gap-2">
              {#each Object.entries(triggerTypeKeys) as [type, key]}
                {@const TrigIcon = triggerTypeIcons[type as TriggerType] as unknown as Component}
                <button
                  onclick={() => setTriggerType(type as TriggerType)}
                  class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium transition-colors {activeTriggerType === type ? 'bg-[var(--color-primary)] text-white' : 'bg-[var(--color-primary)]/15 text-[var(--color-primary)]'} hover:opacity-80"
                >
                  <TrigIcon class="h-4 w-4" />
                  {i18n.t(key)}
                </button>
              {/each}
            </div>

            {#if activeTriggerType === "seeding_ratio"}
              <div class="mt-3 flex items-center gap-2">
                <span class="text-sm text-[var(--color-text-secondary)]">{i18n.t("playlets.targetRatio")}</span>
                <input
                  type="number"
                  step="0.1"
                  min="0.1"
                  value={playlet.trigger.seedingRatio ?? 2.0}
                  oninput={(e) => setSeedingRatio((e.target as HTMLInputElement).value)}
                  class="h-7 w-20 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-primary)]"
                />
              </div>
            {/if}

          </div>

          <!-- Connector line -->
          <div class="pointer-events-none flex justify-center">
            <div class="h-4 w-[4px] bg-[var(--color-border)]"></div>
          </div>

          <!-- WITH block (file filter) — always visible, defaults to "any file" -->
          <div class="rounded-xl border border-[var(--color-accent)]/30 bg-[var(--color-accent)]/5 p-4">
            <div class="flex items-center gap-3">
              <h3 class="text-2xl font-black text-[var(--color-accent)]">{i18n.t("playlets.with")}</h3>
              <FileVideo class="h-4 w-4 text-[var(--color-accent)]" />
            </div>

            <div class="mt-3 grid grid-cols-2 gap-2">
              {#each Object.entries(filterCategoryKeys).filter(([cat]) => cat !== "subtitle") as [cat, key]}
                {@const CatIcon = filterCategoryIcons[cat as FileFilterCategory] as unknown as Component}
                <button
                  onclick={() => setFilterCategory(cat as FileFilterCategory)}
                  class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium transition-colors {activeFileCategory === cat ? 'bg-[var(--color-accent)] text-white' : 'bg-[var(--color-accent)]/15 text-[var(--color-accent)]'} hover:opacity-80"
                >
                  <CatIcon class="h-4 w-4" />
                  {i18n.t(key)}
                </button>
              {/each}
            </div>

            {#if activeFileCategory === "custom"}
              <input
                type="text"
                value={customExtInput}
                oninput={(e) => handleCustomExtChange((e.target as HTMLInputElement).value)}
                placeholder="mkv, mp4"
                autocorrect="off"
                autocapitalize="off"
                spellcheck={false}
                class="mt-3 h-8 w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-accent)]"
              />
            {/if}

            <!-- Secondary filters -->
            <div class="mt-3 space-y-2">
              <label class="flex items-center gap-2 text-xs text-[var(--color-text-secondary)]">
                <input
                  type="checkbox"
                  checked={playlet.fileFilter?.selectLargest ?? false}
                  onchange={toggleSelectLargest}
                  class="rounded"
                />
                {i18n.t("playlets.largestOnly")}
              </label>
              <div class="flex items-center gap-2">
                <span class="text-xs text-[var(--color-text-secondary)]">{i18n.t("playlets.atLeast")}</span>
                <input
                  type="number"
                  value={playlet.fileFilter?.minSizeMb ?? ""}
                  oninput={(e) => setMinSizeMb((e.target as HTMLInputElement).value)}
                  placeholder={i18n.t("playlets.mb")}
                  class="h-7 w-20 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] outline-none focus:border-[var(--color-accent)]"
                />
                <span class="text-xs text-[var(--color-text-muted)]">{i18n.t("playlets.mb")}</span>
              </div>
            </div>
          </div>

            <!-- Connector line -->
            <div class="pointer-events-none flex justify-center">
              <div class="h-4 w-[4px] bg-[var(--color-border)]"></div>
            </div>

          <!-- THEN action boxes -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            bind:this={actionsContainer}
            onpointermove={handleDragMove}
            onpointerup={handleDragEnd}
            onlostpointercapture={handleDragEnd}
          >
            {#each playlet.actions as action, i (action.id)}
              <!-- Connector line or drop indicator -->
              <div class="pointer-events-none flex justify-center">
                {#if dropTargetIndex === i && dragFromIndex !== null && dragFromIndex !== i && dragFromIndex !== i - 1}
                  <div class="h-4 w-full rounded-full flex items-center">
                    <div class="h-[4px] w-full rounded-full bg-[var(--color-primary)]"></div>
                  </div>
                {:else}
                  <div class="h-4 w-[4px] bg-[var(--color-border)]"></div>
                {/if}
              </div>

              <div
                data-action-index={i}
                class="transition-opacity {dragFromIndex === i ? 'opacity-40' : ''}"
              >
                <ActionRow
                  {action}
                  playletId={playlet.id}
                  isFirst={i === 0}
                  index={i}
                  totalActions={playlet.actions.length}
                  onDragStart={(e) => handleActionDragStart(i, e)}
                  onDuplicate={() => {
                    const clone = { ...action, id: crypto.randomUUID() };
                    playletsState.addAction(playlet.id, action.type, i + 1);
                    // Overwrite the newly created action with a copy of the source
                    const p = playletsState.getById(playlet.id);
                    if (p) {
                      const newAction = p.actions[i + 1];
                      if (newAction) playletsState.updateAction(playlet.id, newAction.id, { ...action, id: newAction.id } as any);
                    }
                  }}
                  onMoveUp={() => { if (i > 0) playletsState.reorderActions(playlet.id, i, i - 1); }}
                  onMoveDown={() => { if (i < playlet.actions.length - 1) playletsState.reorderActions(playlet.id, i, i + 1); }}
                />
              </div>
            {/each}

            <!-- Final drop indicator -->
            {#if playlet.actions.length > 0}
              <div class="pointer-events-none flex justify-center">
                {#if dropTargetIndex === playlet.actions.length && dragFromIndex !== null && dragFromIndex !== playlet.actions.length - 1}
                  <div class="h-4 w-full rounded-full flex items-center">
                    <div class="h-[4px] w-full rounded-full bg-[var(--color-primary)]"></div>
                  </div>
                {:else}
                  <div class="h-4 w-[4px] bg-[var(--color-border)]"></div>
                {/if}
              </div>
            {/if}
          </div>

          <!-- Connector line + empty Then box -->
          <div class="pointer-events-none flex justify-center">
            <div class="h-4 w-[4px] bg-[var(--color-border)]"></div>
          </div>

          <div class="relative rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4">
            <h3 class="mb-3 text-2xl font-black text-[var(--color-text-muted)]">{i18n.t("playlets.then")}</h3>

            <div class="grid grid-cols-2 gap-2">
              {#each thenGridItems as item}
                {@const def = getActionDef(item.type)}
                {@const taken = isActionTaken(item.type)}
                {#if def}
                  {@const DefIcon = def.icon as Component}
                  <button
                    onclick={() => addAction(item.type as ActionType)}
                    disabled={taken}
                    class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium transition-opacity {taken ? 'opacity-30 cursor-not-allowed' : 'text-white hover:opacity-80'}"
                    style="background-color: {def.color}"
                  >
                    <DefIcon class="h-4 w-4" />
                    {item.label ?? def.label}
                  </button>
                {/if}
              {/each}
            </div>
          </div>
        </div>
      </div>

      <!-- Footer toolbar -->
      <div class="flex shrink-0 items-center border-t border-[var(--color-border)] px-5 py-3">
        <!-- Toggle -->
        <button
          onclick={toggleEnabled}
          class="flex items-center gap-2 rounded-lg px-3 py-1.5 text-xs transition-colors hover:bg-[var(--color-bg-tertiary)]"
        >
          <span
            class="relative h-5 w-9 shrink-0 rounded-full transition-colors {playlet.enabled ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-border)]'}"
          >
            <span class="absolute top-0.5 h-4 w-4 rounded-full bg-white transition-transform {playlet.enabled ? 'left-[1.125rem]' : 'left-0.5'}"></span>
          </span>
          <span class="{playlet.enabled ? 'text-[var(--color-primary)]' : 'text-[var(--color-text-muted)]'}">
            {playlet.enabled ? i18n.t("common.enabled") : i18n.t("common.disabled")}
          </span>
        </button>

        <!-- Right-side actions -->
        <div class="ml-auto flex items-center">
          <div class="w-px h-4 bg-[var(--color-border)]"></div>
          <button
            onclick={handleDuplicate}
            class="flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
          >
            <Copy class="h-3.5 w-3.5" />
            {i18n.t("common.duplicate")}
          </button>
          {#if playletsState.count > 1}
            <div class="w-px h-4 bg-[var(--color-border)]"></div>
            <button
              onclick={handleDelete}
              class="flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-error)]"
            >
              <Trash2 class="h-3.5 w-3.5" />
              {i18n.t("common.delete")}
            </button>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .sidebar-slide-in {
    animation: slideIn 200ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  .sidebar-slide-out {
    animation: slideOut 200ms cubic-bezier(0.7, 0, 0.84, 0) forwards;
  }

  .backdrop-fade-in {
    animation: fadeIn 200ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  .backdrop-fade-out {
    animation: fadeOut 200ms cubic-bezier(0.7, 0, 0.84, 0) forwards;
  }

  @keyframes slideIn {
    from { transform: translateX(100%); }
    to   { transform: translateX(0); }
  }

  @keyframes slideOut {
    from { transform: translateX(0); }
    to   { transform: translateX(100%); }
  }

  @keyframes fadeIn {
    from { background-color: transparent; }
    to   { background-color: rgb(0 0 0 / 0.3); }
  }

  @keyframes fadeOut {
    from { background-color: rgb(0 0 0 / 0.3); }
    to   { background-color: transparent; }
  }
</style>
