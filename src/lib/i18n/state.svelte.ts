// Reactive i18n state for Svelte 5
import { loadTranslations, t as coreT, getLocale } from "./index";

let version = $state(0);

export const i18n = {
  get locale(): string {
    // Reference version to establish reactivity dependency
    void version;
    return getLocale();
  },

  async setLocale(locale: string): Promise<void> {
    await loadTranslations(locale);
    version += 1;
  },

  /**
   * Reactive translation lookup. References version counter to trigger
   * Svelte reactivity when locale changes.
   */
  t(key: string, args?: Record<string, string | number>): string {
    // Reference version to establish reactivity dependency
    void version;
    return coreT(key, args);
  },
};
