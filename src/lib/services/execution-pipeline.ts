import { tasksState } from "$lib/state/tasks.svelte";
import { playletsState } from "$lib/state/playlets.svelte";
import { playbackState } from "$lib/state/playback.svelte";
import { devicesState } from "$lib/state/devices.svelte";
import { settingsState } from "$lib/state/settings.svelte";
import { uiState } from "$lib/state/ui.svelte";
import { t } from "$lib/i18n";
import type {
  CastAction,
  MoveAction,
  PlayAction,
  SubtitleAction,
  AutomationAction,
  DelayAction,
  WebhookAction,
  DeleteSourceAction,
} from "$lib/types/playlet";
import type { TorrentFileInfo } from "$lib/types/torrent";
import { cleanTorrentName } from "./torrent-name-cleaner";
import {
  playbackCastTorrent,
  moveTorrentFiles,
  playbackOpenInApp,
  torrentFiles,
  subtitleSearchOpensubtitles,
  runShortcut,
  runApplescript,
  runShellCommand,
  torrentDelete,
  checkAutomationPermission,
  getPlaylistUrl,
} from "./tauri-commands";
import { registerExecutor, getExecutor } from "./execution-registry";

class SkipError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "SkipError";
  }
}

// Register all built-in executors

registerExecutor("cast", async (action, torrentId, torrentName, files) => {
  const castAction = action as CastAction;
  let deviceId = castAction.deviceId;

  if (!deviceId) {
    const fallback = settingsState.settings.default_cast_device;
    if (fallback) {
      deviceId = fallback;
    } else {
      const connected = devicesState.connectedDevices;
      if (connected.length > 0) {
        deviceId = connected[0].id;
      } else {
        throw new SkipError("No cast device");
      }
    }
  }

  const playable = files.find((f) => f.is_playable);
  if (!playable) throw new Error("No playable files");

  await playbackCastTorrent(deviceId, torrentId, playable.index);
  const device = devicesState.devices.find((d) => d.id === deviceId);
  playbackState.setContext(torrentName, device?.name ?? deviceId, torrentId, playable.index);
});

registerExecutor("move", async (action, torrentId) => {
  const moveAction = action as MoveAction;
  const destination = moveAction.destination || settingsState.settings.default_move_destination;
  if (!destination) throw new SkipError("No destination folder");
  await moveTorrentFiles(torrentId, destination);
});

registerExecutor("notify", async (_action, _torrentId, torrentName) => {
  const msg = t("toast.torrentDone", { name: torrentName });
  if ("Notification" in window) {
    if (Notification.permission === "granted") {
      new Notification("whenThen", { body: msg });
    } else if (Notification.permission !== "denied") {
      const permission = await Notification.requestPermission();
      if (permission === "granted") {
        new Notification("whenThen", { body: msg });
      }
    }
  }
  uiState.addToast(msg, "success");
});

registerExecutor("play", async (action, torrentId, _torrentName, files) => {
  const playAction = action as PlayAction;
  const app = playAction.app || settingsState.settings.default_media_player;
  if (!app) throw new SkipError("No media player");

  if (playAction.usePlaylist) {
    const playlistUrl = await getPlaylistUrl(torrentId);
    await runShellCommand(`open -a "${app}" "${playlistUrl}"`);
  } else {
    const playable = files.find((f) => f.is_playable);
    if (!playable) throw new Error("No playable files");
    await playbackOpenInApp(torrentId, playable.index, app);
  }
});

registerExecutor("subtitle", async (action, torrentId, _torrentName, files) => {
  const subAction = action as SubtitleAction;
  const languages = subAction.languages.length > 0
    ? subAction.languages
    : settingsState.settings.subtitle_languages;

  const videoFiles = files.filter((f) => f.mime_type?.startsWith("video/"));
  if (videoFiles.length === 0) {
    throw new Error("No video files to add subtitles to");
  }

  // Fetch subtitles for each video file sequentially to respect API rate limits
  for (const file of videoFiles) {
    await subtitleSearchOpensubtitles(torrentId, file.index, languages);
  }
});

registerExecutor("automation", async (action, torrentId, torrentName, files) => {
  const autoAction = action as AutomationAction;

  const payload = JSON.stringify({
    torrentName,
    torrentId,
    files: files.map((f) => ({
      name: f.name,
      path: f.path,
      size: f.length,
      mime_type: f.mime_type,
    })),
    downloadDir: settingsState.downloadDirectory,
  });

  switch (autoAction.method) {
    case "shortcut": {
      if (!autoAction.shortcutName) throw new Error("No shortcut name set");
      await runShortcut(autoAction.shortcutName, payload);
      break;
    }
    case "applescript": {
      if (!autoAction.script) throw new Error("No AppleScript set");
      await checkAutomationPermission();
      // Escape order matters: backslashes first, then quotes, then newlines
      const escape = (s: string) =>
        s.replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\\n").replace(/\r/g, "\\r");
      const cleanName = cleanTorrentName(torrentName);
      const preamble = [
        `set torrentName to "${escape(torrentName)}"`,
        `set cleanName to "${escape(cleanName)}"`,
        `set downloadDir to "${escape(settingsState.downloadDirectory)}"`,
      ].join("\n");
      await runApplescript(preamble + "\n" + autoAction.script);
      break;
    }
    case "shell": {
      if (!autoAction.script) throw new Error("No shell command set");
      await runShellCommand(autoAction.script);
      break;
    }
  }
});

