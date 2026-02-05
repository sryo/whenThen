// Shared playlet rendering helpers used by PlayletsView and TrayPanel.
import type { Component } from "svelte";
import type { IconType } from "$lib/types/ui";
import type { Playlet, TriggerType, FileFilterCategory } from "$lib/types/playlet";
import {
  Link,
  Filter,
  Files,
  FileVideo,
  Music,
  FileCode,
  CircleCheck,
  FileSearch,
  ArrowUpDown,
  Captions,
  FolderSearch,
  Plus,
} from "lucide-svelte";
import { getActionDef, getActionLabel } from "$lib/services/action-registry";

const filterCategoryIcons: Record<FileFilterCategory, typeof Files> = {
  all: Files,
  video: FileVideo,
  audio: Music,
  subtitle: Captions,
  custom: FileCode,
};

const triggerTypeIcons: Record<TriggerType, typeof Link> = {
  torrent_added: Link,
  download_complete: CircleCheck,
  metadata_received: FileSearch,
  seeding_ratio: ArrowUpDown,
  folder_watch: FolderSearch,
};

export interface TriggerIcon {
  icon: IconType;
  color: string;
}

export function buildTriggerIcons(playlet: Playlet): TriggerIcon[] {
  const triggerType = playlet.trigger?.type ?? "torrent_added";
  const icons: TriggerIcon[] = [
    { icon: triggerTypeIcons[triggerType] ?? Link, color: "var(--color-primary)" },
  ];
  if (playlet.conditions.length > 0) {
    icons.push({ icon: Filter, color: "var(--color-info)" });
  }
  if (playlet.fileFilter) {
    icons.push({ icon: filterCategoryIcons[playlet.fileFilter.category], color: "var(--color-accent)" });
  }
  return icons;
}

export interface DetailPart {
  text: string;
  color: string;
}

export function triggerDetails(playlet: Playlet): DetailPart[] {
  const parts: DetailPart[] = [];

  const triggerType = playlet.trigger?.type ?? "torrent_added";
  if (triggerType === "download_complete") {
    parts.push({ text: "complete", color: "var(--color-primary)" });
  } else if (triggerType === "metadata_received") {
    parts.push({ text: "metadata", color: "var(--color-primary)" });
  } else if (triggerType === "seeding_ratio") {
    const ratio = playlet.trigger.seedingRatio ?? 2.0;
    parts.push({ text: `ratio ${ratio}`, color: "var(--color-primary)" });
  } else if (triggerType === "folder_watch") {
    if (playlet.trigger.watchFolder) {
      const segments = playlet.trigger.watchFolder.replace(/\/+$/, "").split("/");
      parts.push({ text: segments[segments.length - 1] || "folder", color: "var(--color-primary)" });
    } else {
      parts.push({ text: "folder", color: "var(--color-primary)" });
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

export interface ActionBlockData {
  label: string;
  color: string;
  icon: IconType;
}

export function buildActionBlocks(playlet: Playlet): ActionBlockData[] {
  return playlet.actions.map((action) => {
    const def = getActionDef(action.type);
    const configured = getActionLabel(action);
    return {
      label: configured ?? def?.label ?? action.type,
      color: def?.color ?? "var(--color-text-muted)",
      icon: def?.icon ?? Plus,
    };
  });
}
