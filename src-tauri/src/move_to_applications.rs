// macOS: prompt to move app to /Applications on first run

use std::path::PathBuf;
use std::process::Command;
use tracing::{info, warn};

/// Returns the .app bundle path if running as a macOS app bundle.
fn get_app_bundle_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    // Binary is at: Foo.app/Contents/MacOS/foo
    // We need to go up 3 levels to get the .app bundle
    let macos_dir = exe.parent()?;
    let contents_dir = macos_dir.parent()?;
    let app_bundle = contents_dir.parent()?;

    // Verify it's actually a .app bundle
    if app_bundle.extension().map(|e| e == "app").unwrap_or(false) {
        Some(app_bundle.to_path_buf())
    } else {
        None
    }
}

/// Check if the app is already in /Applications.
fn is_in_applications(app_path: &std::path::Path) -> bool {
    let path_str = app_path.to_string_lossy();
    path_str.starts_with("/Applications/") || path_str.starts_with("/Applications\\")
}

/// Check if the app is running from a disk image (mounted DMG).
fn is_on_disk_image(app_path: &std::path::Path) -> bool {
    let path_str = app_path.to_string_lossy();
    path_str.starts_with("/Volumes/")
}

/// Move the app bundle to /Applications and relaunch.
fn move_and_relaunch(app_path: &std::path::Path) -> Result<(), String> {
    let app_name = app_path.file_name()
        .ok_or("Invalid app path")?
        .to_string_lossy();
    let dest = PathBuf::from("/Applications").join(&*app_name);

    // Remove existing app in /Applications if present
    if dest.exists() {
        std::fs::remove_dir_all(&dest)
            .map_err(|e| format!("Failed to remove existing app: {}", e))?;
    }

    // Copy the app bundle (can't move across volumes, e.g., from DMG)
    let status = Command::new("cp")
        .args(["-R", &app_path.to_string_lossy(), "/Applications/"])
        .status()
        .map_err(|e| format!("Failed to copy app: {}", e))?;

    if !status.success() {
        return Err("Copy command failed".to_string());
    }

    info!("App copied to /Applications");

    // Launch the new copy
    let _ = Command::new("open")
        .arg(&dest)
        .spawn();

    // Exit this instance
    std::process::exit(0);
}

/// Check if we should prompt to move, and handle the move if user agrees.
/// Returns true if the app was moved (and this process should exit).
pub fn check_and_prompt(_app: &tauri::App) -> bool {
    let Some(app_path) = get_app_bundle_path() else {
        info!("Not running as app bundle, skipping move check");
        return false;
    };

    if is_in_applications(&app_path) {
        info!("Already in /Applications");
        return false;
    }

    let on_dmg = is_on_disk_image(&app_path);
    let location = if on_dmg { "disk image" } else { "current location" };

    info!("App running from {}: {:?}", location, app_path);

    // Use native macOS dialog via osascript for synchronous prompt during setup
    let message = if on_dmg {
        "Move When to Applications folder? The app is currently running from a disk image."
    } else {
        "Move When to Applications folder?"
    };

    let script = format!(
        r#"display dialog "{}" buttons {{"No", "Move to Applications"}} default button "Move to Applications" with icon note"#,
        message
    );

    let output = Command::new("osascript")
        .args(["-e", &script])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.contains("Move to Applications") {
                match move_and_relaunch(&app_path) {
                    Ok(()) => true,
                    Err(e) => {
                        warn!("Failed to move app: {}", e);
                        // Show error dialog
                        let _ = Command::new("osascript")
                            .args(["-e", &format!(
                                r#"display dialog "Failed to move app: {}" buttons {{"OK"}} with icon stop"#,
                                e
                            )])
                            .output();
                        false
                    }
                }
            } else {
                info!("User declined to move app");
                false
            }
        }
        Err(e) => {
            warn!("Failed to show move dialog: {}", e);
            false
        }
    }
}
