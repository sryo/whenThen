import type { Action } from "$lib/types/playlet";
import type { TorrentFileInfo } from "$lib/types/torrent";

export type ActionExecutor = (
  action: Action,
  torrentId: number,
  torrentName: string,
  files: TorrentFileInfo[],
) => Promise<void>;

const executors = new Map<string, ActionExecutor>();

export function registerExecutor(type: string, executor: ActionExecutor): void {
  executors.set(type, executor);
}

export function getExecutor(type: string): ActionExecutor | undefined {
  return executors.get(type);
}
