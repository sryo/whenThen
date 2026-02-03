import { playletsState, derivePlayletName } from "$lib/state/playlets.svelte";
import { tasksState } from "$lib/state/tasks.svelte";
import { uiState } from "$lib/state/ui.svelte";
import type { TorrentAddedResponse } from "$lib/types/torrent";

// Counter for in-flight manual card drops. Incremented before the IPC call
// so the torrent:added event handler (which fires before the IPC resolves)
// knows to skip auto-assignment.
let pendingManualDrops = 0;

export function beginManualDrop(): void {
  pendingManualDrops++;
}

/** Called by the torrent:added listener. Returns true (and decrements) if
 *  a manual drop is in flight, meaning auto-assignment should be skipped. */
export function shouldSkipAutoAssign(): boolean {
  if (pendingManualDrops > 0) {
    pendingManualDrops--;
    return true;
  }
  return false;
}

export function assignTorrentToPlaylet(
  playletId: string,
  response: TorrentAddedResponse,
  /** Skip trigger-type and condition checks (manual card drop). */
  manual = false,
): boolean {
  const playlet = playletsState.getById(playletId);
  if (!playlet) return false;
  if (!playlet.enabled) {
    uiState.addToast(`"${derivePlayletName(playlet)}" is disabled`, "warning");
    return false;
  }
  if (!manual) {
    if (playlet.trigger.type !== "torrent_added") {
      uiState.addToast(`"${derivePlayletName(playlet)}" doesn't trigger on add`, "warning");
      return false;
    }
    if (!playletsState.matchesConditions(playlet, response.name)) {
      uiState.addToast(`"${response.name}" doesn't match "${derivePlayletName(playlet)}"`, "warning");
      return false;
    }
  }
  const existing = tasksState.getByTorrentId(response.id);
  if (existing) {
    uiState.addToast(`"${response.name}" already has an active task`, "warning");
    return false;
  }
  tasksState.createTask(response.id, response.name, playlet.id, derivePlayletName(playlet));
  uiState.addToast(`"${derivePlayletName(playlet)}" applied`, "success");
  return true;
}
