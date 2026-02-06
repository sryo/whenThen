// Native notifications using tauri-plugin-notification.

import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import { t } from "$lib/i18n";

let permissionGranted = false;

export async function initNotifications(): Promise<boolean> {
  permissionGranted = await isPermissionGranted();
  if (!permissionGranted) {
    const permission = await requestPermission();
    permissionGranted = permission === "granted";
  }
  return permissionGranted;
}

async function notify(title: string, body?: string): Promise<void> {
  if (!permissionGranted) {
    await initNotifications();
  }
  if (permissionGranted) {
    sendNotification({ title, body });
  }
}

export async function notifyRssMatch(feedName: string, title: string): Promise<void> {
  await notify(t("notifications.newMatch", { feedName }), title);
}

export async function notifyDownloadComplete(name: string): Promise<void> {
  await notify(t("notifications.downloadComplete"), name);
}
