<script lang="ts">
  import type { Component } from "svelte";
  import {
    Trash2,
    Tv,
    Terminal,
    Copy,
    ArrowUp,
    ArrowDown,
  } from "lucide-svelte";
  import { playletsState } from "$lib/state/playlets.svelte";
  import type {
    Action,
    ActionType,
    CastAction,
    MoveAction,
    PlayAction,
    SubtitleAction,
    AutomationAction,
    DelayAction,
    DelayUnit,
    WebhookAction,
  } from "$lib/types/playlet";
  import { devicesState } from "$lib/state/devices.svelte";
  import { listMediaPlayers } from "$lib/services/tauri-commands";
  import type { MediaPlayer } from "$lib/types/playback";
  import { open } from "@tauri-apps/plugin-dialog";

  import { onMount } from "svelte";
  import { getActionDef } from "$lib/services/action-registry";
  import ContextMenu from "$lib/components/common/ContextMenu.svelte";
  import { useContextMenu } from "$lib/utils";
  import type { ContextMenuEntry } from "$lib/types/ui";

  let {
    action,
    playletId,
    isFirst = false,
    index = 0,
    totalActions = 1,
    onDragStart,
    onDuplicate,
    onMoveUp,
    onMoveDown,
  }: {
    action: Action;
    playletId: string;
    isFirst?: boolean;
    index?: number;
    totalActions?: number;
    onDragStart?: (e: PointerEvent) => void;
    onDuplicate?: () => void;
    onMoveUp?: () => void;
    onMoveDown?: () => void;
  } = $props();

  let mediaPlayers = $state<MediaPlayer[]>([]);

  onMount(async () => {
    try {
      mediaPlayers = await listMediaPlayers();
    } catch {
      // Silently fail if media player detection is unavailable
    }
  });

  const AUTO_GROUP = new Set(["automation", "webhook"]);
  const isAutoGroup = $derived(AUTO_GROUP.has(action.type));

  const def = $derived(
    isAutoGroup ? getActionDef("automation") : getActionDef(action.type)
  );
  const Icon = $derived(
    (isAutoGroup ? Terminal : def?.icon) as Component | undefined
  );
  const cardColor = $derived(
    isAutoGroup ? "#8b5cf6" : (def?.color ?? "var(--color-primary)")
  );

  const configLabel = $derived(
    isAutoGroup
      ? (action.type === "webhook" ? "Send to" : "Run")
      : (def?.configLabel ?? action.type)
  );

  // Automation group: shell/applescript/shortcut come from automation.method, webhook is a separate type
  const autoMethodLabels: { key: string; label: string }[] = [
    { key: "shell", label: "Shell" },
    { key: "applescript", label: "AppleScript" },
    { key: "shortcut", label: "Shortcut" },
    { key: "webhook", label: "Webhook" },
  ];

  const activeAutoMethod = $derived(
    action.type === "webhook" ? "webhook" : (action as AutomationAction).method ?? "shell"
  );

  function switchAutoMethod(method: string) {
    if (method === "webhook") {
      if (action.type === "webhook") return;
      const newDef = getActionDef("webhook");
      playletsState.updateAction(playletId, action.id, { type: "webhook" as ActionType, ...newDef?.defaultData } as any);
    } else {
      if (action.type === "automation" && (action as AutomationAction).method === method) return;
      const newDef = getActionDef("automation");
      playletsState.updateAction(playletId, action.id, { type: "automation" as ActionType, ...newDef?.defaultData, method } as any);
    }
  }

  async function pickFolder() {
    const dir = await open({ directory: true, multiple: false });
    if (dir) {
      playletsState.updateAction<MoveAction>(playletId, action.id, { destination: dir as string });
    }
  }

  let subtitleLangInput = $state("");

  $effect(() => {
    if (action.type === "subtitle") {
      subtitleLangInput = (action as SubtitleAction).languages.join(", ");
    }
  });

  function handleSubtitleLangChange(value: string) {
    subtitleLangInput = value;
    const langs = value.split(",").map((s) => s.trim()).filter(Boolean);
    playletsState.updateAction<SubtitleAction>(playletId, action.id, { languages: langs });
  }

  const delayUnits: { value: DelayUnit; label: string }[] = [
    { value: "seconds", label: "sec" },
    { value: "minutes", label: "min" },
    { value: "days", label: "days" },
    { value: "weeks", label: "weeks" },
    { value: "months", label: "months" },
  ];

  const ctx = useContextMenu();

  function contextMenuItems(): ContextMenuEntry[] {
    return [
      {
        icon: Copy,
        label: "Duplicate",
        action: () => onDuplicate?.(),
      },
      {
        icon: Trash2,
        label: "Remove",
        danger: true,
        action: () => playletsState.removeAction(playletId, action.id),
      },
      { type: "divider" },
      {
        icon: ArrowUp,
        label: "Move Up",
        disabled: index === 0,
        action: () => onMoveUp?.(),
      },
      {
        icon: ArrowDown,
        label: "Move Down",
        disabled: index >= totalActions - 1,
        action: () => onMoveDown?.(),
      },
    ];
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="rounded-xl p-4 text-white"
  style="background-color: {cardColor}"
  oncontextmenu={(e) => ctx.open(e)}
>
  <!-- Header row: drag handle + delete -->
  <div
    class="mb-3 flex items-center justify-between cursor-grab active:cursor-grabbing touch-none"
    onpointerdown={(e) => {
      if ((e.target as HTMLElement).closest("button")) return;
      e.preventDefault();
      onDragStart?.(e);
    }}
  >
    <div class="flex items-center gap-2">
      <h3 class="text-2xl font-black text-white drop-shadow-sm">{isFirst ? "Then" : "And"}</h3>
      {#if Icon}
        <Icon class="h-5 w-5 text-white/70" />
      {/if}
    </div>
    <div class="flex items-center gap-0.5">
      <button
        onclick={() => playletsState.removeAction(playletId, action.id)}
        class="rounded p-1 text-white/40 hover:text-white"
        title="Remove"
      >
        <Trash2 class="h-4 w-4" />
      </button>
    </div>
  </div>

  <!-- Method toggle for auto group -->
  {#if isAutoGroup}
    <div class="mb-3 flex gap-1">
      {#each autoMethodLabels as am}
        <button
          onclick={() => switchAutoMethod(am.key)}
          class="rounded-lg px-2 py-1 text-xs font-medium transition-colors {activeAutoMethod === am.key ? 'bg-white/30 text-white' : 'bg-black/20 text-white/60 hover:text-white/80'}"
        >
          {am.label}
        </button>
      {/each}
    </div>
  {/if}

  <!-- Config row -->
  <div class="flex items-start gap-2">
    <span class="shrink-0 pt-1.5 text-sm font-semibold drop-shadow-sm">{configLabel}</span>
    <div class="min-w-0 flex-1">
      {#if action.type === "cast"}
        {#if devicesState.devices.length === 0}
          <span class="flex h-8 items-center rounded-lg bg-black/20 px-2 text-sm text-white/40">
            No devices found
          </span>
        {:else}
          <div class="grid grid-cols-2 gap-2">
            {#each devicesState.devices as device}
              <button
                onclick={() => playletsState.updateAction<CastAction>(playletId, action.id, { deviceId: device.id })}
                class="flex items-center gap-2 truncate rounded-lg px-3 py-2 text-sm font-medium transition-opacity hover:opacity-80 {(action as CastAction).deviceId === device.id ? 'bg-white/25 text-white' : 'bg-black/20 text-white/70'}"
              >
                <Tv class="h-4 w-4 shrink-0" />
                {device.name}
              </button>
            {/each}
          </div>
        {/if}
      {:else if action.type === "move"}
        <button
          onclick={pickFolder}
          class="h-8 w-full truncate rounded-lg bg-black/20 px-2 text-left text-sm text-white/70 hover:bg-black/30 hover:text-white/90"
        >
          {(action as MoveAction).destination || "Pick a folder..."}
        </button>
      {:else if action.type === "notify"}
        <span class="h-8 flex items-center rounded-lg bg-black/20 px-2 text-sm text-white/70">
          System notification
        </span>
      {:else if action.type === "play"}
        <select
          value={(action as PlayAction).app}
          onchange={(e) => playletsState.updateAction<PlayAction>(playletId, action.id, { app: (e.target as HTMLSelectElement).value })}
          class="h-8 w-full rounded-lg bg-black/20 px-2 text-sm text-white/90 outline-none"
        >
          <option value="" class="text-[var(--color-text)] bg-[var(--color-bg)]">Pick an app...</option>
          {#each mediaPlayers as player}
            <option value={player.name} class="text-[var(--color-text)] bg-[var(--color-bg)]">{player.name}</option>
          {/each}
        </select>
      {:else if action.type === "subtitle"}
        <input
          type="text"
          value={subtitleLangInput}
          oninput={(e) => handleSubtitleLangChange((e.target as HTMLInputElement).value)}
          placeholder="en, es"
          autocorrect="off"
          autocapitalize="off"
          spellcheck={false}
          class="h-8 w-full rounded-lg bg-black/20 px-2 text-sm text-white/90 outline-none placeholder:text-white/40"
        />
      {:else if action.type === "automation"}
        {@const autoAction = action as AutomationAction}
        <div class="space-y-2">
          {#if autoAction.method === "shell"}
            <textarea
              value={autoAction.script}
              oninput={(e) => playletsState.updateAction<AutomationAction>(playletId, action.id, { script: (e.target as HTMLTextAreaElement).value })}
              placeholder="echo $TORRENT_NAME"
              rows={3}
  
              autocapitalize="off"
              spellcheck={false}
              class="w-full rounded-lg bg-black/20 px-2 py-1 text-sm text-white/90 outline-none placeholder:text-white/40 font-mono resize-y"
            ></textarea>
          {:else if autoAction.method === "applescript"}
            <textarea
              value={autoAction.script}
              oninput={(e) => playletsState.updateAction<AutomationAction>(playletId, action.id, { script: (e.target as HTMLTextAreaElement).value })}
              placeholder='display dialog "Done downloading!"'
              rows={3}
  
              autocapitalize="off"
              spellcheck={false}
              class="w-full rounded-lg bg-black/20 px-2 py-1 text-sm text-white/90 outline-none placeholder:text-white/40 font-mono resize-y"
            ></textarea>
          {:else}
            <input
              type="text"
              value={autoAction.shortcutName}
              oninput={(e) => playletsState.updateAction<AutomationAction>(playletId, action.id, { shortcutName: (e.target as HTMLInputElement).value })}
              placeholder="My Shortcut Name"
  
              autocapitalize="off"
              spellcheck={false}
              class="h-8 w-full rounded-lg bg-black/20 px-2 text-sm text-white/90 outline-none placeholder:text-white/40"
            />
          {/if}
        </div>
      {:else if action.type === "delay"}
        <div class="flex items-center gap-2">
          <input
            type="number"
            min="1"
            value={(action as DelayAction).seconds}
            oninput={(e) => playletsState.updateAction<DelayAction>(playletId, action.id, { seconds: parseInt((e.target as HTMLInputElement).value) || 5 })}
            class="h-8 w-20 rounded-lg bg-black/20 px-2 text-sm text-white/90 outline-none"
          />
          <select
            value={(action as DelayAction).delayUnit ?? "seconds"}
            onchange={(e) => playletsState.updateAction<DelayAction>(playletId, action.id, { delayUnit: (e.target as HTMLSelectElement).value as DelayUnit })}
            class="h-8 rounded-lg bg-black/20 px-2 text-sm text-white/90 outline-none"
          >
            {#each delayUnits as du}
              <option value={du.value} class="text-[var(--color-text)] bg-[var(--color-bg)]">{du.label}</option>
            {/each}
          </select>
        </div>
      {:else if action.type === "webhook"}
        {@const webhookAction = action as WebhookAction}
        <div class="space-y-2">
          <input
            type="url"
            value={webhookAction.url}
            oninput={(e) => playletsState.updateAction<WebhookAction>(playletId, action.id, { url: (e.target as HTMLInputElement).value })}
            placeholder="https://example.com/webhook"

            autocapitalize="off"
            spellcheck={false}
            class="h-8 w-full rounded-lg bg-black/20 px-2 text-sm text-white/90 outline-none placeholder:text-white/40"
          />
          <div class="flex gap-1">
            {#each (["POST", "GET"] as const) as method}
              <button
                onclick={() => playletsState.updateAction<WebhookAction>(playletId, action.id, { method })}
                class="rounded-lg px-2 py-1 text-xs font-medium transition-colors {webhookAction.method === method ? 'bg-white/30 text-white' : 'bg-black/20 text-white/60 hover:text-white/80'}"
              >
                {method}
              </button>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

{#if ctx.state}
  <ContextMenu x={ctx.state.x} y={ctx.state.y} items={contextMenuItems()} onclose={ctx.close} />
{/if}
