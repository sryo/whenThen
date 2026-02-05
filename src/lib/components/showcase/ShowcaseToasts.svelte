<!-- Static toast variants for showcase. -->
<script lang="ts">
  import { Info, CheckCircle, AlertTriangle, AlertCircle, X } from "lucide-svelte";

  const toasts = [
    { level: "info", icon: Info, message: "Torrent added to queue" },
    { level: "success", icon: CheckCircle, message: "Download complete" },
    { level: "warning", icon: AlertTriangle, message: "Slow connection detected" },
    { level: "error", icon: AlertCircle, message: "Could not connect to device" },
  ] as const;

  const colorVars: Record<string, string> = {
    info: "var(--color-info)",
    success: "var(--color-success)",
    warning: "var(--color-warning)",
    error: "var(--color-error)",
  };
</script>

<div class="space-y-2">
  {#each toasts as toast}
    <div
      class="flex items-center gap-2 rounded-lg border px-3 py-2 text-sm shadow-lg"
      style="border-color: {colorVars[toast.level]}; background-color: color-mix(in srgb, {colorVars[toast.level]} 12%, var(--color-bg)); color: {colorVars[toast.level]}"
    >
      <toast.icon class="h-4 w-4 shrink-0" />
      <span class="flex-1">{toast.message}</span>
      <div class="shrink-0 rounded p-0.5 opacity-60">
        <X class="h-3 w-3" />
      </div>
    </div>
  {/each}
</div>
