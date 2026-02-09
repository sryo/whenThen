<!-- Expandable task row with execution timeline. -->
<script lang="ts">
  import { Check, X, ChevronDown, ChevronUp, Ban, Settings, ShieldCheck, RotateCw } from "lucide-svelte";
  import type { Task, ActionResult } from "$lib/types/task";
  import { getActionDef } from "$lib/services/action-registry";
  import { actionPhrase, buildActionSummary } from "$lib/utils/playlet-display";
  import { playletsState } from "$lib/state/playlets.svelte";
  import { uiState } from "$lib/state/ui.svelte";
  import { checkAutomationPermission, openSystemSettings } from "$lib/services/tauri-commands";
  import { retryTask } from "$lib/services/execution-pipeline";
  import { i18n } from "$lib/i18n/state.svelte";

  interface Props {
    task: Task;
  }

  let { task }: Props = $props();

  let expanded = $state(false);

  const playlet = $derived(task.playletId ? playletsState.getById(task.playletId) : null);
  const hasFailed = $derived(task.status === "failed" || task.actionResults.some((ar) => ar.status === "failed"));

  const actionSummary = $derived.by(() => {
    // Touch i18n.locale to make this reactive to locale changes
    void i18n.locale;
    if (!playlet) return "";
    return buildActionSummary(playlet);
  });

  function getActionFromResult(result: ActionResult) {
    if (!playlet) return null;
    return playlet.actions.find((a) => a.id === result.actionId);
  }

  function statusIcon(status: ActionResult["status"]) {
    switch (status) {
      case "done": return Check;
      case "failed": return X;
      case "skipped": return Ban;
      default: return null;
    }
  }

  function statusColor(status: ActionResult["status"]) {
    switch (status) {
      case "done": return "var(--color-success)";
      case "failed": return "var(--color-error)";
      case "skipped": return "var(--color-text-muted)";
      default: return "var(--color-text-muted)";
    }
  }

  function formatRelativeTime(dateStr: string): string {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMin = Math.floor(diffMs / 60000);
    const diffHour = Math.floor(diffMin / 60);
    const diffDay = Math.floor(diffHour / 24);

    if (diffMin < 1) return i18n.t("inbox.justNow");
    if (diffMin < 60) return i18n.t("inbox.minutesAgo", { count: diffMin });
    if (diffHour < 24) return i18n.t("inbox.hoursAgo", { count: diffHour });
    if (diffDay < 7) return i18n.t("inbox.daysAgo", { count: diffDay });

    return date.toLocaleDateString(undefined, { month: "short", day: "numeric" });
  }

  function formatDateTime(dateStr: string): string {
    const date = new Date(dateStr);
    return date.toLocaleString(undefined, {
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    });
  }


  // Detect actionable errors/skips and return action info
  type ActionableError = { type: "settings" | "permission" | "retry"; label: string; section?: string };
  function getActionableError(text: string | null): ActionableError | null {
    if (!text) return null;
    const lower = text.toLowerCase();
    // API key issues
    if (lower.includes("opensubtitles") && lower.includes("api key")) {
      return { type: "settings", label: i18n.t("inbox.goToSettings"), section: "subtitles" };
    }
    // Automation/AppleScript errors - could be permissions or script issues
    if (lower.includes("applescript") || lower.includes("system events") || lower.includes("apple event")) {
      return { type: "permission", label: i18n.t("inbox.checkPermissions") };
    }
    // Move errors - directory issues
    if (lower.includes("directory not empty") || lower.includes("failed to move")) {
      return { type: "settings", label: i18n.t("inbox.goToSettings"), section: "playback" };
    }
    // No destination folder configured
    if (lower.includes("no destination") || lower.includes("destination folder")) {
      return { type: "settings", label: i18n.t("inbox.goToSettings"), section: "playback" };
    }
    // No cast device configured
    if (lower.includes("no cast") || lower.includes("no device") || lower.includes("chromecast")) {
      return { type: "settings", label: i18n.t("inbox.goToSettings"), section: "network" };
    }
    // No media player configured
    if (lower.includes("no media player")) {
      return { type: "settings", label: i18n.t("inbox.goToSettings"), section: "playback" };
    }
    return null;
  }

  async function handleActionableError(actionType: "settings" | "permission" | "retry", section?: string) {
    if (actionType === "settings") {
      uiState.goToSettings(section);
    } else if (actionType === "permission") {
      // Open macOS System Settings to Privacy & Security > Automation
      try {
        await openSystemSettings("Privacy_Automation");
      } catch {
        // Fallback: just trigger permission check
        checkAutomationPermission();
      }
    }
  }

  async function handleRetry() {
    await retryTask(task.id);
  }
</script>

