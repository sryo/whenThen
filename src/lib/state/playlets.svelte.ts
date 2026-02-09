import { load } from "@tauri-apps/plugin-store";
import type {
  Playlet,
  Action,
  ActionType,
  TriggerCondition,
  ConditionOperator,
  ConditionField,
  FileFilter,
  TriggerConfig,
} from "$lib/types/playlet";
import type { TorrentFileInfo } from "$lib/types/torrent";
import { getActionDef } from "$lib/services/action-registry";
import { actionPhrase } from "$lib/utils/playlet-display";

let playlets = $state<Playlet[]>([]);

let storeInstance: Awaited<ReturnType<typeof load>> | null = null;

async function getStore() {
  if (!storeInstance) {
    storeInstance = await load("playlets.json", { autoSave: true, defaults: {} });
  }
  return storeInstance;
}

async function markDefaultsSeeded() {
  const store = await getStore();
  await store.set("defaultsSeeded", true);
}

async function wereDefaultsSeeded(): Promise<boolean> {
  const store = await getStore();
  return (await store.get<boolean>("defaultsSeeded")) === true;
}

function defaultPlaylets(): Playlet[] {
  return [
    {
      id: crypto.randomUUID(),
      name: "",
      enabled: true,
      trigger: { type: "download_complete" },
      actions: [createAction("notify")],
      conditions: [],
      conditionLogic: "and",
      fileFilter: null,
      createdAt: new Date().toISOString(),
    },
  ];
}

async function persistPlaylets() {
  try {
    const store = await getStore();
    await store.set("playlets", playlets);
  } catch {
    // fire-and-forget: persistence failure is non-critical
  }
}

// Migrate old shortcut/applescript actions to unified automation type
function migrateAction(action: any): Action {
  if (action.type === "shortcut") {
    return {
      id: action.id,
      type: "automation",
      method: "shortcut",
      shortcutName: action.shortcutName ?? "",
      script: "",
    };
  }
  if (action.type === "applescript") {
    return {
      id: action.id,
      type: "automation",
      method: "applescript",
      script: action.script ?? "",
      shortcutName: "",
    };
  }
  return action as Action;
}

function migratePlaylet(r: any): Playlet {
  return {
    ...r,
    enabled: r.enabled ?? true,
    trigger: r.trigger ?? { type: "torrent_added" },
    actions: (r.actions ?? []).map(migrateAction),
    conditions: (r.conditions ?? []).map((c: any) => ({
      ...c,
      field: c.field ?? "name",
      negate: c.negate ?? false,
    })),
    conditionLogic: r.conditionLogic ?? "and",
    fileFilter: r.fileFilter ?? null,
  };
}

function getSubject(filter: FileFilter | null): string {
  if (!filter) return "any torrent";
  switch (filter.category) {
    case "video": return "video torrents";
    case "audio": return "audio torrents";
    case "subtitle": return "subtitle torrents";
    case "custom": {
      const exts = filter.customExtensions;
      return exts.length > 0 ? `${exts.join(", ")} torrents` : "any torrent";
    }
    default: return "any torrent";
  }
}

const triggerLabel: Record<string, string> = {
  torrent_added: "",
  download_complete: "on complete",
  metadata_received: "on metadata",
  seeding_ratio: "on ratio",
  folder_watch: "from folder",
};

function operatorWord(op: ConditionOperator): string {
  switch (op) {
    case "contains": return "contains";
    case "not_contains": return "excludes";
    case "starts_with": return "starts with";
    case "ends_with": return "ends with";
    case "equals": return "equals";
  }
}

