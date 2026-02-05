<!-- Color scheme picker grid with local state for showcase. -->
<script lang="ts">
  import { colorSchemes } from "$lib/themes/schemes";
  import { applyColorScheme } from "$lib/themes/apply";
  import type { ColorScheme } from "$lib/themes/types";

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

  let selected = $state("default-dark");

  function selectScheme(scheme: ColorScheme) {
    selected = scheme.id;
    applyColorScheme(scheme);
  }
</script>

<div class="grid grid-cols-3 gap-2">
  {#each allSchemes as scheme (scheme.id)}
    <button
      onclick={() => selectScheme(scheme)}
      class="group rounded-lg border p-2 text-left transition-all {selected === scheme.id
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
</div>
