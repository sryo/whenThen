import type { Component } from "svelte";

export type ViewName = "playlets" | "activity" | "settings";

export interface Toast {
  id: string;
  message: string;
  level: "info" | "success" | "warning" | "error";
}

export interface ContextMenuItem {
  type?: "item";
  icon?: Component;
  label: string;
  action: () => void;
  disabled?: boolean;
  danger?: boolean;
  hidden?: boolean;
}

export interface ContextMenuDivider {
  type: "divider";
}

export type ContextMenuEntry = ContextMenuItem | ContextMenuDivider;