<div class="overflow-hidden rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)]">
  <button
    onclick={() => expanded = !expanded}
    class="flex w-full items-start gap-3 p-3 text-left hover:bg-[var(--color-bg-secondary)]"
  >
    <!-- Content -->
    <div class="min-w-0 flex-1">
      <div class="flex items-center gap-2">
        <span class="truncate text-sm font-medium text-[var(--color-text)]">
          {task.torrentName}
        </span>
        {#if expanded}
          <ChevronUp class="ml-auto h-4 w-4 shrink-0 text-[var(--color-text-muted)]" />
        {:else}
          <ChevronDown class="ml-auto h-4 w-4 shrink-0 text-[var(--color-text-muted)]" />
        {/if}
      </div>
      <div class="mt-0.5 flex items-center gap-1.5 text-xs text-[var(--color-text-muted)]">
        {#if playlet && actionSummary}
          <span>{actionSummary}</span>
          <span>·</span>
        {/if}
        <span>{formatRelativeTime(task.createdAt)}</span>
      </div>
      <!-- Action status dots with icons, connected by lines -->
      {#if task.actionResults.length > 0}
        <div class="mt-1.5 flex items-center">
          {#each task.actionResults as result, i}
            {@const action = getActionFromResult(result)}
            {@const def = getActionDef(action?.type ?? result.actionType)}
            {#if i > 0}
              <div class="h-[2px] w-2 bg-[var(--color-border)]"></div>
            {/if}
            <div
              class="flex h-4 w-4 shrink-0 items-center justify-center rounded-full {result.status === 'failed' ? 'ring-2 ring-[var(--color-error)]/50' : ''}"
              style="background-color: {result.status === 'done' ? (def?.color ?? 'var(--color-text-muted)') : statusColor(result.status)}"
              title={action ? actionPhrase(action) : result.actionType}
            >
              {#if def}
                {@const ActionIcon = def.icon}
                <ActionIcon class="h-2.5 w-2.5 text-white" />
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </button>

  <!-- Expanded timeline -->
  <div class="expand-container {expanded ? 'expanded' : ''}">
    <div class="expand-content">
      <div class="border-t border-[var(--color-border)] bg-[var(--color-bg-secondary)] px-4 py-3">
      <div class="space-y-2 text-sm">
        <!-- Started -->
        <div class="flex items-center gap-3 text-[var(--color-text-muted)]">
          <span class="w-16 shrink-0">{i18n.t("inbox.started")}</span>
          <span>{formatDateTime(task.createdAt)}</span>
        </div>

        <!-- Actions -->
        {#if task.actionResults.length > 0}
          <div class="ml-4 border-l-2 border-[var(--color-border)] pl-4 space-y-2">
            {#each task.actionResults as result}
              {@const action = getActionFromResult(result)}
              {@const def = getActionDef(action?.type ?? result.actionType)}
              {@const Icon = statusIcon(result.status)}
              <div class="flex items-start gap-2">
                <div class="flex shrink-0 items-center gap-1">
                  <div class="flex h-5 w-5 items-center justify-center">
                    {#if def}
                      {@const ActionIcon = def.icon}
                      <ActionIcon class="h-4 w-4" style="color: {def.color}" />
                    {/if}
                  </div>
                  {#if Icon}
                    {@const StatusIcon = Icon}
                    <StatusIcon class="h-3.5 w-3.5" style="color: {statusColor(result.status)}" />
                  {/if}
                </div>
                <div class="min-w-0 flex-1">
                  <div class="flex items-center gap-2">
                    <span class="text-[var(--color-text)]">
                      {action ? actionPhrase(action) : result.actionType}
                    </span>
                    {#if result.completedAt}
                      <span class="ml-auto text-xs text-[var(--color-text-muted)]">
                        {formatDateTime(result.completedAt)}
                      </span>
                    {/if}
                  </div>
                  {#if result.status === "skipped" && result.skipReason}
                    <div class="text-xs text-[var(--color-text-muted)]">
                      {i18n.t("inbox.skipped")}: {result.skipReason}
                    </div>
                    {@const actionable = getActionableError(result.skipReason)}
                    {#if actionable}
                      <button
                        onclick={() => handleActionableError(actionable.type, actionable.section)}
                        class="mt-1 flex items-center gap-1 rounded-md bg-[var(--color-bg-tertiary)] px-2 py-1 text-xs text-[var(--color-text)] hover:bg-[var(--color-border)]"
                      >
                        {#if actionable.type === "settings"}
                          <Settings class="h-3 w-3" />
                        {:else}
                          <ShieldCheck class="h-3 w-3" />
                        {/if}
                        {actionable.label}
                      </button>
                    {/if}
                  {/if}
                  {#if result.status === "failed" && result.error}
                    <div class="text-xs text-[var(--color-error)]">
                      {result.error}
                    </div>
                    {@const actionable = getActionableError(result.error)}
                    {#if actionable}
                      <button
                        onclick={() => handleActionableError(actionable.type, actionable.section)}
                        class="mt-1 flex items-center gap-1 rounded-md bg-[var(--color-bg-tertiary)] px-2 py-1 text-xs text-[var(--color-text)] hover:bg-[var(--color-border)]"
                      >
                        {#if actionable.type === "settings"}
                          <Settings class="h-3 w-3" />
                        {:else}
                          <ShieldCheck class="h-3 w-3" />
                        {/if}
                        {actionable.label}
                      </button>
                    {/if}
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {/if}

        <!-- Finished -->
        {#if task.completedAt}
          <div class="flex items-center gap-3 text-[var(--color-text-muted)]">
            <span class="w-16 shrink-0">{hasFailed ? i18n.t("inbox.failed") : i18n.t("inbox.finished")}</span>
            <span>{formatDateTime(task.completedAt)}</span>
            {#if hasFailed}
              <button
                onclick={handleRetry}
                class="ml-auto flex items-center gap-1 rounded-md bg-[var(--color-bg-tertiary)] px-2 py-1 text-xs text-[var(--color-text)] hover:bg-[var(--color-border)]"
              >
                <RotateCw class="h-3 w-3" />
                {i18n.t("inbox.retry")}
              </button>
            {/if}
          </div>
        {/if}
      </div>
    </div>
    </div>
  </div>
</div>
