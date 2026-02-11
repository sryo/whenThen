<script lang="ts">
  import type { Component } from "svelte";
  import {
    Trash2,
    Tv,
    Terminal,
    Copy,
    ArrowUp,
    ArrowDown,
    MonitorPlay,
    Ellipsis,
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
  import { listMediaPlayers, checkAutomationPermission } from "$lib/services/tauri-commands";
  import { uiState } from "$lib/state/ui.svelte";
  import type { MediaPlayer } from "$lib/types/playback";
  import { open } from "@tauri-apps/plugin-dialog";

  import { onMount } from "svelte";
  import { getActionDef } from "$lib/services/action-registry";
  import ContextMenu from "$lib/components/common/ContextMenu.svelte";
  import { useContextMenu } from "$lib/utils";
  import type { ContextMenuEntry } from "$lib/types/ui";
  import { i18n } from "$lib/i18n/state.svelte";

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
  let automationPermissionDenied = $state(false);

  onMount(async () => {
    try {
      mediaPlayers = await listMediaPlayers();
    } catch {
      // Silently fail if media player detection is unavailable
    }

    // Check automation permission if AppleScript is already selected
    if (action.type === "automation" && (action as AutomationAction).method === "applescript") {
      try {
        await checkAutomationPermission();
        automationPermissionDenied = false;
      } catch {
        automationPermissionDenied = true;
      }
    }
  });

  async function requestAutomationPermission() {
    try {
      await checkAutomationPermission();
      automationPermissionDenied = false;
    } catch {
      automationPermissionDenied = true;
      uiState.addToast(i18n.t("actions.enableAutomationSettings"), "warning");
    }
  }

  // Actions sharing method selection UI (shell/applescript/shortcut/webhook)
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
      ? (action.type === "webhook" ? i18n.t("actions.sendTo") : i18n.t("actions.run"))
      : (def?.configLabel ?? action.type)
  );

  const autoMethodLabels = $derived([
    { key: "shell", label: i18n.t("actions.shell") },
    { key: "applescript", label: i18n.t("actions.applescript") },
    { key: "shortcut", label: i18n.t("actions.shortcut") },
    { key: "webhook", label: i18n.t("actions.webhookLabel") },
  ]);

  const activeAutoMethod = $derived(
    action.type === "webhook" ? "webhook" : (action as AutomationAction).method ?? "shell"
  );

  async function switchAutoMethod(method: string) {
    if (method === "webhook") {
      if (action.type === "webhook") return;
      const newDef = getActionDef("webhook");
      playletsState.updateAction(playletId, action.id, { type: "webhook" as ActionType, ...newDef?.defaultData } as any);
    } else {
      if (action.type === "automation" && (action as AutomationAction).method === method) return;
      const newDef = getActionDef("automation");
      playletsState.updateAction(playletId, action.id, { type: "automation" as ActionType, ...newDef?.defaultData, method } as any);

      // Request automation permission immediately for AppleScript
      if (method === "applescript") {
        try {
          await checkAutomationPermission();
          automationPermissionDenied = false;
        } catch {
          automationPermissionDenied = true;
        }
      } else {
        automationPermissionDenied = false;
      }
    }
  }

  async function pickFolder() {
    const dir = await open({ directory: true, multiple: false });
    if (dir) {
      playletsState.updateAction<MoveAction>(playletId, action.id, { destination: dir as string });
    }
  }

  async function pickCustomApp() {
    const path = await open({
      multiple: false,
      filters: [{ name: "Applications", extensions: ["app"] }],
      defaultPath: "/Applications",
    });
    if (path) {
      const name = (path as string).split("/").pop()?.replace(/\.app$/, "") || (path as string);
      playletsState.updateAction<PlayAction>(playletId, action.id, { app: name });
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

  const delayUnits = $derived([
    { value: "seconds" as DelayUnit, label: i18n.t("actions.delaySeconds") },
    { value: "minutes" as DelayUnit, label: i18n.t("actions.delayMinutes") },
    { value: "days" as DelayUnit, label: i18n.t("actions.delayDays") },
    { value: "weeks" as DelayUnit, label: i18n.t("actions.delayWeeks") },
    { value: "months" as DelayUnit, label: i18n.t("actions.delayMonths") },
  ]);

  const ctx = useContextMenu();

  function contextMenuItems(): ContextMenuEntry[] {
    return [
      {
        icon: Copy,
        label: i18n.t("common.duplicate"),
        action: () => onDuplicate?.(),
      },
      {
        icon: Trash2,
        label: i18n.t("common.remove"),
        danger: true,
        action: () => playletsState.removeAction(playletId, action.id),
      },
      { type: "divider" },
      {
        icon: ArrowUp,
        label: i18n.t("common.moveUp"),
        disabled: index === 0,
        action: () => onMoveUp?.(),
      },
      {
        icon: ArrowDown,
        label: i18n.t("common.moveDown"),
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
      <h3 class="text-2xl font-black text-white drop-shadow-sm">{isFirst ? i18n.t("playlets.then") : i18n.t("playlets.and")}</h3>
      {#if Icon}
        <Icon class="h-5 w-5 text-white/70" />
      {/if}
    </div>
    <div class="flex items-center gap-0.5">
      <button
        onclick={() => playletsState.removeAction(playletId, action.id)}
        class="rounded p-1 text-white/40 hover:text-white"
        title={i18n.t("common.remove")}
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
            {i18n.t("actions.noChromecastDevices")}
          </span>
        {:else}
          <div class="grid grid-cols-2 gap-2">
            {#each devicesState.devices as device}
              <button
                onclick={() => playletsState.updateAction<CastAction>(playletId, action.id, { deviceId: device.id })}
                class="flex items-center gap-2 overflow-hidden rounded-lg px-3 py-2 text-sm font-medium transition-opacity hover:opacity-80 {(action as CastAction).deviceId === device.id ? 'bg-white/25 text-white' : 'bg-black/20 text-white/70'}"
              >
                <Tv class="h-4 w-4 shrink-0" />
                <span class="truncate">{device.name}</span>
              </button>
            {/each}
          </div>
        {/if}
      {:else if action.type === "move"}
        <button
          onclick={pickFolder}
          class="h-8 w-full truncate rounded-lg bg-black/20 px-2 text-left text-sm text-white/70 hover:bg-black/30 hover:text-white/90"
        >
          {(action as MoveAction).destination || i18n.t("actions.chooseFolder")}
        </button>
      {:else if action.type === "notify"}
        <span class="h-8 flex items-center rounded-lg bg-black/20 px-2 text-sm text-white/70">
          {i18n.t("actions.systemNotification")}
        </span>
      {:else if action.type === "play"}
        {@const playAction = action as PlayAction}
        {@const currentApp = playAction.app}
        {@const isCustom = currentApp && !mediaPlayers.some((p) => p.name === currentApp)}
        <div class="mb-2">
          <label class="flex items-center gap-2 text-sm text-white/80">
            <input
              type="checkbox"
              checked={playAction.usePlaylist}
              onchange={() => playletsState.updateAction<PlayAction>(playletId, action.id, { usePlaylist: !playAction.usePlaylist })}
              class="rounded"
            />
            {i18n.t("actions.playPlaylist")}
          </label>
        </div>
        <div class="grid grid-cols-2 gap-2">
          {#each mediaPlayers as player}
            <button
              onclick={() => playletsState.updateAction<PlayAction>(playletId, action.id, { app: player.name })}
              class="flex items-center gap-2 overflow-hidden rounded-lg px-3 py-2 text-sm font-medium transition-opacity hover:opacity-80 {currentApp === player.name ? 'bg-white/25 text-white' : 'bg-black/20 text-white/70'}"
            >
              <MonitorPlay class="h-4 w-4 shrink-0" />
              <span class="truncate">{player.name}</span>
            </button>
          {/each}
          {#if isCustom}
            <button
              class="flex items-center gap-2 overflow-hidden rounded-lg px-3 py-2 text-sm font-medium bg-white/25 text-white"
            >
              <MonitorPlay class="h-4 w-4 shrink-0" />
              <span class="truncate">{currentApp}</span>
            </button>
          {/if}
          <button
            onclick={pickCustomApp}
            class="flex items-center gap-2 overflow-hidden rounded-lg px-3 py-2 text-sm font-medium transition-opacity hover:opacity-80 bg-black/20 text-white/70"
          >
            <Ellipsis class="h-4 w-4 shrink-0" />
            <span class="truncate">{i18n.t("common.other")}</span>
          </button>
        </div>
      {:else if action.type === "subtitle"}
        <input
          type="text"
          value={subtitleLangInput}
          oninput={(e) => handleSubtitleLangChange((e.target as HTMLInputElement).value)}
          placeholder="en, es, de"
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
              placeholder="echo $TORRENT_NAME downloaded"
              rows={3}
  
              autocapitalize="off"
              spellcheck={false}
              class="w-full rounded-lg bg-black/20 px-2 py-1 text-sm text-white/90 outline-none placeholder:text-white/40 font-mono resize-y"
            ></textarea>
          {:else if autoAction.method === "applescript"}
            <textarea
              value={autoAction.script}
              oninput={(e) => playletsState.updateAction<AutomationAction>(playletId, action.id, { script: (e.target as HTMLTextAreaElement).value })}
              placeholder='display notification "Done" with title "$TORRENT_NAME"'
              rows={3}

              autocapitalize="off"
              spellcheck={false}
              class="w-full rounded-lg bg-black/20 px-2 py-1 text-sm text-white/90 outline-none placeholder:text-white/40 font-mono resize-y"
            ></textarea>
            {#if automationPermissionDenied}
              <button
                onclick={requestAutomationPermission}
                class="rounded-lg bg-black/30 px-3 py-1.5 text-xs font-medium text-white/90 hover:bg-black/40"
              >
                {i18n.t("actions.grantAutomationPermission")}
              </button>
            {/if}
          {:else}
            <input
              type="text"
              value={autoAction.shortcutName}
              oninput={(e) => playletsState.updateAction<AutomationAction>(playletId, action.id, { shortcutName: (e.target as HTMLInputElement).value })}
              placeholder="Organize Downloads"
  
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
            placeholder="https://hooks.slack.com/..."

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