export function derivePlayletName(playlet: Playlet): string {
  const phrases = playlet.actions.map(actionPhrase);
  const actionStr = phrases.join(" & ");
  const subject = getSubject(playlet.fileFilter);

  const triggerPrefix = triggerLabel[playlet.trigger?.type ?? "torrent_added"] ?? "";

  const filled = playlet.conditions.filter((c) => {
    if (c.field === "name") return c.value.trim();
    return c.numericValue !== undefined;
  });

  if (filled.length > 0) {
    const values = filled.map((c) => {
      if (c.field === "name") {
        const neg = c.negate ? "not " : "";
        const op = operatorWord(c.operator);
        return `${neg}${op} '${c.value.trim()}'`;
      }
      if (c.field === "total_size") return `size ${c.sizeOperator ?? "gt"} ${c.numericValue}`;
      return `files ${c.sizeOperator ?? "gt"} ${c.numericValue}`;
    });
    const logic = playlet.conditionLogic === "or" ? " or " : " & ";
    const condStr = values.join(logic);

    if (!actionStr) return `When ${subject} ${condStr}`;
    const base = `When ${subject} ${condStr}, ${actionStr}`;
    return triggerPrefix ? `${base} ${triggerPrefix}` : base;
  }

  if (actionStr) {
    const capitalized = actionStr.charAt(0).toUpperCase() + actionStr.slice(1);
    return triggerPrefix ? `${capitalized} ${triggerPrefix}` : capitalized;
  }

  return "New playlet";
}

function createAction(type: ActionType): Action {
  const id = crypto.randomUUID();
  const def = getActionDef(type);
  if (def) {
    return { id, type, ...def.defaultData } as Action;
  }
  return { id, type } as Action;
}

const SUBTITLE_EXTENSIONS = ["srt", "ass", "sub", "vtt"];

// Compare a numeric value using size operators
function numericMatch(actual: number, operator: string | undefined, target: number | undefined, targetEnd: number | undefined): boolean {
  if (target === undefined) return true;
  const op = operator ?? "gt";
  switch (op) {
    case "gt": return actual > target;
    case "lt": return actual < target;
    case "between": return actual >= target && actual <= (targetEnd ?? target);
    default: return true;
  }
}