registerExecutor("delay", async (action) => {
  const delayAction = action as DelayAction;
  // "months" = 30 days (simplified)
  const multipliers: Record<string, number> = {
    seconds: 1, minutes: 60, days: 86400, weeks: 604800, months: 2592000,
  };
  const unit = delayAction.delayUnit ?? "seconds";
  const totalSeconds = (delayAction.seconds ?? 5) * (multipliers[unit] ?? 1);
  await new Promise((r) => setTimeout(r, totalSeconds * 1000));
});

registerExecutor("webhook", async (action, torrentId, torrentName, files) => {
  const webhookAction = action as WebhookAction;
  if (!webhookAction.url) throw new Error("No webhook URL set");

  const payload = {
    torrentName,
    torrentId,
    files: files.map((f) => ({
      name: f.name,
      path: f.path,
      size: f.length,
      mime_type: f.mime_type,
    })),
    downloadDir: settingsState.downloadDirectory,
  };

  const fetchOptions: RequestInit = {
    method: webhookAction.method,
    headers: { "Content-Type": "application/json" },
  };

  if (webhookAction.method === "POST") {
    fetchOptions.body = JSON.stringify(payload);
  }

  const response = await fetch(webhookAction.url, fetchOptions);
  if (!response.ok) {
    throw new Error(`Webhook returned ${response.status}`);
  }
});

registerExecutor("delete_source", async (action, torrentId) => {
  const deleteAction = action as DeleteSourceAction;
  await torrentDelete(torrentId, deleteAction.deleteFiles);
});

// Tasks ready to execute, waiting for concurrency slot
const readyQueue: string[] = [];

/** Called when a torrent completes. Queues the task and drains if slots are open. */
export function tryExecuteNext(taskId: string): void {
  if (!readyQueue.includes(taskId)) readyQueue.push(taskId);
  drainQueue();
}

function drainQueue(): void {
  const limit = settingsState.settings.max_concurrent_tasks;
  const executing = tasksState.tasks.filter((t) => t.status === "executing").length;
  const available = limit > 0 ? limit - executing : Infinity;

  for (let i = 0; i < available && readyQueue.length > 0; i++) {
    const taskId = readyQueue.shift()!;
    executePipeline(taskId).then(() => drainQueue());
  }
}

export async function executePipeline(taskId: string): Promise<void> {
  const task = tasksState.getById(taskId);
  if (!task) return;

  const playlet = task.playletId ? playletsState.getById(task.playletId) : null;
  if (!playlet || task.actionResults.length === 0) {
    tasksState.setTaskStatus(taskId, "completed");
    return;
  }

  tasksState.setTaskStatus(taskId, "executing");

  // Fetch files once and apply file filter
  let filteredFiles: TorrentFileInfo[];
  try {
    const allFiles = await torrentFiles(task.torrentId);
    filteredFiles = playletsState.filterFiles(playlet, allFiles);
  } catch {
    filteredFiles = [];
  }

  let hasFailed = false;

  for (const actionResult of task.actionResults) {
    if (actionResult.status === "done") continue;

    const action = playlet.actions.find((a) => a.id === actionResult.actionId);
    if (!action) {
      tasksState.markActionSkipped(taskId, actionResult.actionId);
      continue;
    }

    tasksState.markActionRunning(taskId, actionResult.actionId);

    try {
      const executor = getExecutor(action.type);
      if (!executor) throw new Error(`No executor for action type: ${action.type}`);
      await executor(action, task.torrentId, task.torrentName, filteredFiles);
      tasksState.markActionDone(taskId, actionResult.actionId);
    } catch (err: any) {
      if (err instanceof SkipError) {
        tasksState.markActionSkipped(taskId, actionResult.actionId, err.message);
      } else {
        const errorMsg = err?.message || String(err);
        tasksState.markActionFailed(taskId, actionResult.actionId, errorMsg);
        hasFailed = true;
      }
    }
  }

  tasksState.setTaskStatus(taskId, hasFailed ? "failed" : "completed");
}

export async function retryTask(taskId: string): Promise<void> {
  tasksState.resetTaskForRetry(taskId);
  await executePipeline(taskId);
}
