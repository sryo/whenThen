import type { ColorScheme } from "./types";

export function applyColorScheme(scheme: ColorScheme) {
  const root = document.documentElement;
  Object.entries(scheme.colors).forEach(([key, value]) => {
    const cssVar = `--color-${key.replace(/[A-Z]/g, (m) => "-" + m.toLowerCase())}`;
    root.style.setProperty(cssVar, value);
  });
  // Tell the browser the active mode so native elements (scrollbars, inputs,
  // selects, checkboxes) render with matching colors.
  root.style.setProperty("color-scheme", scheme.variant);
}
