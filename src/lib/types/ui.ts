import type { Component, ComponentType, SvelteComponent } from "svelte";

// lucide-svelte exports Svelte 4 class components; this union accepts both styles
export type IconType = Component | ComponentType<SvelteComponent<any>>;

export type ViewName = "playlets" | "activity" | "settings";

export interface Toast {
  id: string;
  message: string;
  level: "info" | "success" | "warning" | "error";
}

export interface ContextMenuItem {
  type?: "item";
  icon?: IconType;
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
