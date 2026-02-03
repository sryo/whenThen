import type { ActionType } from "./playlet";

export type ActionExecutionStatus = "pending" | "running" | "done" | "failed" | "skipped";
export type TaskStatus = "waiting" | "executing" | "completed" | "failed";

export interface ActionResult {
  actionId: string;
  actionType: ActionType;
  status: ActionExecutionStatus;
  startedAt: string | null;
  completedAt: string | null;
  error: string | null;
}

export interface Task {
  id: string;
  torrentId: number;
  torrentName: string;
  playletId: string | null;
  playletName: string | null;
  status: TaskStatus;
  actionResults: ActionResult[];
  createdAt: string;
  completedAt: string | null;
}
