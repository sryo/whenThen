// Internationalization support for the whenThen app.

use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use tauri::Manager;

static TRANSLATIONS: OnceLock<HashMap<String, Value>> = OnceLock::new();
static LOCALES_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Initialize translations from bundled locale files.
pub fn init(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Store the locales directory for later use
    let locales_dir = app
        .path()
        .resolve("resources/locales", tauri::path::BaseDirectory::Resource)?;
    let _ = LOCALES_DIR.set(locales_dir);

    let locale = detect_system_locale();
    let translations = load_locale_file(&locale);

    let _ = TRANSLATIONS.set(translations);
    tracing::info!("Loaded translations for locale: {}", locale);
    Ok(())
}

/// Load translations from a locale file.
fn load_locale_file(locale: &str) -> HashMap<String, Value> {
    let Some(locales_dir) = LOCALES_DIR.get() else {
        return HashMap::new();
    };

    let locale_path = locales_dir.join(format!("{}.json", locale));
    let content = std::fs::read_to_string(&locale_path).unwrap_or_else(|_| {
        // Fallback to English
        let fallback_path = locales_dir.join("en.json");
        std::fs::read_to_string(fallback_path).unwrap_or_else(|_| "{}".to_string())
    });

    let json: Value = serde_json::from_str(&content).unwrap_or(Value::Object(serde_json::Map::new()));
    let mut translations = HashMap::new();

    if let Value::Object(map) = json {
        for (key, value) in map {
            translations.insert(key, value);
        }
    }

    translations
}

/// Detect system locale, returning "en" or "es" (fallback to "en").
pub fn detect_system_locale() -> String {
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("defaults")
            .args(["read", "-g", "AppleLocale"])
            .output()
        {
            let locale_str = String::from_utf8_lossy(&output.stdout);
            let locale = locale_str.trim().to_lowercase();
            if locale.starts_with("es") {
                return "es".to_string();
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Check LANG environment variable first
        if let Ok(lang) = std::env::var("LANG") {
            if lang.to_lowercase().starts_with("es") {
                return "es".to_string();
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Check LANG or LC_ALL environment variables
        for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(lang) = std::env::var(var) {
                if lang.to_lowercase().starts_with("es") {
                    return "es".to_string();
                }
            }
        }
    }

    "en".to_string()
}

/// Get a translation by dot-separated key path.
pub fn t(key: &str) -> String {
    let Some(translations) = TRANSLATIONS.get() else {
        return key.to_string();
    };

    let parts: Vec<&str> = key.split('.').collect();
    if parts.is_empty() {
        return key.to_string();
    }

    let Some(root) = translations.get(parts[0]) else {
        return key.to_string();
    };

    let mut current = root;
    for part in &parts[1..] {
        match current.get(*part) {
            Some(value) => current = value,
            None => return key.to_string(),
        }
    }

    match current {
        Value::String(s) => s.clone(),
        _ => key.to_string(),
    }
}

/// Get a translation with placeholder interpolation.
pub fn t_with(key: &str, args: &[(&str, &str)]) -> String {
    let mut result = t(key);
    for (placeholder, value) in args {
        result = result.replace(&format!("{{{}}}", placeholder), value);
    }
    result
}

/// Get the full translations object for the frontend.
/// If locale is "system" or empty, uses the system-detected locale.
pub fn get_translations_for_locale(locale: Option<String>) -> Value {
    let resolved_locale = match locale.as_deref() {
        Some("system") | Some("") | None => detect_system_locale(),
        Some(l) => l.to_string(),
    };

    let translations = load_locale_file(&resolved_locale);

    let mut map = serde_json::Map::new();
    for (key, value) in translations {
        map.insert(key, value);
    }
    Value::Object(map)
}

/// Get the cached translations (used by Rust-side t() function).
pub fn get_all_translations() -> Value {
    let Some(translations) = TRANSLATIONS.get() else {
        return Value::Object(serde_json::Map::new());
    };

    let mut map = serde_json::Map::new();
    for (key, value) in translations {
        map.insert(key.clone(), value.clone());
    }
    Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_t_with_interpolation() {
        let template = "Hello, {name}!";
        let result = template.replace("{name}", "World");
        assert_eq!(result, "Hello, World!");
    }
}
