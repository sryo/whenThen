// Tray icon setup, panel show/hide, right-click menu.

#[cfg(target_os = "macos")]
pub mod macos_drag;

use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Listener, Manager,
};
use tauri_plugin_positioner::{Position, WindowExt};
use tracing::info;

const PANEL_LABEL: &str = "tray-panel";
const MAIN_LABEL: &str = "main";

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let show_item = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

    let icon = Image::from_path("icons/32x32.png")
        .or_else(|_| Image::from_path("icons/icon.png"))
        .unwrap_or_else(|_| Image::from_bytes(include_bytes!("../../icons/32x32.png")).expect("bundled tray icon"));

    let _tray = TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("whenThen")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                show_main_window(app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            let app = tray.app_handle();
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_panel(app);
            }
        })
        .build(app)?;

    #[cfg(target_os = "macos")]
    {
        if let Err(e) = macos_drag::register_drag_types(&_tray, app.clone()) {
            tracing::warn!("Could not register tray drag types: {e}");
        }
    }

    // Listen for "show main window" requests from the panel
    let app_for_listener = app.clone();
    app.listen("tray:show-main", move |_| {
        show_main_window(&app_for_listener);
    });

    info!("Tray icon ready");
    Ok(())
}

fn show_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(MAIN_LABEL) {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

fn toggle_panel(app: &AppHandle) {
    if let Some(panel) = app.get_webview_window(PANEL_LABEL) {
        if panel.is_visible().unwrap_or(false) {
            hide_panel(app);
        } else {
            show_panel(app);
        }
    }
}

pub fn show_panel(app: &AppHandle) {
    if let Some(panel) = app.get_webview_window(PANEL_LABEL) {
        // Position below tray icon
        let _ = panel.move_window(Position::TrayCenter);
        let _ = panel.show();
        let _ = panel.set_focus();
        let _ = app.emit("tray:panel-show", ());
    }
}

pub fn hide_panel(app: &AppHandle) {
    if let Some(panel) = app.get_webview_window(PANEL_LABEL) {
        let _ = app.emit("tray:panel-hide", ());
        let _ = panel.hide();
    }
}

pub fn set_visible(app: &AppHandle, visible: bool) {
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_visible(visible);
    }
    if !visible {
        hide_panel(app);
    }
}
