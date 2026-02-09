<!-- App preferences and configuration. -->
<script lang="ts">
  import { Folder, X, Clipboard, FolderOpen, Trash2, Cast, MonitorPlay, FolderOutput, Eye, EyeOff, Globe, Check } from "lucide-svelte";
  import { settingsState } from "$lib/state/settings.svelte";
  import { uiState } from "$lib/state/ui.svelte";
  import { i18n } from "$lib/i18n/state.svelte";
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

  let savedRecently = $state(false);
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;

  function showSaved() {
    savedRecently = true;
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => { savedRecently = false; }, 1500);
  }

  async function pickDirectory() {
    const dir = await openDialog({ directory: true, multiple: false });
    if (dir) {
      await settingsState.updateAndSave({ download_directory: dir as string });
      showSaved();
    }
  }

  async function pickIncompleteDirectory() {
    const dir = await openDialog({ directory: true, multiple: false });
    if (dir) {
      await settingsState.updateAndSave({ incomplete_directory: dir as string });
      showSaved();
    }
  }

  function clearIncompleteDirectory() {
    settingsState.updateAndSave({ incomplete_directory: "" });
    showSaved();
  }

  function handleToggle(key: "auto_discover" | "enable_upnp" | "watch_folders_enabled" | "auto_play_next" | "delete_torrent_file_on_add" | "show_tray_icon" | "skip_template_picker") {
    settingsState.updateAndSave({ [key]: !settingsState.settings[key] });
    showSaved();
  }

  function handleNumber(key: "max_download_speed" | "max_upload_speed" | "media_server_port" | "listen_port" | "max_concurrent_tasks" | "picker_countdown_seconds" | "rss_check_interval_minutes" | "metadata_timeout_secs", e: Event) {
    const value = parseInt((e.target as HTMLInputElement).value) || 0;
    settingsState.updateAndSave({ [key]: value });
    showSaved();
  }

  const fieldClass = "h-10 w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] px-3 text-sm text-[var(--color-text)] focus:border-[var(--color-primary)] focus:outline-none focus:ring-1 focus:ring-[var(--color-primary)]";

  const appLanguageCodes = ["system", "en", "es"];

  async function handleLocaleChange(e: Event) {
    const value = (e.target as HTMLSelectElement).value;
    await settingsState.updateAndSave({ locale: value });
    await i18n.setLocale(value);
    showSaved();
  }

  let subtitleLangInput = $state(settingsState.settings.subtitle_languages.join(", "));

  function handleSubtitleLangChange(value: string) {
    subtitleLangInput = value;
    const langs = value.split(",").map((s) => s.trim().toLowerCase()).filter(Boolean);
    settingsState.updateAndSave({ subtitle_languages: langs });
    showSaved();
  }

  async function addWatchFolder() {
    const dir = await openDialog({ directory: true, multiple: false });
    if (dir) {
      const folders = [...settingsState.settings.watch_folders];
      if (!folders.includes(dir as string)) {
        folders.push(dir as string);
        await settingsState.updateAndSave({ watch_folders: folders });
        showSaved();
      }
    }
  }

  function removeWatchFolder(folder: string) {
    const folders = settingsState.settings.watch_folders.filter((f) => f !== folder);
    settingsState.updateAndSave({ watch_folders: folders });
    showSaved();
  }

  let showApiKey = $state(false);

  function handleApiKeyChange(e: Event) {
    const value = (e.target as HTMLInputElement).value;
    settingsState.updateAndSave({ opensubtitles_api_key: value });
    showSaved();
  }

  let associations = $state<FileAssociationStatus>({ torrent_files: false, magnet_links: false });
  let associationsLoading = $state(false);

  async function loadAssociations() {
    try {
      associations = await checkFileAssociations();
    } catch {}
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

  let mediaPlayers = $state<MediaPlayer[]>([]);

  const currentCastDevice = $derived(devicesState.devices.find(d => d.id === settingsState.settings.default_cast_device));
  const otherCastDevices = $derived(devicesState.devices.filter(d => d.id !== settingsState.settings.default_cast_device));

  async function loadMediaPlayers() {
    try {
      mediaPlayers = await listMediaPlayers();
    } catch {}
  }

  async function pickMoveDestination() {
    const dir = await openDialog({ directory: true, multiple: false });
    if (dir) {
      await settingsState.updateAndSave({ default_move_destination: dir as string });
      showSaved();
    }
  }

  onMount(() => {
    loadAssociations();
    loadMediaPlayers();
  });

  const dirCtx = useContextMenu<{ path: string; browse: () => void }>();
  const watchCtx = useContextMenu<string>();

  function dirMenuItems(path: string, browse: () => void): ContextMenuEntry[] {
    return [
      {
        icon: Clipboard,
        label: i18n.t("common.copyPath"),
        disabled: !path,
        action: () => {
          navigator.clipboard.writeText(path);
          uiState.addToast(i18n.t("common.copiedToClipboard"), "success");
        },
      },
      {
        icon: FolderOpen,
        label: i18n.t("common.revealInFinder"),
        disabled: !path,
        action: () => openShell(path),
      },
      { type: "divider" },
      {
        icon: Folder,
        label: i18n.t("common.browse"),
        action: browse,
      },
    ];
  }

  function watchFolderMenuItems(folder: string): ContextMenuEntry[] {
    return [
      {
        icon: Clipboard,
        label: i18n.t("common.copyPath"),
        action: () => {
          navigator.clipboard.writeText(folder);
          uiState.addToast(i18n.t("common.copiedToClipboard"), "success");
        },
      },
      {
        icon: FolderOpen,
        label: i18n.t("common.revealInFinder"),
        action: () => openShell(folder),
      },
      { type: "divider" },
      {
        icon: Trash2,
        label: i18n.t("common.remove"),
        danger: true,
        action: () => removeWatchFolder(folder),
      },
    ];
  }
</script>

<div class="mx-auto max-w-2xl p-6">
  <div class="flex flex-col gap-4">
    <!-- Language -->
    <div class="rounded-xl bg-[var(--color-bg-secondary)] p-4">
      <div class="mb-4 flex items-center gap-2">
        <h3 class="text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">{i18n.t("settings.language")}</h3>
        {#if savedRecently}
          {#key Date.now()}
            <span class="flex items-center gap-1 text-xs text-[var(--color-success)] animate-fade-in-out">
              <Check class="h-3 w-3" />
              {i18n.t("common.saved")}
            </span>
          {/key}
        {/if}
      </div>
      <div class="space-y-4">
        <div>
          <label for="app-locale" class="mb-1 flex items-center gap-1.5 text-sm text-[var(--color-text-secondary)]">
            <Globe class="h-3.5 w-3.5" />
            {i18n.t("settings.appLanguage")}
          </label>
          <select
            id="app-locale"
            class={fieldClass}
            value={settingsState.settings.locale}
            onchange={handleLocaleChange}
          >
            {#each appLanguageCodes as code}
              <option value={code}>{i18n.t(`languages.${code}`)}</option>
            {/each}
          </select>
        </div>
      </div>
    </div>

    <!-- Files -->
    <div class="rounded-xl bg-[var(--color-bg-secondary)] p-4">
      <div class="mb-4 flex items-center gap-2">
        <h3 class="text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">{i18n.t("settings.files")}</h3>
        {#if savedRecently}
          {#key Date.now()}
            <span class="flex items-center gap-1 text-xs text-[var(--color-success)] animate-fade-in-out">
              <Check class="h-3 w-3" />
              {i18n.t("common.saved")}
            </span>
          {/key}
        {/if}
      </div>
      <div class="space-y-4">
        <div>
          <label for="download-dir" class="mb-1 block text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.downloadFolder")}</label>
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
          <label for="incomplete-dir" class="mb-1 block text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.partialDownloads")}</label>
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="flex gap-2" oncontextmenu={(e) => dirCtx.open(e, { path: settingsState.settings.incomplete_directory, browse: pickIncompleteDirectory })}>
            <input
              id="incomplete-dir"
              type="text"
              readonly
              value={settingsState.settings.incomplete_directory || i18n.t("settings.useDownloadFolder")}
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
          <p class="mt-1 text-xs text-[var(--color-text-muted)]">{i18n.t("settings.movesWhenComplete")}</p>
        </div>
        <div class="flex items-center justify-between">
          <span class="text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.deleteTorrentFiles")}</span>
          <button
            onclick={() => handleToggle("delete_torrent_file_on_add")}
            class="relative h-6 w-11 rounded-full transition-colors {settingsState.settings.delete_torrent_file_on_add ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-bg-tertiary)]'}"
          >
            <span class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform shadow-sm {settingsState.settings.delete_torrent_file_on_add ? 'translate-x-5' : ''}"></span>
          </button>
        </div>
        <div class="flex items-center justify-between">
          <div>
            <span class="text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.autoImportFolders")}</span>
            <p class="text-xs text-[var(--color-text-muted)]">{i18n.t("settings.watchesForTorrentFiles")}</p>
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
          {i18n.t("settings.addFolder")}
        </button>
      </div>
    </div>

    <!-- Playback -->
    <div class="rounded-xl bg-[var(--color-bg-secondary)] p-4 {uiState.highlightedSection === 'playback' ? 'section-highlight' : ''}">
      <div class="mb-4 flex items-center gap-2">
        <h3 class="text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">{i18n.t("settings.playback")}</h3>
        {#if savedRecently}
          {#key Date.now()}
            <span class="flex items-center gap-1 text-xs text-[var(--color-success)] animate-fade-in-out">
              <Check class="h-3 w-3" />
              {i18n.t("common.saved")}
            </span>
          {/key}
        {/if}
      </div>
      <div class="space-y-4">
        <div>
          <label for="default-cast" class="mb-1 flex items-center gap-1.5 text-sm text-[var(--color-text-secondary)]">
            <Cast class="h-3.5 w-3.5" />
            {i18n.t("settings.defaultCastDevice")}
          </label>
          <select
            id="default-cast"
            class={fieldClass}
            value={settingsState.settings.default_cast_device}
            onchange={(e) => { settingsState.updateAndSave({ default_cast_device: (e.target as HTMLSelectElement).value }); showSaved(); }}
          >
            <option value="">{i18n.t("common.none")}</option>
            {#if currentCastDevice}
              <optgroup label={i18n.t("settings.currentDefault")}>
                <option value={currentCastDevice.id}>{currentCastDevice.name}</option>
              </optgroup>
            {/if}
            {#if otherCastDevices.length > 0}
              <optgroup label={i18n.t("settings.availableDevices")}>
                {#each otherCastDevices as device}
                  <option value={device.id}>{device.name}</option>
                {/each}
              </optgroup>
            {/if}
          </select>
        </div>
        <div>
          <label for="default-player" class="mb-1 flex items-center gap-1.5 text-sm text-[var(--color-text-secondary)]">
            <MonitorPlay class="h-3.5 w-3.5" />
            {i18n.t("settings.defaultMediaPlayer")}
          </label>
          <select
            id="default-player"
            class={fieldClass}
            value={settingsState.settings.default_media_player}
            onchange={(e) => { settingsState.updateAndSave({ default_media_player: (e.target as HTMLSelectElement).value }); showSaved(); }}
          >
            <option value="">{i18n.t("common.none")}</option>
            {#each mediaPlayers as player}
              <option value={player.name}>{player.name}</option>
            {/each}
          </select>
        </div>
        <div class="flex items-center justify-between">
          <span class="text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.autoPlayNext")}</span>
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
            {i18n.t("settings.defaultMoveDestination")}
          </label>
          <div class="flex gap-2">
            <input
              id="default-move"
              type="text"
              readonly
              value={settingsState.settings.default_move_destination || i18n.t("settings.notSet")}
              class="flex-1 {fieldClass} {!settingsState.settings.default_move_destination ? 'text-[var(--color-text-muted)]' : ''}"
            />
            {#if settingsState.settings.default_move_destination}
              <button
                onclick={() => { settingsState.updateAndSave({ default_move_destination: "" }); showSaved(); }}
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

    <!-- Subtitles -->
    <div class="rounded-xl bg-[var(--color-bg-secondary)] p-4 {uiState.highlightedSection === 'subtitles' ? 'section-highlight' : ''}">
      <div class="mb-4 flex items-center gap-2">
        <h3 class="text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">{i18n.t("settings.subtitles")}</h3>
        {#if savedRecently}
          {#key Date.now()}
            <span class="flex items-center gap-1 text-xs text-[var(--color-success)] animate-fade-in-out">
              <Check class="h-3 w-3" />
              {i18n.t("common.saved")}
            </span>
          {/key}
        {/if}
      </div>
      <div class="space-y-4">
        <div>
          <label for="opensub-key" class="mb-1 block text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.openSubtitlesApiKey")}</label>
          <div class="relative">
            <input
              id="opensub-key"
              type={showApiKey ? "text" : "password"}
              value={settingsState.settings.opensubtitles_api_key}
              onchange={handleApiKeyChange}
              class="{fieldClass} pr-10"
              placeholder={i18n.t("settings.openSubtitlesPlaceholder")}
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
          <label for="subtitle-languages" class="mb-2 block text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.subtitleLanguages")}</label>
          <input
            id="subtitle-languages"
            type="text"
            value={subtitleLangInput}
            oninput={(e) => handleSubtitleLangChange((e.target as HTMLInputElement).value)}
            placeholder="en, es, de"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
            class={fieldClass}
          />
        </div>
      </div>
    </div>

    <!-- Downloads -->
    <div class="rounded-xl bg-[var(--color-bg-secondary)] p-4">
      <div class="mb-4 flex items-center gap-2">
        <h3 class="text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">{i18n.t("settings.downloads")}</h3>
        {#if savedRecently}
          {#key Date.now()}
            <span class="flex items-center gap-1 text-xs text-[var(--color-success)] animate-fade-in-out">
              <Check class="h-3 w-3" />
              {i18n.t("common.saved")}
            </span>
          {/key}
        {/if}
      </div>
      <div class="space-y-4">
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label for="max-dl" class="mb-1 block text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.downloadLimit")}</label>
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
              <span class="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-xs text-[var(--color-text-muted)]">{i18n.t("settings.kbs")}</span>
            </div>
          </div>
          <div>
            <label for="max-ul" class="mb-1 block text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.uploadLimit")}</label>
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
              <span class="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-xs text-[var(--color-text-muted)]">{i18n.t("settings.kbs")}</span>
            </div>
          </div>
        </div>
        <p class="text-xs text-[var(--color-text-muted)]">{i18n.t("settings.leaveEmptyUnlimited")}</p>
        <div>
          <label for="max-tasks" class="mb-1 block text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.simultaneousDownloads")}</label>
          <input
            id="max-tasks"
            type="number"
            min="0"
            value={settingsState.settings.max_concurrent_tasks}
            onchange={(e) => handleNumber("max_concurrent_tasks", e)}
            class={fieldClass}
            placeholder={i18n.t("settings.noLimit")}
          />
        </div>
        <div class="flex items-center justify-between">
          <span class="text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.handleTorrentFiles")}</span>
          {#if associations.torrent_files}
            <span class="text-xs text-[var(--color-text-muted)]">{i18n.t("common.default")}</span>
          {:else}
            <button
              onclick={handleSetTorrentDefault}
              disabled={associationsLoading}
              class="rounded-lg bg-[var(--color-bg-tertiary)] px-3 py-1.5 text-xs text-[var(--color-text-secondary)] hover:bg-[var(--color-border)] disabled:opacity-50"
            >
              {i18n.t("settings.makeDefault")}
            </button>
          {/if}
        </div>
        <div class="flex items-center justify-between">
          <span class="text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.handleMagnetLinks")}</span>
          {#if associations.magnet_links}
            <span class="text-xs text-[var(--color-text-muted)]">{i18n.t("common.default")}</span>
          {:else}
            <button
              onclick={handleSetMagnetDefault}
              disabled={associationsLoading}
              class="rounded-lg bg-[var(--color-bg-tertiary)] px-3 py-1.5 text-xs text-[var(--color-text-secondary)] hover:bg-[var(--color-border)] disabled:opacity-50"
            >
              {i18n.t("settings.makeDefault")}
            </button>
          {/if}
        </div>
      </div>
    </div>

    <!-- Network -->
    <div class="rounded-xl bg-[var(--color-bg-secondary)] p-4 {uiState.highlightedSection === 'network' ? 'section-highlight' : ''}">
      <div class="mb-4 flex items-center gap-2">
        <h3 class="text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">{i18n.t("settings.network")}</h3>
        {#if savedRecently}
          {#key Date.now()}
            <span class="flex items-center gap-1 text-xs text-[var(--color-success)] animate-fade-in-out">
              <Check class="h-3 w-3" />
              {i18n.t("common.saved")}
            </span>
          {/key}
        {/if}
      </div>
      <div class="space-y-4">
        <div>
          <label for="listen-port" class="mb-1 block text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.peerPort")}</label>
          <input
            id="listen-port"
            type="number"
            min="1024"
            max="65535"
            value={settingsState.settings.listen_port}
            onchange={(e) => handleNumber("listen_port", e)}
            class={fieldClass}
          />
          <p class="mt-1 text-xs text-[var(--color-text-muted)]">{i18n.t("settings.restartToApply")}</p>
        </div>
        <div>
          <label for="media-port" class="mb-1 block text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.castServerPort")}</label>
          <input
            id="media-port"
            type="number"
            min="1024"
            max="65535"
            value={settingsState.settings.media_server_port}
            onchange={(e) => handleNumber("media_server_port", e)}
            class={fieldClass}
          />
          <p class="mt-1 text-xs text-[var(--color-text-muted)]">{i18n.t("settings.mediaStreamsPort")}</p>
        </div>
        <div class="flex items-center justify-between">
          <div>
            <span class="text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.upnp")}</span>
            <p class="text-xs text-[var(--color-text-muted)]">{i18n.t("settings.upnpDescription")}</p>
          </div>
          <button
            onclick={() => handleToggle("enable_upnp")}
            class="relative h-6 w-11 shrink-0 rounded-full transition-colors {settingsState.settings.enable_upnp ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-bg-tertiary)]'}"
          >
            <span class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform shadow-sm {settingsState.settings.enable_upnp ? 'translate-x-5' : ''}"></span>
          </button>
        </div>
        <div class="flex items-center justify-between">
          <span class="text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.discoverChromecast")}</span>
          <button
            onclick={() => handleToggle("auto_discover")}
            class="relative h-6 w-11 rounded-full transition-colors {settingsState.settings.auto_discover ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-bg-tertiary)]'}"
          >
            <span class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform shadow-sm {settingsState.settings.auto_discover ? 'translate-x-5' : ''}"></span>
          </button>
        </div>
        <div>
          <label for="metadata-timeout" class="mb-1 block text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.metadataTimeout")}</label>
          <input
            id="metadata-timeout"
            type="number"
            min="5"
            max="120"
            value={settingsState.settings.metadata_timeout_secs}
            onchange={(e) => handleNumber("metadata_timeout_secs", e)}
            class={fieldClass}
          />
          <p class="mt-1 text-xs text-[var(--color-text-muted)]">{i18n.t("settings.metadataTimeoutDescription")}</p>
        </div>
      </div>
    </div>

    <!-- Behavior -->
    <div class="rounded-xl bg-[var(--color-bg-secondary)] p-4">
      <div class="mb-4 flex items-center gap-2">
        <h3 class="text-xs font-medium uppercase tracking-wide text-[var(--color-text-muted)]">{i18n.t("settings.behavior")}</h3>
        {#if savedRecently}
          {#key Date.now()}
            <span class="flex items-center gap-1 text-xs text-[var(--color-success)] animate-fade-in-out">
              <Check class="h-3 w-3" />
              {i18n.t("common.saved")}
            </span>
          {/key}
        {/if}
      </div>
      <div class="space-y-4">
        <div>
          <label for="rss-interval" class="mb-1 block text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.checkFeedsEvery")}</label>
          <input
            id="rss-interval"
            type="number"
            min="1"
            max="120"
            value={settingsState.settings.rss_check_interval_minutes}
            onchange={(e) => handleNumber("rss_check_interval_minutes", e)}
            class={fieldClass}
          />
          <p class="mt-1 text-xs text-[var(--color-text-muted)]">{i18n.t("settings.minutesBetweenRss")}</p>
        </div>
        <div>
          <label for="picker-countdown" class="mb-1 block text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.autoApproveAfter")}</label>
          <input
            id="picker-countdown"
            type="number"
            min="0"
            value={settingsState.settings.picker_countdown_seconds}
            onchange={(e) => handleNumber("picker_countdown_seconds", e)}
            class={fieldClass}
            placeholder={i18n.t("common.disabled")}
          />
          <p class="mt-1 text-xs text-[var(--color-text-muted)]">{i18n.t("settings.autoApproveDescription")}</p>
        </div>
        <div class="flex items-center justify-between">
          <div>
            <span class="text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.alwaysCreateBlankPlaylet")}</span>
            <p class="text-xs text-[var(--color-text-muted)]">{i18n.t("settings.skipTemplateSelection")}</p>
          </div>
          <button
            onclick={() => handleToggle("skip_template_picker")}
            class="relative h-6 w-11 shrink-0 rounded-full transition-colors {settingsState.settings.skip_template_picker ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-bg-tertiary)]'}"
          >
            <span class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform shadow-sm {settingsState.settings.skip_template_picker ? 'translate-x-5' : ''}"></span>
          </button>
        </div>
        <div class="flex items-center justify-between">
          <span class="text-sm text-[var(--color-text-secondary)]">{i18n.t("settings.menuBarIcon")}</span>
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

{#if dirCtx.state}
  <ContextMenu x={dirCtx.state.x} y={dirCtx.state.y} items={dirMenuItems(dirCtx.state.data.path, dirCtx.state.data.browse)} onclose={dirCtx.close} />
{/if}

{#if watchCtx.state}
  <ContextMenu x={watchCtx.state.x} y={watchCtx.state.y} items={watchFolderMenuItems(watchCtx.state.data)} onclose={watchCtx.close} />
{/if}
