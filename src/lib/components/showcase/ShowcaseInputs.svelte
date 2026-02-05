<!-- Input and control variants for showcase. -->
<script lang="ts">
  import { Folder } from "lucide-svelte";

  const fieldClass = "h-10 w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 text-sm text-[var(--color-text)] focus:border-[var(--color-primary)] focus:outline-none focus:ring-1 focus:ring-[var(--color-primary)]";

  let toggleOn = $state(true);
  let toggleOff = $state(false);
  let checkboxOn = $state(true);
  let checkboxOff = $state(false);
</script>

<div class="space-y-6">
  <!-- Text input -->
  <div>
    <label for="sc-text" class="mb-1 block text-sm text-[var(--color-text-secondary)]">Text input</label>
    <input
      id="sc-text"
      type="text"
      value="OpenSubtitles API key"
      class={fieldClass}
      placeholder="Paste your key"
    />
  </div>

  <!-- Number inputs -->
  <div class="grid grid-cols-2 gap-4">
    <div>
      <label for="sc-num" class="mb-1 block text-sm text-[var(--color-text-secondary)]">Number input</label>
      <input
        id="sc-num"
        type="number"
        value={8080}
        class={fieldClass}
        placeholder="0 = no limit"
      />
    </div>
    <div>
      <label for="sc-num-empty" class="mb-1 block text-sm text-[var(--color-text-secondary)]">Number (empty)</label>
      <input
        id="sc-num-empty"
        type="number"
        class={fieldClass}
        placeholder="0 = no limit"
      />
    </div>
  </div>

  <!-- Select dropdown -->
  <div>
    <label for="sc-select" class="mb-1 block text-sm text-[var(--color-text-secondary)]">Select dropdown</label>
    <select id="sc-select" class={fieldClass}>
      <option value="">None</option>
      <option value="living-room">Living Room TV</option>
      <option value="bedroom">Bedroom Chromecast</option>
      <option value="office">Office Display</option>
    </select>
  </div>

  <!-- Toggle switches -->
  <div>
    <span class="mb-2 block text-sm text-[var(--color-text-secondary)]">Toggle switch</span>
    <div class="flex items-center gap-6">
      <div class="flex items-center gap-2">
        <button
          onclick={() => (toggleOn = !toggleOn)}
          class="relative h-6 w-11 rounded-full transition-colors {toggleOn ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-bg-tertiary)]'}"
          title="Toggle on"
        >
          <span
            class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform shadow-sm {toggleOn ? 'translate-x-5' : ''}"
          ></span>
        </button>
        <span class="text-sm text-[var(--color-text-secondary)]">On</span>
      </div>
      <div class="flex items-center gap-2">
        <button
          onclick={() => (toggleOff = !toggleOff)}
          class="relative h-6 w-11 rounded-full transition-colors {toggleOff ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-bg-tertiary)]'}"
          title="Toggle off"
        >
          <span
            class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform shadow-sm {toggleOff ? 'translate-x-5' : ''}"
          ></span>
        </button>
        <span class="text-sm text-[var(--color-text-secondary)]">Off</span>
      </div>
    </div>
  </div>

  <!-- Checkboxes -->
  <div>
    <span class="mb-2 block text-sm text-[var(--color-text-secondary)]">Checkbox</span>
    <div class="flex items-center gap-6">
      <label class="flex items-center gap-2 text-sm text-[var(--color-text-secondary)]">
        <input type="checkbox" bind:checked={checkboxOn} class="rounded" />
        Checked
      </label>
      <label class="flex items-center gap-2 text-sm text-[var(--color-text-secondary)]">
        <input type="checkbox" bind:checked={checkboxOff} class="rounded" />
        Unchecked
      </label>
    </div>
  </div>

  <!-- Button variants -->
  <div>
    <span class="mb-2 block text-sm text-[var(--color-text-secondary)]">Buttons</span>
    <div class="flex flex-wrap items-center gap-2">
      <!-- Primary (filled) -->
      <button class="flex h-10 items-center rounded-lg bg-[var(--color-primary)] px-4 text-sm font-medium text-white hover:bg-[var(--color-primary-hover)]">
        Primary
      </button>
      <!-- Secondary (bg-tertiary) -->
      <button class="flex h-10 items-center rounded-lg bg-[var(--color-bg-tertiary)] px-4 text-sm text-[var(--color-text-secondary)] hover:bg-[var(--color-border)]">
        Secondary
      </button>
      <!-- Danger -->
      <button class="flex h-10 items-center rounded-lg bg-[var(--color-bg-tertiary)] px-4 text-sm text-[var(--color-text-secondary)] hover:bg-[var(--color-error)] hover:text-white">
        Danger
      </button>
      <!-- Icon-only -->
      <button class="flex h-10 w-10 items-center justify-center rounded-lg bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)] hover:bg-[var(--color-border)]" title="Icon button">
        <Folder class="h-4 w-4" />
      </button>
      <!-- Pill -->
      <button class="flex items-center gap-1.5 rounded-full bg-[var(--color-primary)]/15 px-3 py-1.5 text-xs font-medium text-[var(--color-primary)]">
        Pill label
      </button>
      <!-- Disabled -->
      <button disabled class="flex h-10 items-center rounded-lg bg-[var(--color-bg-tertiary)] px-4 text-sm text-[var(--color-text-muted)] opacity-50">
        Disabled
      </button>
    </div>
  </div>

  <!-- Directory picker row -->
  <div>
    <label for="sc-dir" class="mb-1 block text-sm text-[var(--color-text-secondary)]">Directory picker</label>
    <div class="flex gap-2">
      <input
        id="sc-dir"
        type="text"
        readonly
        value="/Users/demo/Downloads"
        class="flex-1 {fieldClass}"
      />
      <button class="flex h-10 items-center gap-2 rounded-lg bg-[var(--color-bg-tertiary)] px-3 text-sm text-[var(--color-text-secondary)] hover:bg-[var(--color-border)]">
        <Folder class="h-4 w-4" />
        Browse...
      </button>
    </div>
    <p class="mt-1 text-xs text-[var(--color-text-muted)]">Helper text below the input</p>
  </div>
</div>
