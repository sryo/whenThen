// Starter playlet templates for quick setup.
import type { Playlet, Action, TriggerConfig, FileFilter } from "$lib/types/playlet";

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

export const playletTemplates: PlayletTemplate[] = [
  {
    name: "Movie Night",
    description: "Sit back — subtitles grabbed, TV starts playing",
    trigger: { type: "torrent_added" },
    actions: [
      action("cast", { deviceId: null }),
      action("subtitle", { languages: ["en"] }),
    ],
    fileFilter: { category: "video", customExtensions: [], selectLargest: true },
  },
  {
    name: "Organize Media",
    description: "No mess — files get renamed, sorted, and subbed automatically",
    trigger: { type: "torrent_added" },
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
    name: "Seed & Notify",
    description: "Be a good peer — get notified when you've seeded enough",
    trigger: { type: "seeding_ratio", seedingRatio: 2.0 },
    actions: [
      action("notify", { method: "system" }),
    ],
    fileFilter: null,
  },
];

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
