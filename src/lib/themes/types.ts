export interface ColorScheme {
  id: string;
  name: string;
  variant: "dark" | "light";
  colors: {
    bg: string;
    bgSecondary: string;
    bgTertiary: string;
    text: string;
    textSecondary: string;
    textMuted: string;
    border: string;
    primary: string;
    primaryHover: string;
    success: string;
    warning: string;
    error: string;
    info: string;
    accent: string;
  };
}
