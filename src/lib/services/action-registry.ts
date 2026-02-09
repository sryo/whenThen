import type { IconType } from "$lib/types/ui";
import type { Action } from "$lib/types/playlet";
import { devicesState } from "$lib/state/devices.svelte";
import { t } from "$lib/i18n";
import {
  Cast,
  FolderOutput,
  Bell,
  MonitorPlay,
  Subtitles,
  Terminal,
  Timer,
  Globe,
  Trash2,
} from "lucide-svelte";

export interface ActionDefinition {
  type: string;
  labelKey: string;
  verbKey: string;
  icon: IconType;
  color: string;
  configLabelKey: string;
  defaultData: Record<string, unknown>;
}

export interface ResolvedActionDefinition {
  type: string;
  label: string;
  verb: string;
  icon: IconType;
  color: string;
  configLabel: string;
  defaultData: Record<string, unknown>;
}

const registry = new Map<string, ActionDefinition>();

export function registerAction(def: ActionDefinition): void {
  registry.set(def.type, def);
}

export function getActionDef(type: string): ResolvedActionDefinition | undefined {
  const def = registry.get(type);
  if (!def) return undefined;
  return {
    type: def.type,
    label: t(def.labelKey),
    verb: t(def.verbKey),
    icon: def.icon,
    color: def.color,
    configLabel: t(def.configLabelKey),
    defaultData: def.defaultData,
  };
}

export function getAllActionDefs(): ResolvedActionDefinition[] {
  return Array.from(registry.values()).map((def) => ({
    type: def.type,
    label: t(def.labelKey),
    verb: t(def.verbKey),
    icon: def.icon,
    color: def.color,
    configLabel: t(def.configLabelKey),
    defaultData: def.defaultData,
  }));
}

/** Returns the configured value for an action, or null if unconfigured. */
export function getActionLabel(action: Action): string | null {
  switch (action.type) {
    case "cast": {
      if (!action.deviceId) return null;
      const dev = devicesState.devices.find((d) => d.id === action.deviceId);
      return dev?.name ?? null;
    }
    case "move": {
      if (!action.destination) return null;
      const parts = action.destination.replace(/\/+$/, "").split("/");
      return parts[parts.length - 1] || null;
    }
    case "play":
      return action.app || t("common.default");
    case "notify":
      return t("actions.notify.label");
    case "subtitle":
      return action.languages.length > 0 ? action.languages.join(", ") : t("common.system");
    case "automation":
      return action.method || null;
    case "delay":
      return `${action.seconds}s`;
    case "webhook":
      return action.method || null;
    case "delete_source":
      return action.deleteFiles ? t("common.withFiles") : t("common.torrentOnly");
  }
}

registerAction({
  type: "cast",
  labelKey: "actions.cast.label",
  verbKey: "actions.cast.verb",
  icon: Cast,
  color: "var(--color-primary)",
  configLabelKey: "actions.cast.config",
  defaultData: { deviceId: null },
});

registerAction({
  type: "move",
  labelKey: "actions.move.label",
  verbKey: "actions.move.verb",
  icon: FolderOutput,
  color: "var(--color-warning)",
  configLabelKey: "actions.move.config",
  defaultData: { destination: "" },
});

registerAction({
  type: "notify",
  labelKey: "actions.notify.label",
  verbKey: "actions.notify.verb",
  icon: Bell,
  color: "var(--color-success)",
  configLabelKey: "actions.notify.config",
  defaultData: { method: "system" },
});

registerAction({
  type: "play",
  labelKey: "actions.play.label",
  verbKey: "actions.play.verb",
  icon: MonitorPlay,
  color: "var(--color-error)",
  configLabelKey: "actions.play.config",
  defaultData: { app: "", usePlaylist: false },
});

registerAction({
  type: "subtitle",
  labelKey: "actions.subtitle.label",
  verbKey: "actions.subtitle.verb",
  icon: Subtitles,
  color: "#6366f1",
  configLabelKey: "actions.subtitle.config",
  defaultData: { languages: [] },
});

registerAction({
  type: "automation",
  labelKey: "actions.automation.label",
  verbKey: "actions.automation.verb",
  icon: Terminal,
  color: "#8b5cf6",
  configLabelKey: "actions.automation.config",
  defaultData: { method: "shell", script: "", shortcutName: "" },
});

registerAction({
  type: "delay",
  labelKey: "actions.delay.label",
  verbKey: "actions.delay.verb",
  icon: Timer,
  color: "#64748b",
  configLabelKey: "actions.delay.config",
  defaultData: { seconds: 5, delayUnit: "seconds" },
});

registerAction({
  type: "webhook",
  labelKey: "actions.webhook.label",
  verbKey: "actions.webhook.verb",
  icon: Globe,
  color: "#06b6d4",
  configLabelKey: "actions.webhook.config",
  defaultData: { url: "", method: "POST" },
});

registerAction({
  type: "delete_source",
  labelKey: "actions.deleteSource.label",
  verbKey: "actions.deleteSource.verb",
  icon: Trash2,
  color: "#ef4444",
  configLabelKey: "actions.deleteSource.config",
  defaultData: { deleteFiles: true },
});
