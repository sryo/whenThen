// Event listeners for the tray panel window.
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

let unlisteners: UnlistenFn[] = [];

export interface TrayDropPayload {
  content_type: "file_paths" | "urls" | "text";
  data: string[];
}

let onShow: (() => void) | null = null;
let onHide: (() => void) | null = null;
let onDrop: ((payload: TrayDropPayload) => void) | null = null;

export function setCallbacks(cbs: {
  onShow?: () => void;
  onHide?: () => void;
  onDrop?: (payload: TrayDropPayload) => void;
}) {
  onShow = cbs.onShow ?? null;
  onHide = cbs.onHide ?? null;
  onDrop = cbs.onDrop ?? null;
}

export async function setupTrayPanelListeners() {
  unlisteners.push(
    await listen("tray:panel-show", () => {
      onShow?.();
    }),
  );

  unlisteners.push(
    await listen("tray:panel-hide", () => {
      onHide?.();
    }),
  );

  unlisteners.push(
    await listen<TrayDropPayload>("tray:drop", (event) => {
      onDrop?.(event.payload);
    }),
  );

  unlisteners.push(
    await listen("tray:drag-enter", () => {
      // Show the panel when something is dragged over the tray icon
      onShow?.();
    }),
  );

  // Hide panel when it loses focus
  const currentWindow = getCurrentWindow();
  unlisteners.push(
    await currentWindow.onFocusChanged(({ payload: focused }) => {
      if (!focused) {
        onHide?.();
      }
    }),
  );
}

export function cleanupTrayPanelListeners() {
  unlisteners.forEach((fn) => fn());
  unlisteners = [];
  onShow = null;
  onHide = null;
  onDrop = null;
}
