<script lang="ts">
  import { colorSchemes } from "$lib/themes/schemes";
  import type { ColorScheme } from "$lib/themes/types";
  import { settingsState } from "$lib/state/settings.svelte";

  const darkSchemes = colorSchemes.filter((s) => s.variant === "dark");
  const lightSchemes = colorSchemes.filter((s) => s.variant === "light");
  const allSchemes = [...darkSchemes, ...lightSchemes];

  const swatchKeys: (keyof ColorScheme["colors"])[] = [
    "bg",
    "text",
    "primary",
    "success",
    "warning",
    "error",
  ];

  function isSelected(scheme: ColorScheme): boolean {
    const current = settingsState.settings.color_scheme;
    if (current === "auto") return false;
    return scheme.id === current;
  }

  function selectScheme(id: string) {
    settingsState.updateAndSave({ color_scheme: id });
  }
</script>

<div class="grid grid-cols-3 gap-2">
  {#each allSchemes as scheme (scheme.id)}
    <button
      onclick={() => selectScheme(scheme.id)}
      class="group rounded-lg border p-2 text-left transition-all {isSelected(scheme)
        ? 'border-[var(--color-primary)] shadow-sm'
        : 'border-[var(--color-border)] hover:border-[var(--color-text-muted)]'}"
    >
      <div
        class="flex h-5 items-center gap-0 overflow-hidden rounded"
        style="background-color: {scheme.colors.bg}"
      >
        {#each swatchKeys as key}
          <div class="h-full flex-1" style="background-color: {scheme.colors[key]}"></div>
        {/each}
      </div>
      <p class="mt-1.5 truncate text-center text-xs font-medium text-[var(--color-text)]" title={scheme.name}>
        {scheme.name}
      </p>
    </button>
  {/each}

  <!-- Auto option -->
  <button
    onclick={() => selectScheme("auto")}
    class="group rounded-lg border p-2 text-center text-xs font-medium transition-all {settingsState.settings.color_scheme === 'auto'
      ? 'border-[var(--color-primary)] text-[var(--color-primary)] shadow-sm'
      : 'border-[var(--color-border)] text-[var(--color-text-muted)] hover:border-[var(--color-text-muted)]'}"
  >
    <div class="flex h-5 items-center justify-center rounded bg-[var(--color-bg-tertiary)]">
      <span class="text-[10px] uppercase tracking-wide">Auto</span>
    </div>
    <p class="mt-1.5">System</p>
  </button>
</div>
