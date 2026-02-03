<script lang="ts">
  import { Folder, X, Clipboard, FolderOpen, Trash2 } from "lucide-svelte";
  import { settingsState } from "$lib/state/settings.svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { open as openShell } from "@tauri-apps/plugin-shell";
  import ColorSchemePicker from "$lib/components/common/ColorSchemePicker.svelte";
  import ContextMenu from "$lib/components/common/ContextMenu.svelte";
  import { useContextMenu } from "$lib/utils";
  import { uiState } from "$lib/state/ui.svelte";
  import type { ContextMenuEntry } from "$lib/types/ui";

  async function pickDirectory() {
    const dir = await openDialog({ directory: true, multiple: false });
    if (dir) {
      await settingsState.updateAndSave({ download_directory: dir as string });
    }
  }

  async function pickIncompleteDirectory() {
    const dir = await openDialog({ directory: true, multiple: false });
    if (dir) {
      await settingsState.updateAndSave({ incomplete_directory: dir as string });
    }
  }

  function clearIncompleteDirectory() {
    settingsState.updateAndSave({ incomplete_directory: "" });
  }

  function handleToggle(key: "always_on_top" | "auto_discover" | "enable_upnp" | "watch_folders_enabled") {
    settingsState.updateAndSave({ [key]: !settingsState.settings[key] });
  }

  function handleNumber(key: "max_download_speed" | "max_upload_speed" | "media_server_port" | "listen_port" | "max_concurrent_tasks", e: Event) {
    const value = parseInt((e.target as HTMLInputElement).value) || 0;
    settingsState.updateAndSave({ [key]: value });
  }

  const fieldClass = "h-10 w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 text-sm text-[var(--color-text)] focus:border-[var(--color-primary)] focus:outline-none focus:ring-1 focus:ring-[var(--color-primary)]";

  let subtitleLangInput = $state(settingsState.settings.subtitle_languages.join(", "));

  function handleSubtitleLangChange(e: Event) {
    const value = (e.target as HTMLInputElement).value;
    subtitleLangInput = value;
    const langs = value.split(",").map((s: string) => s.trim()).filter(Boolean);
    settingsState.updateAndSave({ subtitle_languages: langs });
  }

  async function addWatchFolder() {
    const dir = await openDialog({ directory: true, multiple: false });
    if (dir) {
      const folders = [...settingsState.settings.watch_folders];
      if (!folders.includes(dir as string)) {
        folders.push(dir as string);
        await settingsState.updateAndSave({ watch_folders: folders });
      }
    }
  }

  function removeWatchFolder(folder: string) {
    const folders = settingsState.settings.watch_folders.filter((f) => f !== folder);
    settingsState.updateAndSave({ watch_folders: folders });
  }

  function handleApiKeyChange(e: Event) {
    const value = (e.target as HTMLInputElement).value;
    settingsState.updateAndSave({ opensubtitles_api_key: value });
  }

  // Context menus for directory paths
  const dirCtx = useContextMenu<{ path: string; browse: () => void }>();
  const watchCtx = useContextMenu<string>();

  function dirMenuItems(path: string, browse: () => void): ContextMenuEntry[] {
    return [
      {
        icon: Clipboard,
        label: "Copy Path",
        disabled: !path,
        action: () => {
          navigator.clipboard.writeText(path);
          uiState.addToast("Copied to clipboard", "success");
        },
      },
      {
        icon: FolderOpen,
        label: "Reveal in Finder",
        disabled: !path,
        action: () => openShell(path),
      },
      { type: "divider" },
      {
        icon: Folder,
        label: "Browse...",
        action: browse,
      },
    ];
  }

  function watchFolderMenuItems(folder: string): ContextMenuEntry[] {
    return [
      {
        icon: Clipboard,
        label: "Copy Path",
        action: () => {
          navigator.clipboard.writeText(folder);
          uiState.addToast("Copied to clipboard", "success");
        },
      },
      {
        icon: FolderOpen,
        label: "Reveal in Finder",
        action: () => openShell(folder),
      },
      { type: "divider" },
      {
        icon: Trash2,
        label: "Remove",
        danger: true,
        action: () => removeWatchFolder(folder),
      },
    ];
  }
</script>

