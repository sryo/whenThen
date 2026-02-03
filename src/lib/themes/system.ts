// Builds a ColorScheme from OS system colors (AccentColor, Canvas, etc.)
import type { ColorScheme } from "./types";

type RGB = [number, number, number];

function resolveSystemColor(keyword: string): string {
  const el = document.createElement("div");
  el.style.color = keyword;
  document.body.appendChild(el);
  const color = getComputedStyle(el).color;
  el.remove();
  return color;
}

function parseRgb(raw: string): RGB {
  const m = raw.match(/(\d+),\s*(\d+),\s*(\d+)/);
  if (!m) return [128, 128, 128];
  return [parseInt(m[1]), parseInt(m[2]), parseInt(m[3])];
}

function toHex(rgb: RGB): string {
  return "#" + rgb.map((c) => c.toString(16).padStart(2, "0")).join("");
}

function mix(a: RGB, b: RGB, w: number): string {
  return toHex([
    Math.round(a[0] * (1 - w) + b[0] * w),
    Math.round(a[1] * (1 - w) + b[1] * w),
    Math.round(a[2] * (1 - w) + b[2] * w),
  ]);
}

function lighten(rgb: RGB, amount: number): string {
  return mix(rgb, [255, 255, 255], amount);
}

function darken(rgb: RGB, amount: number): string {
  return mix(rgb, [0, 0, 0], amount);
}

function luminance(rgb: RGB): number {
  return (rgb[0] * 0.299 + rgb[1] * 0.587 + rgb[2] * 0.114) / 255;
}

// Build color scheme from OS system colors, with WKWebView mismatch fallback.
export function buildSystemScheme(isDark: boolean): ColorScheme {
  let bg = parseRgb(resolveSystemColor("Canvas"));
  let text = parseRgb(resolveSystemColor("CanvasText"));
  const primary = parseRgb(resolveSystemColor("AccentColor"));
  let muted = parseRgb(resolveSystemColor("GrayText"));

  // Validate: if CSS thinks light but OS is dark (or vice-versa), override
  // all mode-dependent colors — not just bg/text but also muted text
  const cssDark = luminance(bg) < 0.5;
  if (cssDark !== isDark) {
    bg = isDark ? [30, 30, 30] : [255, 255, 255];
    text = isDark ? [255, 255, 255] : [0, 0, 0];
    muted = isDark ? [142, 142, 147] : [142, 142, 147];
  }

  // Derive shades by mixing bg toward text
  const bgSecondary = mix(bg, text, 0.04);
  const bgTertiary = mix(bg, text, 0.08);
  const textSecondary = mix(text, bg, 0.25);
  const border = mix(bg, text, 0.13);

  const primaryHover = isDark ? lighten(primary, 0.15) : darken(primary, 0.15);

  // Semantic status colors adapted for light/dark
  const success = isDark ? "#4ade80" : "#16a34a";
  const warning = isDark ? "#fbbf24" : "#d97706";
  const error = isDark ? "#f87171" : "#dc2626";
  const info = isDark ? lighten(primary, 0.25) : darken(primary, 0.1);
  const accent = isDark ? "#2dd4bf" : "#0d9488";

  return {
    id: "system",
    name: "System",
    variant: isDark ? "dark" : "light",
    colors: {
      bg: toHex(bg),
      bgSecondary,
      bgTertiary,
      text: toHex(text),
      textSecondary,
      textMuted: toHex(muted),
      border,
      primary: toHex(primary),
      primaryHover,
      success,
      warning,
      error,
      info,
      accent,
    },
  };
}
