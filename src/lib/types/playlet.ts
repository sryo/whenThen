// Trigger types
export type TriggerType = "torrent_added" | "download_complete" | "metadata_received" | "seeding_ratio";

export interface TriggerConfig {
  type: TriggerType;
  seedingRatio?: number;
  watchFolder?: string;
}

// Condition types
export type ConditionField = "name" | "total_size" | "file_count";
export type ConditionOperator = "contains" | "not_contains" | "starts_with" | "ends_with" | "equals" | "regex";
export type SizeOperator = "gt" | "lt" | "between";

export interface TriggerCondition {
  id: string;
  field: ConditionField;
  operator: ConditionOperator;
  sizeOperator?: SizeOperator;
  value: string;
  numericValue?: number;
  numericValueEnd?: number;
  negate: boolean;
}

export type ConditionLogic = "and" | "or";

// File filter types
export type FileFilterCategory = "all" | "video" | "audio" | "subtitle" | "custom";

export interface FileFilter {
  category: FileFilterCategory;
  customExtensions: string[];
  selectLargest?: boolean;
  minSizeMb?: number;
  namePattern?: string;
}

// Action types
export type ActionType =
  | "cast"
  | "move"
  | "notify"
  | "play"
  | "subtitle"
  | "automation"
  | "delay"
  | "webhook"
  | "delete_source";

export type AutomationMethod = "shell" | "applescript" | "shortcut";

export interface ActionBase {
  id: string;
  type: ActionType;
}

export interface CastAction extends ActionBase {
  type: "cast";
  deviceId: string | null;
}

export interface MoveAction extends ActionBase {
  type: "move";
  destination: string;
}

export interface NotifyAction extends ActionBase {
  type: "notify";
  method: "system";
}

export interface PlayAction extends ActionBase {
  type: "play";
  app: string;
  usePlaylist?: boolean;
}

export interface SubtitleAction extends ActionBase {
  type: "subtitle";
  languages: string[];
}

export interface AutomationAction extends ActionBase {
  type: "automation";
  method: AutomationMethod;
  script: string;
  shortcutName: string;
}

export type DelayUnit = "seconds" | "minutes" | "days" | "weeks" | "months";

export interface DelayAction extends ActionBase {
  type: "delay";
  seconds: number;
  delayUnit?: DelayUnit;
}

export interface WebhookAction extends ActionBase {
  type: "webhook";
  url: string;
  method: "POST" | "GET";
}

export interface DeleteSourceAction extends ActionBase {
  type: "delete_source";
  deleteFiles: boolean;
}

export type Action =
  | CastAction
  | MoveAction
  | NotifyAction
  | PlayAction
  | SubtitleAction
  | AutomationAction
  | DelayAction
  | WebhookAction
  | DeleteSourceAction;

export interface Playlet {
  id: string;
  name: string;
  enabled: boolean;
  trigger: TriggerConfig;
  actions: Action[];
  conditions: TriggerCondition[];
  conditionLogic: ConditionLogic;
  fileFilter: FileFilter | null;
  createdAt: string;
}
