// macOS native drag-and-drop on the tray icon's NSStatusItem.
//
// The tray-icon crate (v0.21) doesn't expose the underlying NSStatusItem
// directly, so registering NSDraggingDestination drag types isn't possible
// without patching the crate or using runtime introspection to locate the
// status item's NSWindow. For now, drag-drop works by clicking the tray icon
// to open the panel, then dropping files onto playlet cards (HTML5 DnD).
//
// A future iteration can use NSApp's windows list to find the status bar
// window and register drag types on it.

use tauri::AppHandle;
use tauri::tray::TrayIcon;

pub fn register_drag_types(
    _tray: &TrayIcon,
    _app: AppHandle,
) -> Result<(), String> {
    // No-op until tray-icon exposes NSStatusItem access
    Ok(())
}
