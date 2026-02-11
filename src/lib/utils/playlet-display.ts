// Shared playlet rendering helpers used by PlayletsView.
import type { Action, Playlet, DelayUnit, PlayAction } from "$lib/types/playlet";
import { getActionDef } from "$lib/services/action-registry";
import { devicesState } from "$lib/state/devices.svelte";
import { t } from "$lib/i18n";

export interface DetailPart {
  text: string;
  color: string;
}

export function triggerDetails(playlet: Playlet): DetailPart[] {
  const parts: DetailPart[] = [];

  const triggerType = playlet.trigger?.type ?? "torrent_added";
  if (triggerType === "torrent_added") {
    parts.push({ text: t("playlets.onAdded"), color: "var(--color-primary)" });
  } else if (triggerType === "download_complete") {
    parts.push({ text: t("playlets.onComplete"), color: "var(--color-primary)" });
  } else if (triggerType === "metadata_received") {
    parts.push({ text: t("playlets.onMetadata"), color: "var(--color-primary)" });
  } else if (triggerType === "seeding_ratio") {
    const ratio = playlet.trigger.seedingRatio ?? 2.0;
    parts.push({ text: t("playlets.atRatio", { ratio: String(ratio) }), color: "var(--color-primary)" });
  } else if (triggerType === "folder_watch") {
    const folder = playlet.trigger.watchFolder
      ? playlet.trigger.watchFolder.replace(/\/+$/, "").split("/").pop() || "folder"
      : "folder";
    parts.push({ text: t("playlets.whenFolder", { folder }), color: "var(--color-primary)" });
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
      case "all": text = t("common.any"); break;
      case "video": text = t("common.video"); break;
      case "audio": text = t("common.audio"); break;
      case "subtitle": text = t("common.subtitle"); break;
      case "custom": {
        const exts = playlet.fileFilter.customExtensions;
        if (exts.length > 0) text = exts.join(", ");
        break;
      }
    }
    if (text) parts.push({ text, color: "var(--color-accent)" });
  }

  if (parts.length === 0) {
    parts.push({ text: t("common.any"), color: "var(--color-text-muted)" });
  }

  return parts;
}

type TranslateFn = (key: string, args?: Record<string, string | number>) => string;

// Returns phrases like "Move to Downloads" or "Cast to Living Room".
// Optional translate fn for reactive contexts.
export function actionPhrase(action: Action, translate?: TranslateFn): string {
  const tr = translate ?? t;
  const verb = tr(`actions.${action.type}.verb`);

  switch (action.type) {
    case "move": {
      if (!action.destination) return verb;
      const parts = action.destination.replace(/\/+$/, "").split("/");
      const folder = parts[parts.length - 1];
      return folder ? tr("actions.phrases.moveTo", { target: folder }) : verb;
    }
    case "play": {
      const play = action as PlayAction;
      const target = play.app || tr("common.default");
      if (play.usePlaylist) {
        return tr("actions.phrases.playPlaylistIn", { target });
      }
      return tr("actions.phrases.playIn", { target });
    }
    case "cast": {
      if (!action.deviceId) return verb;
      const dev = devicesState.devices.find((d) => d.id === action.deviceId);
      return dev?.name ? tr("actions.phrases.castTo", { target: dev.name }) : verb;
    }
    case "subtitle":
      return action.languages.length > 0
        ? tr("actions.phrases.subtitleIn", { target: action.languages.join(", ") })
        : verb;
    case "delay": {
      const unitKey = `actions.delay${capitalize(action.delayUnit ?? "seconds")}`;
      const unit = tr(unitKey);
      return tr("actions.phrases.delayFor", { seconds: action.seconds, unit });
    }
    case "automation": {
      const methodLabel = action.method ? tr(`actions.${action.method}`) : "";
      return methodLabel ? tr("actions.phrases.automationWith", { method: methodLabel }) : verb;
    }
    case "notify":
      return verb;
    case "webhook":
      return verb;
    case "delete_source":
      return action.deleteFiles
        ? tr("actions.phrases.deleteWithFiles")
        : tr("actions.phrases.deleteTorrentOnly");
    default:
      return verb;
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

// Verb-only summary like "Cast & Move & Notify"
export function buildActionSummary(playlet: Playlet): string {
  if (playlet.actions.length === 0) return "";
  return playlet.actions.map((a) => getActionDef(a.type)?.verb ?? a.type).join(" & ");
}
