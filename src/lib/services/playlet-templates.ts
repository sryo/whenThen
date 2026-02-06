// Starter playlet templates for quick setup.
import type { Playlet, Action, TriggerConfig, FileFilter } from "$lib/types/playlet";
import { t } from "$lib/i18n";

export interface PlayletTemplate {
  name: string;
  description: string;
  trigger: TriggerConfig;
  actions: Omit<Action, "id">[];
  fileFilter: FileFilter | null;
}

function action(type: string, data: Record<string, unknown>): Omit<Action, "id"> {
  return { type, ...data } as Omit<Action, "id">;
}

// AppleScript that renames video files to the cleaned torrent name.
// The automation executor injects cleanName, torrentName, and downloadDir.
const RENAME_CLEAN_SCRIPT = `set videoExts to {"mp4", "mkv", "avi", "mov", "webm"}
set torrentPath to downloadDir & "/" & torrentName
tell application "System Events"
\tif exists folder torrentPath then
\t\trepeat with f in (every file of folder torrentPath)
\t\t\tif videoExts contains (name extension of f) then
\t\t\t\tset name of f to cleanName & "." & (name extension of f)
\t\t\tend if
\t\tend repeat
\telse if exists file torrentPath then
\t\tset ext to name extension of file torrentPath
\t\tset name of file torrentPath to cleanName & "." & ext
\tend if
end tell`;

export function getPlayletTemplates(): PlayletTemplate[] {
  return [
    {
      name: t("templates.watchNow.name"),
      description: t("templates.watchNow.description"),
      trigger: { type: "torrent_added" },
      actions: [
        action("subtitle", { languages: ["en"] }),
        action("cast", { deviceId: null }),
      ],
      fileFilter: { category: "video", customExtensions: [], selectLargest: true },
    },
    {
      name: t("templates.organizeVideos.name"),
      description: t("templates.organizeVideos.description"),
      trigger: { type: "download_complete" },
      actions: [
        action("automation", {
          method: "applescript",
          shortcutName: "",
          script: RENAME_CLEAN_SCRIPT,
        }),
        action("move", { destination: "" }),
        action("subtitle", { languages: ["en"] }),
      ],
      fileFilter: { category: "video", customExtensions: [] },
    },
    {
      name: t("templates.goodSeeder.name"),
      description: t("templates.goodSeeder.description"),
      trigger: { type: "seeding_ratio", seedingRatio: 2.0 },
      actions: [
        action("notify", { method: "system" }),
      ],
      fileFilter: null,
    },
  ];
}

/** @deprecated Use getPlayletTemplates() instead */
export const playletTemplates: PlayletTemplate[] = [];

export function createPlayletFromTemplate(template: PlayletTemplate): Playlet {
  return {
    id: crypto.randomUUID(),
    name: "",
    enabled: true,
    trigger: { ...template.trigger },
    actions: template.actions.map((a) => ({ ...a, id: crypto.randomUUID() }) as Action),
    conditions: [],
    conditionLogic: "and",
    fileFilter: template.fileFilter ? { ...template.fileFilter } : null,
    createdAt: new Date().toISOString(),
  };
}
