// macOS dock badge integration (placeholder).

#[cfg(target_os = "macos")]
#[allow(dead_code)]
mod macos {
    /// Set the dock badge text.
    /// Note: Full implementation requires NSApplication/NSDockTile APIs.
    pub fn set_badge(_text: &str) {
        // Placeholder - badge support requires cocoa crate or objc bindings
    }
}

#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
mod macos {
    pub fn set_badge(_text: &str) {}
}

#[allow(dead_code)]
pub use macos::set_badge;

/// Update dock badge based on download status.
#[allow(dead_code)]
pub fn update_dock_status(active_count: usize, _overall_progress: f64) {
    if active_count == 0 {
        set_badge("");
    } else {
        set_badge(&format!("{}", active_count));
    }
}
