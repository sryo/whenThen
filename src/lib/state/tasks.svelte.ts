import { load } from "@tauri-apps/plugin-store";
import type { Task, TaskStatus, ActionResult, ActionExecutionStatus } from "$lib/types/task";
import type { ActionType } from "$lib/types/playlet";
import { playletsState } from "./playlets.svelte";

let tasks = $state<Task[]>([]);

let storeInstance: Awaited<ReturnType<typeof load>> | null = null;

async function getStore() {
  if (!storeInstance) {
    storeInstance = await load("tasks.json", { autoSave: true, defaults: {} });
  }
  return storeInstance;
}

async function persistTasks() {
  try {
    const store = await getStore();
    await store.set("tasks", tasks);
  } catch {}

}

export const tasksState = {
  get tasks() {
    return tasks;
  },
  get activeTasks() {
    return tasks.filter((t) => t.status === "waiting" || t.status === "executing");
  },
  get completedTasks() {
    return tasks.filter((t) => t.status === "completed" || t.status === "failed");
  },

  async loadTasks() {
    try {
      const store = await getStore();
      const saved = await store.get<Task[]>("tasks");
      if (saved && Array.isArray(saved)) {
        // Reset any "executing" tasks to "waiting" (crashed mid-run)
        tasks = saved.map((t) => {
          if (t.status === "executing") {
            return {
              ...t,
              status: "waiting" as TaskStatus,
              actionResults: t.actionResults.map((ar) =>
                ar.status === "running"
                  ? { ...ar, status: "pending" as ActionExecutionStatus }
                  : ar,
              ),
            };
          }
          return t;
        });
        return;
      }
    } catch {}
    tasks = [];
  },

  createTask(
    torrentId: number,
    torrentName: string,
    playletId: string | null,
    playletName: string | null,
  ): Task {
    const playlet = playletId ? playletsState.getById(playletId) : null;
    const actionResults: ActionResult[] = playlet
      ? playlet.actions.map((a) => ({
          actionId: a.id,
          actionType: a.type as ActionType,
          status: "pending" as ActionExecutionStatus,
          startedAt: null,
          completedAt: null,
          error: null,
          skipReason: null,
        }))
      : [];

    const task: Task = {
      id: crypto.randomUUID(),
      torrentId,
      torrentName,
      playletId,
      playletName,
      status: "waiting",
      actionResults,
      createdAt: new Date().toISOString(),
      completedAt: null,
    };
    tasks = [task, ...tasks];
    persistTasks();
    return task;
  },

  getByTorrentId(torrentId: number): Task | undefined {
    return tasks.find((t) => t.torrentId === torrentId && (t.status === "waiting" || t.status === "executing"));
  },

  getById(id: string): Task | undefined {
    return tasks.find((t) => t.id === id);
  },

  setTaskStatus(taskId: string, status: TaskStatus) {
    const idx = tasks.findIndex((t) => t.id === taskId);
    if (idx < 0) return;
    tasks[idx] = {
      ...tasks[idx],
      status,
      completedAt: status === "completed" || status === "failed" ? new Date().toISOString() : tasks[idx].completedAt,
    };
    persistTasks();
  },

  markActionRunning(taskId: string, actionId: string) {
    const tIdx = tasks.findIndex((t) => t.id === taskId);
    if (tIdx < 0) return;
    const results = tasks[tIdx].actionResults.map((ar) =>
      ar.actionId === actionId
        ? { ...ar, status: "running" as ActionExecutionStatus, startedAt: new Date().toISOString() }
        : ar,
    );
    tasks[tIdx] = { ...tasks[tIdx], actionResults: results };
    persistTasks();
  },

  markActionDone(taskId: string, actionId: string) {
    const tIdx = tasks.findIndex((t) => t.id === taskId);
    if (tIdx < 0) return;
    const results = tasks[tIdx].actionResults.map((ar) =>
      ar.actionId === actionId
        ? { ...ar, status: "done" as ActionExecutionStatus, completedAt: new Date().toISOString(), error: null, skipReason: null }
        : ar,
    );
    tasks[tIdx] = { ...tasks[tIdx], actionResults: results };
    persistTasks();
  },

  markActionFailed(taskId: string, actionId: string, error: string) {
    const tIdx = tasks.findIndex((t) => t.id === taskId);
    if (tIdx < 0) return;
    const results = tasks[tIdx].actionResults.map((ar) =>
      ar.actionId === actionId
        ? { ...ar, status: "failed" as ActionExecutionStatus, completedAt: new Date().toISOString(), error, skipReason: null }
        : ar,
    );
    tasks[tIdx] = { ...tasks[tIdx], actionResults: results };
    persistTasks();
  },

  markActionSkipped(taskId: string, actionId: string, reason?: string) {
    const tIdx = tasks.findIndex((t) => t.id === taskId);
    if (tIdx < 0) return;
    const results = tasks[tIdx].actionResults.map((ar) =>
      ar.actionId === actionId
        ? { ...ar, status: "skipped" as ActionExecutionStatus, completedAt: new Date().toISOString(), skipReason: reason ?? null }
        : ar,
    );
    tasks[tIdx] = { ...tasks[tIdx], actionResults: results };
    persistTasks();
  },

  removeTask(id: string) {
    tasks = tasks.filter((t) => t.id !== id);
    persistTasks();
  },

  clearCompleted() {
    tasks = tasks.filter((t) => t.status !== "completed" && t.status !== "failed");
    persistTasks();
  },

  reassignTask(taskId: string, newPlayletId: string) {
    const tIdx = tasks.findIndex((t) => t.id === taskId);
    if (tIdx < 0) return;
    if (tasks[tIdx].status !== "waiting") return;

    const playlet = playletsState.getById(newPlayletId);
    if (!playlet) return;

    const actionResults: ActionResult[] = playlet.actions.map((a) => ({
      actionId: a.id,
      actionType: a.type as ActionType,
      status: "pending" as ActionExecutionStatus,
      startedAt: null,
      completedAt: null,
      error: null,
      skipReason: null,
    }));

    tasks[tIdx] = {
      ...tasks[tIdx],
      playletId: newPlayletId,
      playletName: null,
      actionResults,
    };
    persistTasks();
  },

  clearTaskPlaylet(taskId: string) {
    const tIdx = tasks.findIndex((t) => t.id === taskId);
    if (tIdx < 0) return;
    if (tasks[tIdx].status !== "waiting") return;

    tasks[tIdx] = {
      ...tasks[tIdx],
      playletId: null,
      playletName: null,
      actionResults: [],
    };
    persistTasks();
  },

  updateTorrentId(oldTorrentId: number, newTorrentId: number) {
    const idx = tasks.findIndex((t) => t.torrentId === oldTorrentId && (t.status === "waiting" || t.status === "executing"));
    if (idx < 0) return;
    tasks[idx] = { ...tasks[idx], torrentId: newTorrentId };
    persistTasks();
  },

  resetTaskForRetry(taskId: string) {
    const tIdx = tasks.findIndex((t) => t.id === taskId);
    if (tIdx < 0) return;
    const results = tasks[tIdx].actionResults.map((ar) =>
      ar.status === "failed" || ar.status === "skipped"
        ? { ...ar, status: "pending" as ActionExecutionStatus, startedAt: null, completedAt: null, error: null, skipReason: null }
        : ar,
    );
    tasks[tIdx] = { ...tasks[tIdx], status: "waiting", actionResults: results, completedAt: null };
    persistTasks();
  },

  // Fail any active tasks for a torrent that was deleted
  failTasksForTorrent(torrentId: number) {
    let changed = false;
    tasks = tasks.map((t) => {
      if (t.torrentId === torrentId && (t.status === "waiting" || t.status === "executing")) {
        changed = true;
        return {
          ...t,
          status: "failed" as TaskStatus,
          completedAt: new Date().toISOString(),
        };
      }
      return t;
    });
    if (changed) persistTasks();
  },

  // Create a failed task for a magnet that couldn't fetch metadata
  createFailedMagnetTask(torrentName: string, infoHash: string, error: string): Task {
    const task: Task = {
      id: crypto.randomUUID(),
      torrentId: -1,
      torrentName,
      playletId: null,
      playletName: null,
      status: "failed",
      actionResults: [{
        actionId: "metadata-fetch",
        actionType: "metadata" as ActionType,
        status: "failed",
        startedAt: new Date().toISOString(),
        completedAt: new Date().toISOString(),
        error,
        skipReason: null,
      }],
      createdAt: new Date().toISOString(),
      completedAt: new Date().toISOString(),
    };
    tasks = [task, ...tasks];
    persistTasks();
    return task;
  },

  // Remove orphaned tasks that reference non-existent torrents
  reconcileWithTorrents(validTorrentIds: Set<number>) {
    let changed = false;
    tasks = tasks.map((t) => {
      if ((t.status === "waiting" || t.status === "executing") && !validTorrentIds.has(t.torrentId)) {
        changed = true;
        return {
          ...t,
          status: "failed" as TaskStatus,
          completedAt: new Date().toISOString(),
        };
      }
      return t;
    });
    if (changed) persistTasks();
  },
};