export const playletsState = {
  get playlets() {
    return playlets;
  },
  get isEmpty() {
    return playlets.length === 0;
  },
  get count() {
    return playlets.length;
  },

  getById(id: string): Playlet | undefined {
    return playlets.find((r) => r.id === id);
  },

  async loadPlaylets() {
    try {
      const store = await getStore();
      const saved = await store.get<Playlet[]>("playlets");
      if (saved && Array.isArray(saved)) {
        if (saved.length > 0 || await wereDefaultsSeeded()) {
          playlets = saved.map(migratePlaylet);
          return;
        }
      }
    } catch {
      // store load failed
    }

    // Try migrating from old actions.json
    try {
      const actionsStore = await load("actions.json", { autoSave: false, defaults: {} });
      const oldActions = await actionsStore.get<Action[]>("actions");
      if (oldActions && Array.isArray(oldActions) && oldActions.length > 0) {
        const defaultPlaylet: Playlet = {
          id: crypto.randomUUID(),
          name: "",
          enabled: true,
          trigger: { type: "torrent_added" },
          actions: oldActions.map(migrateAction),
          conditions: [],
          conditionLogic: "and",
          fileFilter: null,
          createdAt: new Date().toISOString(),
        };
        playlets = [defaultPlaylet];
        persistPlaylets();
        markDefaultsSeeded();
        return;
      }
    } catch {
      // no old actions to migrate
    }

    // Seed starter playlets
    playlets = defaultPlaylets();
    persistPlaylets();
    markDefaultsSeeded();
  },

  addPlaylet(): Playlet {
    const playlet: Playlet = {
      id: crypto.randomUUID(),
      name: "",
      enabled: true,
      trigger: { type: "torrent_added" },
      actions: [],
      conditions: [],
      conditionLogic: "and",
      fileFilter: null,
      createdAt: new Date().toISOString(),
    };
    playlets = [...playlets, playlet];
    persistPlaylets();
    return playlet;
  },

  addPlayletFromData(data: Playlet): void {
    playlets = [...playlets, data];
    persistPlaylets();
  },

  removePlaylet(id: string) {
    playlets = playlets.filter((r) => r.id !== id);
    persistPlaylets();
  },

  updatePlaylet(id: string, partial: Partial<Pick<Playlet, "conditionLogic" | "enabled" | "trigger">>) {
    const idx = playlets.findIndex((r) => r.id === id);
    if (idx >= 0) {
      playlets[idx] = { ...playlets[idx], ...partial };
    }
    persistPlaylets();
  },

  duplicatePlaylet(id: string): Playlet | undefined {
    const source = playlets.find((r) => r.id === id);
    if (!source) return undefined;
    const newPlaylet: Playlet = {
      id: crypto.randomUUID(),
      name: "",
      enabled: source.enabled,
      trigger: { ...source.trigger },
      actions: source.actions.map((a) => ({ ...a, id: crypto.randomUUID() })),
      conditions: source.conditions.map((c) => ({ ...c, id: crypto.randomUUID() })),
      conditionLogic: source.conditionLogic,
      fileFilter: source.fileFilter ? { ...source.fileFilter, customExtensions: [...source.fileFilter.customExtensions] } : null,
      createdAt: new Date().toISOString(),
    };
    playlets = [...playlets, newPlaylet];
    persistPlaylets();
    return newPlaylet;
  },

  // Action CRUD within playlet
  addAction(playletId: string, type: ActionType, atIndex?: number) {
    const idx = playlets.findIndex((r) => r.id === playletId);
    if (idx < 0) return;
    const action = createAction(type);
    const actions = [...playlets[idx].actions];
    if (atIndex !== undefined && atIndex >= 0 && atIndex <= actions.length) {
      actions.splice(atIndex, 0, action);
    } else {
      actions.push(action);
    }
    playlets[idx] = { ...playlets[idx], actions };
    persistPlaylets();
  },

  removeAction(playletId: string, actionId: string) {
    const idx = playlets.findIndex((r) => r.id === playletId);
    if (idx < 0) return;
    playlets[idx] = {
      ...playlets[idx],
      actions: playlets[idx].actions.filter((a) => a.id !== actionId),
    };
    persistPlaylets();
  },

  updateAction<T extends Action>(playletId: string, actionId: string, partial: Partial<T>) {
    const rIdx = playlets.findIndex((r) => r.id === playletId);
    if (rIdx < 0) return;
    const aIdx = playlets[rIdx].actions.findIndex((a) => a.id === actionId);
    if (aIdx < 0) return;
    const actions = [...playlets[rIdx].actions];
    actions[aIdx] = { ...actions[aIdx], ...partial } as Action;
    playlets[rIdx] = { ...playlets[rIdx], actions };
    persistPlaylets();
  },

  reorderActions(playletId: string, fromIndex: number, toIndex: number) {
    const idx = playlets.findIndex((r) => r.id === playletId);
    if (idx < 0) return;
    const actions = [...playlets[idx].actions];
    const [moved] = actions.splice(fromIndex, 1);
    actions.splice(toIndex, 0, moved);
    playlets[idx] = { ...playlets[idx], actions };
    persistPlaylets();
  },

  reorderPlaylet(fromIndex: number, toIndex: number) {
    if (fromIndex < 0 || fromIndex >= playlets.length) return;
    if (toIndex < 0 || toIndex >= playlets.length) return;
    if (fromIndex === toIndex) return;

    const arr = [...playlets];
    const [moved] = arr.splice(fromIndex, 1);
    arr.splice(toIndex, 0, moved);
    playlets = arr;
    persistPlaylets();
  },

  // Condition CRUD
  addCondition(playletId: string, field: ConditionField = "name", operator: ConditionOperator = "contains") {
    const idx = playlets.findIndex((r) => r.id === playletId);
    if (idx < 0) return;
    const condition: TriggerCondition = {
      id: crypto.randomUUID(),
      field,
      operator,
      value: "",
      negate: false,
    };
    playlets[idx] = {
      ...playlets[idx],
      conditions: [...playlets[idx].conditions, condition],
    };
    persistPlaylets();
  },

  updateCondition(playletId: string, conditionId: string, partial: Partial<Pick<TriggerCondition, "operator" | "value" | "negate" | "field" | "sizeOperator" | "numericValue" | "numericValueEnd">>) {
    const rIdx = playlets.findIndex((r) => r.id === playletId);
    if (rIdx < 0) return;
    const cIdx = playlets[rIdx].conditions.findIndex((c) => c.id === conditionId);
    if (cIdx < 0) return;
    const conditions = [...playlets[rIdx].conditions];
    conditions[cIdx] = { ...conditions[cIdx], ...partial };
    playlets[rIdx] = { ...playlets[rIdx], conditions };
    persistPlaylets();
  },

  removeCondition(playletId: string, conditionId: string) {
    const idx = playlets.findIndex((r) => r.id === playletId);
    if (idx < 0) return;
    playlets[idx] = {
      ...playlets[idx],
      conditions: playlets[idx].conditions.filter((c) => c.id !== conditionId),
    };
    persistPlaylets();
  },

  setFileFilter(playletId: string, filter: FileFilter | null) {
    const idx = playlets.findIndex((r) => r.id === playletId);
    if (idx < 0) return;
    playlets[idx] = { ...playlets[idx], fileFilter: filter };
    persistPlaylets();
  },

  filterFiles(playlet: Playlet, files: TorrentFileInfo[]): TorrentFileInfo[] {
    const filter = playlet.fileFilter;
    if (!filter || filter.category === "all") return applySecondaryFilters(filter, files);

    let result: TorrentFileInfo[];
    switch (filter.category) {
      case "video":
        result = files.filter((f) => f.mime_type?.startsWith("video/"));
        break;
      case "audio":
        result = files.filter((f) => f.mime_type?.startsWith("audio/"));
        break;
      case "subtitle": {
        result = files.filter((f) => {
          const dot = f.name.lastIndexOf(".");
          if (dot < 0) return false;
          return SUBTITLE_EXTENSIONS.includes(f.name.slice(dot + 1).toLowerCase());
        });
        break;
      }
      case "custom": {
        const exts = filter.customExtensions.map((e) => e.toLowerCase().replace(/^\./, ""));
        result = files.filter((f) => {
          const dot = f.name.lastIndexOf(".");
          if (dot < 0) return false;
          return exts.includes(f.name.slice(dot + 1).toLowerCase());
        });
        break;
      }
      default:
        result = files;
    }

    return applySecondaryFilters(filter, result);
  },

  matchesConditions(playlet: Playlet, torrentName: string, totalBytes?: number, fileCount?: number): boolean {
    if (playlet.conditions.length === 0) return true;
    const name = torrentName.toLowerCase();

    const test = (c: TriggerCondition) => {
      let result: boolean;

      if (c.field === "total_size") {
        if (totalBytes === undefined) return true;
        const sizeMb = totalBytes / (1024 * 1024);
        result = numericMatch(sizeMb, c.sizeOperator, c.numericValue, c.numericValueEnd);
      } else if (c.field === "file_count") {
        if (fileCount === undefined) return true;
        result = numericMatch(fileCount, c.sizeOperator, c.numericValue, c.numericValueEnd);
      } else {
        // field === "name"
        const val = c.value.toLowerCase();
        switch (c.operator) {
          case "contains":
            result = name.includes(val);
            break;
          case "not_contains":
            result = !name.includes(val);
            break;
          case "starts_with":
            result = name.startsWith(val);
            break;
          case "ends_with":
            result = name.endsWith(val);
            break;
          case "equals":
            result = name === val;
            break;
          case "regex":
            try {
              result = new RegExp(c.value, "i").test(torrentName);
            } catch {
              result = false;
            }
            break;
        }
      }

      return c.negate ? !result : result;
    };

    return playlet.conditionLogic === "or"
      ? playlet.conditions.some(test)
      : playlet.conditions.every(test);
  },
};

// Pipeline: minSizeMb → namePattern → selectLargest
function applySecondaryFilters(filter: FileFilter | null, files: TorrentFileInfo[]): TorrentFileInfo[] {
  if (!filter) return files;
  let result = files;

  if (filter.minSizeMb !== undefined && filter.minSizeMb > 0) {
    const minBytes = filter.minSizeMb * 1024 * 1024;
    result = result.filter((f) => f.length >= minBytes);
  }

  if (filter.namePattern) {
    const pattern = filter.namePattern.toLowerCase();
    result = result.filter((f) => f.name.toLowerCase().includes(pattern));
  }

  if (filter.selectLargest && result.length > 0) {
    const largest = result.reduce((a, b) => (b.length > a.length ? b : a));
    result = [largest];
  }

  return result;
}
