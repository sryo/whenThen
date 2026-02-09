import type { AppSettings } from "$lib/types/settings";
import { DEFAULT_SETTINGS } from "$lib/types/settings";
import { settingsUpdate } from "$lib/services/tauri-commands";
import { applyColorScheme, buildSystemScheme } from "$lib/themes";
import { getCurrentWindow } from "@tauri-apps/api/window";

let settings = $state<AppSettings>({ ...DEFAULT_SETTINGS });

let cachedOsIsDark =
  typeof window !== "undefined" && window.matchMedia("(prefers-color-scheme: dark)").matches;

export const settingsState = {
  get settings() {
    return settings;
  },
  get theme() {
    return settings.theme;
  },
  get downloadDirectory() {
    return settings.download_directory;
  },

  setSettings(newSettings: AppSettings) {
    settings = { ...newSettings };
    this.applyScheme();
  },

  updateSetting<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    settings = { ...settings, [key]: value };
  },

  async updateAndSave(partial: Partial<AppSettings>) {
    settings = { ...settings, ...partial };
    try {
      await settingsUpdate(settings);
    } catch {
      // Persist failed silently
    }
  },

  applyScheme() {
    applyColorScheme(buildSystemScheme(cachedOsIsDark));
  },
};

if (typeof window !== "undefined") {
  getCurrentWindow().theme().then((t) => {
    cachedOsIsDark = t === "dark";
    settingsState.applyScheme();
  });

  getCurrentWindow().onThemeChanged(({ payload: theme }) => {
    cachedOsIsDark = theme === "dark";
    requestAnimationFrame(() => {
      settingsState.applyScheme();
    });
  });
}
