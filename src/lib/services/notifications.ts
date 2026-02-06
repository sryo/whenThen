// Native notifications using tauri-plugin-notification.

import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

let permissionGranted = false;

export async function initNotifications(): Promise<boolean> {
  permissionGranted = await isPermissionGranted();
  if (!permissionGranted) {
    const permission = await requestPermission();
    permissionGranted = permission === "granted";
  }
  return permissionGranted;
}

export async function notify(title: string, body?: string): Promise<void> {
  if (!permissionGranted) {
    await initNotifications();
  }
  if (permissionGranted) {
    sendNotification({ title, body });
  }
}

export async function notifyRssMatch(feedName: string, title: string): Promise<void> {
  await notify(`New match: ${feedName}`, title);
}

export async function notifyDownloadStarted(name: string): Promise<void> {
  await notify("Download started", name);
}

export async function notifyDownloadComplete(name: string): Promise<void> {
  await notify("Download complete", name);
}

export async function notifyActionsComplete(name: string, actionCount: number): Promise<void> {
  await notify(
    "Actions complete",
    `${name} - ${actionCount} action${actionCount !== 1 ? "s" : ""} executed`,
  );
}
