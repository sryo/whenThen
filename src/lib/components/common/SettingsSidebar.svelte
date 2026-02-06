<script lang="ts">
  import { Folder, X, Clipboard, FolderOpen, Trash2, Cast, MonitorPlay, FolderOutput, Eye, EyeOff } from "lucide-svelte";
  import { settingsState } from "$lib/state/settings.svelte";
  import { uiState } from "$lib/state/ui.svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { open as openShell } from "@tauri-apps/plugin-shell";
  import ContextMenu from "$lib/components/common/ContextMenu.svelte";
  import { useContextMenu } from "$lib/utils";
  import type { ContextMenuEntry } from "$lib/types/ui";
  import {
    checkFileAssociations,
    setDefaultForTorrents,
    setDefaultForMagnets,
    listMediaPlayers,
    type FileAssociationStatus,
  } from "$lib/services/tauri-commands";
  import { devicesState } from "$lib/state/devices.svelte";
  import type { MediaPlayer } from "$lib/types/playback";
  import { onMount } from "svelte";

  let closing = $state(false);

  function close() {
    closing = true;
    setTimeout(() => {
      uiState.closeSettings();
      closing = false;
    }, 200);
  }

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

  function handleToggle(key: "auto_discover" | "enable_upnp" | "watch_folders_enabled" | "auto_play_next" | "delete_torrent_file_on_add" | "show_tray_icon" | "skip_template_picker") {
    settingsState.updateAndSave({ [key]: !settingsState.settings[key] });
  }

  function handleNumber(key: "max_download_speed" | "max_upload_speed" | "media_server_port" | "listen_port" | "max_concurrent_tasks" | "picker_countdown_seconds", e: Event) {
    const value = parseInt((e.target as HTMLInputElement).value) || 0;
    settingsState.updateAndSave({ [key]: value });
  }

  const fieldClass = "h-10 w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 text-sm text-[var(--color-text)] focus:border-[var(--color-primary)] focus:outline-none focus:ring-1 focus:ring-[var(--color-primary)]";

  // Common languages for subtitle chip picker
  const commonLanguages = [
    { code: "en", name: "English" },
    { code: "es", name: "Spanish" },
    { code: "fr", name: "French" },
    { code: "de", name: "German" },
    { code: "pt", name: "Portuguese" },
    { code: "it", name: "Italian" },
    { code: "ja", name: "Japanese" },
    { code: "ko", name: "Korean" },
    { code: "zh", name: "Chinese" },
    { code: "ar", name: "Arabic" },
  ];

  function toggleLanguage(code: string) {
    const current = settingsState.settings.subtitle_languages;
    const newLangs = current.includes(code)
      ? current.filter((c) => c !== code)
      : [...current, code];
    settingsState.updateAndSave({ subtitle_languages: newLangs });
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

  // API key visibility toggle
  let showApiKey = $state(false);

  function handleApiKeyChange(e: Event) {
    const value = (e.target as HTMLInputElement).value;
    settingsState.updateAndSave({ opensubtitles_api_key: value });
  }

  // File associations
  let associations = $state<FileAssociationStatus>({ torrent_files: false, magnet_links: false });
  let associationsLoading = $state(false);

  async function loadAssociations() {
    try {
      associations = await checkFileAssociations();
    } catch {
      // Non-macOS or bundle not available
    }
  }

  async function handleSetTorrentDefault() {
    associationsLoading = true;
    try {
      await setDefaultForTorrents();
      await loadAssociations();
    } catch (e) {
      uiState.addToast(String(e), "error");
    } finally {
      associationsLoading = false;
    }
  }

  async function handleSetMagnetDefault() {
    associationsLoading = true;
    try {
      await setDefaultForMagnets();
      await loadAssociations();
    } catch (e) {
      uiState.addToast(String(e), "error");
    } finally {
      associationsLoading = false;
    }
  }

  // Media players for default player picker
  let mediaPlayers = $state<MediaPlayer[]>([]);

  async function loadMediaPlayers() {
    try {
      mediaPlayers = await listMediaPlayers();
    } catch {
      // Not available on this platform
    }
  }

  async function pickMoveDestination() {
    const dir = await openDialog({ directory: true, multiple: false });
    if (dir) {
      await settingsState.updateAndSave({ default_move_destination: dir as string });
    }
  }

  onMount(() => {
    loadAssociations();
    loadMediaPlayers();
  });

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

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  role="dialog"
  aria-modal="true"
  aria-label="Settings"
  tabindex="-1"
  class="fixed inset-0 z-50 flex justify-end {closing ? 'backdrop-fade-out' : 'backdrop-fade-in'}"
  onclick={(e) => { if (e.target === e.currentTarget) close(); }}
  onkeydown={(e) => { if (e.key === "Escape") close(); }}
>
  <div
    class="{closing ? 'sidebar-slide-out' : 'sidebar-slide-in'} flex h-full w-full sm:max-w-md flex-col border-l border-[var(--color-border)] bg-[var(--color-bg)] shadow-2xl"
    onclick={(e) => e.stopPropagation()}
  >
    <!-- Header -->
    <div class="flex shrink-0 items-center justify-between border-b border-[var(--color-border)] px-5 py-4">
      <span class="text-lg font-bold text-[var(--color-text)]">Settings</span>
      <button
        onclick={close}
        class="ml-3 shrink-0 rounded-lg p-1.5 text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text)]"
      >
        <X class="h-5 w-5" />
      </button>
    </div>

    <!-- Scrollable content -->
    <div class="min-h-0 flex-1 overflow-y-auto p-4">
      <div class="flex flex-col gap-3">
      <!-- 1. Storage -->
      <div class="rounded-xl bg-[var(--color-bg-secondary)] p-4">
        <h3 class="mb-4 text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">Storage</h3>
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
              </button>
            </div>
            <p class="mt-1 text-xs text-[var(--color-text-muted)]">Files move to the download folder when done</p>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-sm text-[var(--color-text-secondary)]">Delete .torrent file after adding</span>
            <button
              onclick={() => handleToggle("delete_torrent_file_on_add")}
              class="relative h-6 w-11 rounded-full transition-colors {settingsState.settings.delete_torrent_file_on_add ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-bg-tertiary)]'}"
            >
              <span class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform shadow-sm {settingsState.settings.delete_torrent_file_on_add ? 'translate-x-5' : ''}"></span>
            </button>
          </div>
          <div class="flex items-center justify-between">
            <div>
              <span class="text-sm text-[var(--color-text-secondary)]">Watch folders for .torrent files</span>
              <p class="text-xs text-[var(--color-text-muted)]">New .torrent files will be auto-added</p>
            </div>
            <button
              onclick={() => handleToggle("watch_folders_enabled")}
              class="relative h-6 w-11 shrink-0 rounded-full transition-colors {settingsState.settings.watch_folders_enabled ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-bg-tertiary)]'}"
            >
              <span class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform shadow-sm {settingsState.settings.watch_folders_enabled ? 'translate-x-5' : ''}"></span>
            </button>
          </div>
          {#if settingsState.settings.watch_folders.length > 0}
            <div class="space-y-2">
              {#each settingsState.settings.watch_folders as folder}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div class="flex items-center gap-2 rounded-lg bg-[var(--color-bg-secondary)] px-3 py-2" oncontextmenu={(e) => watchCtx.open(e, folder)}>
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

      <!-- 2. Playback -->
      <div class="rounded-xl bg-[var(--color-bg-secondary)] p-4">
        <h3 class="mb-4 text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">Playback</h3>
        <div class="space-y-4">
          <div>
            <label for="default-cast" class="mb-1 flex items-center gap-1.5 text-sm text-[var(--color-text-secondary)]">
              <Cast class="h-3.5 w-3.5" />
              Default cast device
            </label>
            <select
              id="default-cast"
              class={fieldClass}
              value={settingsState.settings.default_cast_device}
              onchange={(e) => settingsState.updateAndSave({ default_cast_device: (e.target as HTMLSelectElement).value })}
            >
              <option value="">None</option>
              {#each devicesState.devices as device}
                <option value={device.id}>{device.name}</option>
              {/each}
            </select>
          </div>
          <div>
            <label for="default-player" class="mb-1 flex items-center gap-1.5 text-sm text-[var(--color-text-secondary)]">
              <MonitorPlay class="h-3.5 w-3.5" />
              Default media player
            </label>
            <select
              id="default-player"
              class={fieldClass}
              value={settingsState.settings.default_media_player}
              onchange={(e) => settingsState.updateAndSave({ default_media_player: (e.target as HTMLSelectElement).value })}
            >
              <option value="">None</option>
              {#each mediaPlayers as player}
                <option value={player.name}>{player.name}</option>
              {/each}
            </select>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-sm text-[var(--color-text-secondary)]">Auto-play next file</span>
            <button
              onclick={() => handleToggle("auto_play_next")}
              class="relative h-6 w-11 rounded-full transition-colors {settingsState.settings.auto_play_next ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-bg-tertiary)]'}"
            >
              <span class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform shadow-sm {settingsState.settings.auto_play_next ? 'translate-x-5' : ''}"></span>
            </button>
          </div>
          <div>
            <label for="default-move" class="mb-1 flex items-center gap-1.5 text-sm text-[var(--color-text-secondary)]">
              <FolderOutput class="h-3.5 w-3.5" />
              Default move destination
            </label>
            <div class="flex gap-2">
              <input
                id="default-move"
                type="text"
                readonly
                value={settingsState.settings.default_move_destination || "None"}
                class="flex-1 {fieldClass} {!settingsState.settings.default_move_destination ? 'text-[var(--color-text-muted)]' : ''}"
              />
              {#if settingsState.settings.default_move_destination}
                <button
                  onclick={() => settingsState.updateAndSave({ default_move_destination: "" })}
                  class="flex h-10 items-center gap-2 rounded-lg bg-[var(--color-bg-tertiary)] px-3 text-sm text-[var(--color-text-secondary)] hover:bg-[var(--color-border)]"
                >
                  <X class="h-4 w-4" />
                </button>
              {/if}
              <button
                onclick={pickMoveDestination}
                class="flex h-10 items-center gap-2 rounded-lg bg-[var(--color-bg-tertiary)] px-3 text-sm text-[var(--color-text-secondary)] hover:bg-[var(--color-border)]"
              >
                <Folder class="h-4 w-4" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- 3. Subtitles -->
      <div class="rounded-xl bg-[var(--color-bg-secondary)] p-4">
        <h3 class="mb-4 text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">Subtitles</h3>
        <div class="space-y-4">
          <div>
            <label for="opensub-key" class="mb-1 block text-sm text-[var(--color-text-secondary)]">OpenSubtitles API key</label>
            <div class="relative">
              <input
                id="opensub-key"
                type={showApiKey ? "text" : "password"}
                value={settingsState.settings.opensubtitles_api_key}
                onchange={handleApiKeyChange}
                class="{fieldClass} pr-10"
                placeholder="Paste your key from opensubtitles.com"
              />
              <button
                type="button"
                onclick={() => showApiKey = !showApiKey}
                class="absolute right-3 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)] hover:text-[var(--color-text-secondary)]"
              >
                {#if showApiKey}
                  <EyeOff class="h-4 w-4" />
                {:else}
                  <Eye class="h-4 w-4" />
                {/if}
              </button>
            </div>
          </div>
          <div>
            <label class="mb-2 block text-sm text-[var(--color-text-secondary)]">Subtitle languages</label>
            <div class="flex flex-wrap gap-1.5">
              {#each commonLanguages as lang}
                <button
                  onclick={() => toggleLanguage(lang.code)}
                  class="rounded-full px-2.5 py-1 text-xs transition-colors {settingsState.settings.subtitle_languages.includes(lang.code) ? 'bg-[var(--color-primary)] text-white' : 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)] hover:bg-[var(--color-border)]'}"
                >
                  {lang.name}
                </button>
              {/each}
            </div>
          </div>
        </div>
      </div>

      <!-- 4. Transfers -->
      <div class="rounded-xl bg-[var(--color-bg-secondary)] p-4">
        <h3 class="mb-4 text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">Transfers</h3>
        <div class="space-y-4">
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label for="max-dl" class="mb-1 block text-sm text-[var(--color-text-secondary)]">Max download speed</label>
              <div class="relative">
                <input
                  id="max-dl"
                  type="number"
                  min="0"
                  value={settingsState.settings.max_download_speed}
                  onchange={(e) => handleNumber("max_download_speed", e)}
                  class="{fieldClass} pr-14"
                  placeholder="0"
                />
                <span class="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-xs text-[var(--color-text-muted)]">KB/s</span>
              </div>
            </div>
            <div>
              <label for="max-ul" class="mb-1 block text-sm text-[var(--color-text-secondary)]">Max upload speed</label>
              <div class="relative">
                <input
                  id="max-ul"
                  type="number"
                  min="0"
                  value={settingsState.settings.max_upload_speed}
                  onchange={(e) => handleNumber("max_upload_speed", e)}
                  class="{fieldClass} pr-14"
                  placeholder="0"
                />
                <span class="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-xs text-[var(--color-text-muted)]">KB/s</span>
              </div>
            </div>
          </div>
          <p class="text-xs text-[var(--color-text-muted)]">0 = unlimited</p>
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
          <div class="flex items-center justify-between">
            <span class="text-sm text-[var(--color-text-secondary)]">Default for .torrent files</span>
            {#if associations.torrent_files}
              <span class="text-xs text-[var(--color-text-muted)]">Default</span>
            {:else}
              <button
                onclick={handleSetTorrentDefault}
                disabled={associationsLoading}
                class="rounded-lg bg-[var(--color-bg-tertiary)] px-3 py-1.5 text-xs text-[var(--color-text-secondary)] hover:bg-[var(--color-border)] disabled:opacity-50"
              >
                Make default
              </button>
            {/if}
          </div>
          <div class="flex items-center justify-between">
            <span class="text-sm text-[var(--color-text-secondary)]">Default for magnet links</span>
            {#if associations.magnet_links}
              <span class="text-xs text-[var(--color-text-muted)]">Default</span>
            {:else}
              <button
                onclick={handleSetMagnetDefault}
                disabled={associationsLoading}
                class="rounded-lg bg-[var(--color-bg-tertiary)] px-3 py-1.5 text-xs text-[var(--color-text-secondary)] hover:bg-[var(--color-border)] disabled:opacity-50"
              >
                Make default
              </button>
            {/if}
          </div>
        </div>
      </div>

      <!-- 5. Network -->
      <div class="rounded-xl bg-[var(--color-bg-secondary)] p-4">
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
            <p class="mt-1 text-xs text-[var(--color-text-muted)]">For BitTorrent connections. Restart required.</p>
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
            <p class="mt-1 text-xs text-[var(--color-text-muted)]">For casting to devices</p>
          </div>
          <div class="flex items-center justify-between">
            <div>
              <span class="text-sm text-[var(--color-text-secondary)]">UPnP port forwarding</span>
              <p class="text-xs text-[var(--color-text-muted)]">Auto-open port on your router</p>
            </div>
            <button
              onclick={() => handleToggle("enable_upnp")}
              class="relative h-6 w-11 shrink-0 rounded-full transition-colors {settingsState.settings.enable_upnp ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-bg-tertiary)]'}"
            >
              <span class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform shadow-sm {settingsState.settings.enable_upnp ? 'translate-x-5' : ''}"></span>
            </button>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-sm text-[var(--color-text-secondary)]">Find devices on your network</span>
            <button
              onclick={() => handleToggle("auto_discover")}
              class="relative h-6 w-11 rounded-full transition-colors {settingsState.settings.auto_discover ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-bg-tertiary)]'}"
            >
              <span class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform shadow-sm {settingsState.settings.auto_discover ? 'translate-x-5' : ''}"></span>
            </button>
          </div>
        </div>
      </div>

      <!-- 6. Automation -->
      <div class="rounded-xl bg-[var(--color-bg-secondary)] p-4">
        <h3 class="mb-4 text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">Automation</h3>
        <div class="space-y-4">
          <div>
            <label for="picker-countdown" class="mb-1 block text-sm text-[var(--color-text-secondary)]">Auto-approve delay (seconds)</label>
            <input
              id="picker-countdown"
              type="number"
              min="0"
              value={settingsState.settings.picker_countdown_seconds}
              onchange={(e) => handleNumber("picker_countdown_seconds", e)}
              class={fieldClass}
              placeholder="0 = disabled"
            />
            <p class="mt-1 text-xs text-[var(--color-text-muted)]">Countdown before auto-selecting the first option</p>
          </div>
          <div class="flex items-center justify-between">
            <div>
              <span class="text-sm text-[var(--color-text-secondary)]">Skip template picker</span>
              <p class="text-xs text-[var(--color-text-muted)]">Use default playlet for new torrents</p>
            </div>
            <button
              onclick={() => handleToggle("skip_template_picker")}
              class="relative h-6 w-11 shrink-0 rounded-full transition-colors {settingsState.settings.skip_template_picker ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-bg-tertiary)]'}"
            >
              <span class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform shadow-sm {settingsState.settings.skip_template_picker ? 'translate-x-5' : ''}"></span>
            </button>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-sm text-[var(--color-text-secondary)]">Show menu bar icon</span>
            <button
              onclick={() => handleToggle("show_tray_icon")}
              class="relative h-6 w-11 rounded-full transition-colors {settingsState.settings.show_tray_icon ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-bg-tertiary)]'}"
            >
              <span class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform shadow-sm {settingsState.settings.show_tray_icon ? 'translate-x-5' : ''}"></span>
            </button>
          </div>
        </div>
      </div>
      </div>
    </div>
  </div>
</div>

{#if dirCtx.state}
  <ContextMenu x={dirCtx.state.x} y={dirCtx.state.y} items={dirMenuItems(dirCtx.state.data.path, dirCtx.state.data.browse)} onclose={dirCtx.close} />
{/if}

{#if watchCtx.state}
  <ContextMenu x={watchCtx.state.x} y={watchCtx.state.y} items={watchFolderMenuItems(watchCtx.state.data)} onclose={watchCtx.close} />
{/if}

<style>
  .backdrop-fade-in {
    animation: backdropFadeIn 0.2s ease-out forwards;
  }
  .backdrop-fade-out {
    animation: backdropFadeOut 0.2s ease-in forwards;
  }
  .sidebar-slide-in {
    animation: sidebarSlideIn 0.2s ease-out forwards;
  }
  .sidebar-slide-out {
    animation: sidebarSlideOut 0.2s ease-in forwards;
  }

  @keyframes backdropFadeIn {
    from { background-color: transparent; }
    to { background-color: rgba(0, 0, 0, 0.3); }
  }
  @keyframes backdropFadeOut {
    from { background-color: rgba(0, 0, 0, 0.3); }
    to { background-color: transparent; }
  }
  @keyframes sidebarSlideIn {
    from { transform: translateX(100%); }
    to { transform: translateX(0); }
  }
  @keyframes sidebarSlideOut {
    from { transform: translateX(0); }
    to { transform: translateX(100%); }
  }
</style>