<div class="mx-auto max-w-2xl p-6">
  <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)]">
    <!-- General -->
    <div class="px-6 py-5">
      <h3 class="mb-4 text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">General</h3>
      <div class="space-y-4">
        <div>
          <label for="download-dir" class="mb-1 block text-sm text-[var(--color-text-secondary)]">Save downloads to</label>
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="flex gap-2" oncontextmenu={(e) => dirCtx.open(e, { path: settingsState.settings.download_directory, browse: pickDirectory })}>
            <input
              id="download-dir"
              type="text"
              readonly
              value={settingsState.settings.download_directory}
              class="flex-1 {fieldClass}"
            />
            <button
              onclick={pickDirectory}
              class="flex h-10 items-center gap-2 rounded-lg bg-[var(--color-bg-tertiary)] px-3 text-sm text-[var(--color-text-secondary)] hover:bg-[var(--color-border)]"
            >
              <Folder class="h-4 w-4" />
              Browse
            </button>
          </div>
        </div>
        <div>
          <label for="incomplete-dir" class="mb-1 block text-sm text-[var(--color-text-secondary)]">Incomplete downloads</label>
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="flex gap-2" oncontextmenu={(e) => dirCtx.open(e, { path: settingsState.settings.incomplete_directory, browse: pickIncompleteDirectory })}>
            <input
              id="incomplete-dir"
              type="text"
              readonly
              value={settingsState.settings.incomplete_directory || "Same as download folder"}
              class="flex-1 {fieldClass} {!settingsState.settings.incomplete_directory ? 'text-[var(--color-text-muted)]' : ''}"
            />
            {#if settingsState.settings.incomplete_directory}
              <button
                onclick={clearIncompleteDirectory}
                class="flex h-10 items-center gap-2 rounded-lg bg-[var(--color-bg-tertiary)] px-3 text-sm text-[var(--color-text-secondary)] hover:bg-[var(--color-border)]"
              >
                <X class="h-4 w-4" />
              </button>
            {/if}
            <button
              onclick={pickIncompleteDirectory}
              class="flex h-10 items-center gap-2 rounded-lg bg-[var(--color-bg-tertiary)] px-3 text-sm text-[var(--color-text-secondary)] hover:bg-[var(--color-border)]"
            >
              <Folder class="h-4 w-4" />
              Browse
            </button>
          </div>
          <p class="mt-1 text-xs text-[var(--color-text-muted)]">Files move to the download folder when done</p>
        </div>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label for="max-tasks" class="mb-1 block text-sm text-[var(--color-text-secondary)]">Concurrent tasks</label>
            <input
              id="max-tasks"
              type="number"
              min="0"
              value={settingsState.settings.max_concurrent_tasks}
              onchange={(e) => handleNumber("max_concurrent_tasks", e)}
              class={fieldClass}
              placeholder="0 = no limit"
            />
          </div>
        </div>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label for="max-dl" class="mb-1 block text-sm text-[var(--color-text-secondary)]">Max download speed</label>
            <input
              id="max-dl"
              type="number"
              min="0"
              value={settingsState.settings.max_download_speed}
              onchange={(e) => handleNumber("max_download_speed", e)}
              class={fieldClass}
              placeholder="0 = no limit"
            />
          </div>
          <div>
            <label for="max-ul" class="mb-1 block text-sm text-[var(--color-text-secondary)]">Max upload speed</label>
            <input
              id="max-ul"
              type="number"
              min="0"
              value={settingsState.settings.max_upload_speed}
              onchange={(e) => handleNumber("max_upload_speed", e)}
              class={fieldClass}
              placeholder="0 = no limit"
            />
          </div>
        </div>
        <div>
          <label for="media-port" class="mb-1 block text-sm text-[var(--color-text-secondary)]">Streaming port</label>
          <input
            id="media-port"
            type="number"
            min="1024"
            max="65535"
            value={settingsState.settings.media_server_port}
            onchange={(e) => handleNumber("media_server_port", e)}
            class={fieldClass}
          />
        </div>
        <div class="flex items-center justify-between">
          <span class="text-sm text-[var(--color-text-secondary)]">Find devices on your network</span>
          <button
            onclick={() => handleToggle("auto_discover")}
            class="relative h-6 w-11 rounded-full transition-colors {settingsState.settings.auto_discover ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-bg-tertiary)]'}"
            title="Find devices on your network"
          >
            <span
              class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform shadow-sm {settingsState.settings.auto_discover ? 'translate-x-5' : ''}"
            ></span>
          </button>
        </div>
        <!-- TODO: always_on_top ~40% — toggle persists the setting but no
             setAlwaysOnTop() Tauri API call to actually pin the window -->
        <div class="flex items-center justify-between">
          <span class="text-sm text-[var(--color-text-secondary)]">Keep window in front</span>
          <button
            onclick={() => handleToggle("always_on_top")}
            class="relative h-6 w-11 rounded-full transition-colors {settingsState.settings.always_on_top ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-bg-tertiary)]'}"
            title="Keep window in front"
          >
            <span
              class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform shadow-sm {settingsState.settings.always_on_top ? 'translate-x-5' : ''}"
            ></span>
          </button>
        </div>
      </div>
    </div>

    <!-- Folder Watch -->
    <div class="border-t border-[var(--color-border)] px-6 py-5">
      <h3 class="mb-4 text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">Folder Watch</h3>
      <div class="space-y-4">
        <div class="flex items-center justify-between">
          <div>
            <span class="text-sm text-[var(--color-text-secondary)]">Watch folders for .torrent files</span>
            <p class="text-xs text-[var(--color-text-muted)]">New .torrent files will be auto-added</p>
          </div>
          <button
            onclick={() => handleToggle("watch_folders_enabled")}
            class="relative h-6 w-11 shrink-0 rounded-full transition-colors {settingsState.settings.watch_folders_enabled ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-bg-tertiary)]'}"
            title="Watch folders for .torrent files"
          >
            <span
              class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform shadow-sm {settingsState.settings.watch_folders_enabled ? 'translate-x-5' : ''}"
            ></span>
          </button>
        </div>
        {#if settingsState.settings.watch_folders.length > 0}
          <div class="space-y-2">
            {#each settingsState.settings.watch_folders as folder}
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div class="flex items-center gap-2 rounded-lg bg-[var(--color-bg)] px-3 py-2" oncontextmenu={(e) => watchCtx.open(e, folder)}>
                <Folder class="h-4 w-4 shrink-0 text-[var(--color-text-muted)]" />
                <span class="select-text min-w-0 flex-1 truncate text-sm text-[var(--color-text-secondary)]">{folder}</span>
                <button
                  onclick={() => removeWatchFolder(folder)}
                  class="shrink-0 rounded p-1 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-error)]"
                >
                  <X class="h-3.5 w-3.5" />
                </button>
              </div>
            {/each}
          </div>
        {/if}
        <button
          onclick={addWatchFolder}
          class="flex items-center gap-2 rounded-lg bg-[var(--color-bg-tertiary)] px-3 py-2 text-sm text-[var(--color-text-secondary)] hover:bg-[var(--color-border)]"
        >
          <Folder class="h-4 w-4" />
          Add folder
        </button>
      </div>
    </div>

    <!-- Network -->
    <div class="border-t border-[var(--color-border)] px-6 py-5">
      <h3 class="mb-4 text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">Network</h3>
      <div class="space-y-4">
        <div>
          <label for="listen-port" class="mb-1 block text-sm text-[var(--color-text-secondary)]">Incoming peer port</label>
          <input
            id="listen-port"
            type="number"
            min="1024"
            max="65535"
            value={settingsState.settings.listen_port}
            onchange={(e) => handleNumber("listen_port", e)}
            class={fieldClass}
          />
          <p class="mt-1 text-xs text-[var(--color-text-muted)]">Restart required to apply</p>
        </div>
        <div class="flex items-center justify-between">
          <div>
            <span class="text-sm text-[var(--color-text-secondary)]">UPnP port forwarding</span>
            <p class="text-xs text-[var(--color-text-muted)]">Auto-open port on your router. Restart required.</p>
          </div>
          <button
            onclick={() => handleToggle("enable_upnp")}
            class="relative h-6 w-11 shrink-0 rounded-full transition-colors {settingsState.settings.enable_upnp ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-bg-tertiary)]'}"
            title="UPnP port forwarding"
          >
            <span
              class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform shadow-sm {settingsState.settings.enable_upnp ? 'translate-x-5' : ''}"
            ></span>
          </button>
        </div>
      </div>
    </div>

    <!-- Subtitles -->
    <div class="border-t border-[var(--color-border)] px-6 py-5">
      <h3 class="mb-4 text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">Subtitles</h3>
      <div class="space-y-4">
        <div>
          <label for="opensub-key" class="mb-1 block text-sm text-[var(--color-text-secondary)]">OpenSubtitles key</label>
          <input
            id="opensub-key"
            type="text"
            value={settingsState.settings.opensubtitles_api_key}
            onchange={handleApiKeyChange}
            class={fieldClass}
            placeholder="Paste your key from opensubtitles.com"
          />
        </div>
        <div>
          <label for="sub-langs" class="mb-1 block text-sm text-[var(--color-text-secondary)]">Subtitle languages</label>
          <input
            id="sub-langs"
            type="text"
            value={subtitleLangInput}
            oninput={handleSubtitleLangChange}
            class={fieldClass}
            placeholder="English, Spanish, French"
          />
        </div>
      </div>
    </div>

    <!-- Appearance -->
    <div class="border-t border-[var(--color-border)] px-6 py-5">
      <h3 class="mb-4 text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">Appearance</h3>
      <ColorSchemePicker />
    </div>
  </div>
</div>

{#if dirCtx.state}
  <ContextMenu x={dirCtx.state.x} y={dirCtx.state.y} items={dirMenuItems(dirCtx.state.data.path, dirCtx.state.data.browse)} onclose={dirCtx.close} />
{/if}

{#if watchCtx.state}
  <ContextMenu x={watchCtx.state.x} y={watchCtx.state.y} items={watchFolderMenuItems(watchCtx.state.data)} onclose={watchCtx.close} />
{/if}
