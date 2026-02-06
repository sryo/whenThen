import { playletsState, derivePlayletName } from "$lib/state/playlets.svelte";
import { tasksState } from "$lib/state/tasks.svelte";
import { uiState } from "$lib/state/ui.svelte";
import type { TorrentAddedResponse } from "$lib/types/torrent";
import type { Playlet, TriggerType } from "$lib/types/playlet";

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

/** Score a playlet based on specificity. Higher = more specific. */
function scorePlaylet(p: Playlet): number {
  let score = 0;

  // Conditions (2 points each, more specific operators get bonus)
  for (const c of p.conditions) {
    score += 2;
    if (c.operator === "equals") score += 1;
    if (c.field === "name" && c.operator === "regex") score += 1;
  }

  // File filter specificity
  if (p.fileFilter) {
    switch (p.fileFilter.category) {
      case "custom": score += 3; break;
      case "video": case "audio": case "subtitle": score += 2; break;
      case "all": score += 1; break;
    }
    if (p.fileFilter.selectLargest) score += 1;
    if (p.fileFilter.minSizeMb) score += 1;
  }

  // Trigger specificity
  if (p.trigger.type === "folder_watch" && p.trigger.watchFolder) score += 2;
  if (p.trigger.type === "seeding_ratio") score += 1;

  return score;
}

/** Find the best-matching playlet for a torrent based on conditions and specificity. */
export function findBestMatch(
  torrentName: string,
  triggerType: TriggerType = "torrent_added",
  totalBytes?: number,
  fileCount?: number,
): Playlet | null {
  const eligible = playletsState.playlets.filter(
    (p) => p.enabled && p.trigger.type === triggerType,
  );

  let best: Playlet | null = null;
  let bestScore = -1;

  for (const p of eligible) {
    if (!playletsState.matchesConditions(p, torrentName, totalBytes, fileCount)) continue;
    const score = scorePlaylet(p);
    if (score > bestScore) {
      bestScore = score;
      best = p;
    }
  }

  return best;
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
