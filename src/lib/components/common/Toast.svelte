<script lang="ts">
  import { X, CheckCircle, AlertCircle, AlertTriangle, Info, Clipboard, XCircle } from "lucide-svelte";
  import { uiState, type Toast } from "$lib/state/ui.svelte";
  import ContextMenu from "$lib/components/common/ContextMenu.svelte";
  import { useContextMenu } from "$lib/utils";
  import type { ContextMenuEntry } from "$lib/types/ui";
  import { i18n } from "$lib/i18n/state.svelte";

  let { toast }: { toast: Toast } = $props();

  const icons = {
    info: Info,
    success: CheckCircle,
    warning: AlertTriangle,
    error: AlertCircle,
  };

  const colorVars: Record<string, string> = {
    info: "var(--color-info)",
    success: "var(--color-success)",
    warning: "var(--color-warning)",
    error: "var(--color-error)",
  };

  const Icon = icons[toast.level];

  const ctx = useContextMenu();

  const menuItems = $derived<ContextMenuEntry[]>([
    {
      icon: Clipboard,
      label: i18n.t("toast.copyMessage"),
      action: () => {
        navigator.clipboard.writeText(toast.message);
        uiState.addToast(i18n.t("common.copiedToClipboard"), "success");
      },
    },
    {
      icon: XCircle,
      label: i18n.t("toast.dismiss"),
      action: () => uiState.removeToast(toast.id),
    },
  ]);
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="flex items-center gap-2 rounded-lg border px-3 py-2 text-sm shadow-lg"
  style="border-color: {colorVars[toast.level]}; background-color: color-mix(in srgb, {colorVars[toast.level]} 12%, var(--color-bg)); color: {colorVars[toast.level]}"
  oncontextmenu={(e) => ctx.open(e)}
>
  <Icon class="h-4 w-4 shrink-0" />
  <span class="select-text flex-1">{toast.message}</span>
  <button
    onclick={() => uiState.removeToast(toast.id)}
    class="shrink-0 rounded p-0.5 opacity-60 hover:opacity-100"
  >
    <X class="h-3 w-3" />
  </button>
</div>

{#if ctx.state}
  <ContextMenu x={ctx.state.x} y={ctx.state.y} items={menuItems} onclose={ctx.close} />
{/if}
