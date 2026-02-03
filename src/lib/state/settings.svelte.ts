import type { AppSettings } from "$lib/types/settings";
import { DEFAULT_SETTINGS } from "$lib/types/settings";
import { settingsUpdate } from "$lib/services/tauri-commands";
import { applyColorScheme } from "$lib/themes/apply";
import { getSchemeById } from "$lib/themes/schemes";
import { buildSystemScheme } from "$lib/themes/system";
import { getCurrentWindow } from "@tauri-apps/api/window";

let settings = $state<AppSettings>({ ...DEFAULT_SETTINGS });

// Synchronous best-guess for first render; corrected by Tauri theme() immediately after
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

  // TODO: always_on_top ~40% — when partial includes always_on_top, call
  // Tauri's setAlwaysOnTop() window API to actually pin/unpin the window.
  async updateAndSave(partial: Partial<AppSettings>) {
    settings = { ...settings, ...partial };
    if (partial.color_scheme !== undefined) {
      this.applyScheme();
    }
    try {
      await settingsUpdate(settings);
    } catch {
      // Persist failed silently
    }
  },

  applyScheme() {
    const scheme =
      settings.color_scheme === "auto"
        ? buildSystemScheme(cachedOsIsDark)
        : getSchemeById(settings.color_scheme);
    if (scheme) {
      applyColorScheme(scheme);
    }
  },
};

if (typeof window !== "undefined") {
  // Get the correct OS theme on startup (fixes matchMedia being wrong in WKWebView)
  getCurrentWindow().theme().then((t) => {
    cachedOsIsDark = t === "dark";
    if (settings.color_scheme === "auto") {
      settingsState.applyScheme();
    }
  });

  getCurrentWindow().onThemeChanged(({ payload: theme }) => {
    cachedOsIsDark = theme === "dark";
    if (settings.color_scheme === "auto") {
      requestAnimationFrame(() => {
        settingsState.applyScheme();
      });
    }
  });
}
