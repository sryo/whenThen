// Shared playlet rendering helpers used by PlayletsView.
import type { Action, Playlet, DelayUnit } from "$lib/types/playlet";
import { getActionDef, getActionLabel } from "$lib/services/action-registry";
import { devicesState } from "$lib/state/devices.svelte";

export interface DetailPart {
  text: string;
  color: string;
}

export function triggerDetails(playlet: Playlet): DetailPart[] {
  const parts: DetailPart[] = [];

  const triggerType = playlet.trigger?.type ?? "torrent_added";
  if (triggerType === "download_complete") {
    parts.push({ text: "on complete", color: "var(--color-primary)" });
  } else if (triggerType === "metadata_received") {
    parts.push({ text: "on metadata", color: "var(--color-primary)" });
  } else if (triggerType === "seeding_ratio") {
    const ratio = playlet.trigger.seedingRatio ?? 2.0;
    parts.push({ text: `at ratio ${ratio}`, color: "var(--color-primary)" });
  } else if (triggerType === "folder_watch") {
    if (playlet.trigger.watchFolder) {
      const segments = playlet.trigger.watchFolder.replace(/\/+$/, "").split("/");
      parts.push({ text: `when ${segments[segments.length - 1] || "folder"}`, color: "var(--color-primary)" });
    } else {
      parts.push({ text: "when folder", color: "var(--color-primary)" });
    }
  }

  const vals = playlet.conditions
    .filter((c) => {
      if (c.field === "name") return c.value.trim();
      return c.numericValue !== undefined;
    })
    .map((c) => {
      if (c.field === "name") return c.value.trim();
      if (c.field === "total_size") return `${c.numericValue}MB`;
      return `${c.numericValue} files`;
    });
  if (vals.length > 0) {
    const join = playlet.conditionLogic === "or" ? " | " : " & ";
    parts.push({ text: vals.join(join), color: "var(--color-info)" });
  }

  if (playlet.fileFilter) {
    let text = "";
    switch (playlet.fileFilter.category) {
      case "all": text = "any"; break;
      case "video": text = "video"; break;
      case "audio": text = "audio"; break;
      case "subtitle": text = "subtitle"; break;
      case "custom": {
        const exts = playlet.fileFilter.customExtensions;
        if (exts.length > 0) text = exts.join(", ");
        break;
      }
    }
    if (text) parts.push({ text, color: "var(--color-accent)" });
  }

  if (parts.length === 0) {
    parts.push({ text: "any", color: "var(--color-text-muted)" });
  }

  return parts;
}

// Build a phrase like "Move to Downloads" or "Cast to Living Room".
export function actionPhrase(action: Action): string {
  const def = getActionDef(action.type);
  const verb = def?.verb ?? action.type;

  switch (action.type) {
    case "move": {
      if (!action.destination) return capitalize(verb);
      const parts = action.destination.replace(/\/+$/, "").split("/");
      const folder = parts[parts.length - 1];
      return folder ? `${capitalize(verb)} to ${folder}` : capitalize(verb);
    }
    case "play":
      return action.app ? `${capitalize(verb)} in ${action.app}` : capitalize(verb);
    case "cast": {
      if (!action.deviceId) return capitalize(verb);
      const dev = devicesState.devices.find((d) => d.id === action.deviceId);
      return dev?.name ? `${capitalize(verb)} to ${dev.name}` : capitalize(verb);
    }
    case "subtitle":
      return action.languages.length > 0
        ? `Subtitle in ${action.languages.join(", ")}`
        : "Subtitle";
    case "delay": {
      const unitLabels: Record<DelayUnit, string> = {
        seconds: "sec", minutes: "min", days: "days", weeks: "weeks", months: "months",
      };
      const u = action.delayUnit ?? "seconds";
      return `${capitalize(verb)} ${action.seconds} ${unitLabels[u]}`;
    }
    case "automation":
      return action.method ? `${capitalize(verb)} (${action.method})` : capitalize(verb);
    case "notify":
      return "Notify";
    case "webhook":
      return action.url ? "Webhook" : "Webhook";
    case "delete_source":
      return action.deleteFiles ? "Delete with files" : "Delete torrent";
    default:
      return capitalize(verb);
  }
}

function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

export interface ActionPhrase {
  text: string;
  color: string;
}

export function buildActionPhrases(playlet: Playlet): ActionPhrase[] {
  return playlet.actions.map((action) => {
    const def = getActionDef(action.type);
    return {
      text: actionPhrase(action),
      color: def?.color ?? "var(--color-text-muted)",
    };
  });
}
