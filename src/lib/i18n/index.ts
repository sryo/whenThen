// i18n core: translation loading and lookup
import { getTranslations } from "$lib/services/tauri-commands";

type TranslationValue = string | Record<string, unknown>;
type Translations = Record<string, TranslationValue>;

let translations: Translations = {};
let currentLocale = "en";

/**
 * Load translations for the given locale from Tauri backend.
 * If no locale provided, uses the current locale.
 */
export async function loadTranslations(locale?: string): Promise<void> {
  if (locale) {
    currentLocale = locale;
  }
  try {
    const result = await getTranslations(currentLocale);
    translations = result as Translations;
  } catch (e) {
    console.error("Failed to load translations:", e);
    translations = {};
  }
}

/**
 * Get a translated string by key (dot-notation).
 * Supports placeholder replacement with {name} syntax.
 */
export function t(key: string, args?: Record<string, string | number>): string {
  const parts = key.split(".");
  let value: unknown = translations;

  for (const part of parts) {
    if (value && typeof value === "object" && part in value) {
      value = (value as Record<string, unknown>)[part];
    } else {
      return key;
    }
  }

  if (typeof value !== "string") {
    return key;
  }

  if (!args) {
    return value;
  }

  return value.replace(/\{(\w+)\}/g, (_, name) => {
    const replacement = args[name];
    return replacement !== undefined ? String(replacement) : `{${name}}`;
  });
}

/**
 * Get the current locale code.
 */
export function getLocale(): string {
  return currentLocale;
}
