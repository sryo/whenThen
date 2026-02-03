import type { Component } from "svelte";
import type { Action } from "$lib/types/playlet";
import { devicesState } from "$lib/state/devices.svelte";
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
  label: string;
  verb: string;
  icon: Component;
  color: string;
  configLabel: string;
  defaultData: Record<string, unknown>;
}

const registry = new Map<string, ActionDefinition>();

export function registerAction(def: ActionDefinition): void {
  registry.set(def.type, def);
}

export function getActionDef(type: string): ActionDefinition | undefined {
  return registry.get(type);
}

export function getAllActions(): ActionDefinition[] {
  return Array.from(registry.values());
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
      return action.app || "Default";
    case "notify":
      return "Notify";
    case "subtitle":
      return action.languages.length > 0 ? action.languages.join(", ") : "System";
    case "automation":
      return action.method || null;
    case "delay":
      return `${action.seconds}s`;
    case "webhook":
      return action.method || null;
    case "delete_source":
      return action.deleteFiles ? "with files" : "torrent only";
  }
}

registerAction({
  type: "cast",
  label: "Cast",
  verb: "cast",
  icon: Cast as Component,
  color: "var(--color-primary)",
  configLabel: "Cast to",
  defaultData: { deviceId: null },
});

registerAction({
  type: "move",
  label: "Move",
  verb: "move",
  icon: FolderOutput as Component,
  color: "var(--color-warning)",
  configLabel: "Move to",
  defaultData: { destination: "" },
});

registerAction({
  type: "notify",
  label: "Notify",
  verb: "notify",
  icon: Bell as Component,
  color: "var(--color-success)",
  configLabel: "Send",
  defaultData: { method: "system" },
});

registerAction({
  type: "play",
  label: "Play",
  verb: "play",
  icon: MonitorPlay as Component,
  color: "var(--color-error)",
  configLabel: "Play in",
  defaultData: { app: "" },
});

registerAction({
  type: "subtitle",
  label: "Subtitles",
  verb: "subtitle",
  icon: Subtitles as Component,
  color: "#6366f1",
  configLabel: "Fetch in",
  defaultData: { languages: [] },
});

registerAction({
  type: "automation",
  label: "Automation",
  verb: "automate",
  icon: Terminal as Component,
  color: "#8b5cf6",
  configLabel: "Run",
  defaultData: { method: "shell", script: "", shortcutName: "" },
});

registerAction({
  type: "delay",
  label: "Delay",
  verb: "wait",
  icon: Timer as Component,
  color: "#64748b",
  configLabel: "Wait",
  defaultData: { seconds: 5, delayUnit: "seconds" },
});

registerAction({
  type: "webhook",
  label: "Webhook",
  verb: "webhook",
  icon: Globe as Component,
  color: "#06b6d4",
  configLabel: "Send to",
  defaultData: { url: "", method: "POST" },
});

registerAction({
  type: "delete_source",
  label: "Delete",
  verb: "clean up",
  icon: Trash2 as Component,
  color: "#ef4444",
  configLabel: "Delete",
  defaultData: { deleteFiles: true },
});
