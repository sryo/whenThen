// Tray icon setup, right-click menu, left-click shows main window.

use std::sync::atomic::Ordering;

use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Listener, Manager,
};

use crate::i18n::t;
use crate::state::AppState;
use tracing::info;

const MAIN_LABEL: &str = "main";

// Embed both icon variants
const ICON_NORMAL: &[u8] = include_bytes!("../../icons/tray.png");
const ICON_ACTIVE: &[u8] = include_bytes!("../../icons/tray-active.png");

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let show_item = MenuItem::with_id(app, "show", t("tray.showWindow"), true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", t("tray.quit"), true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

    let icon = Image::from_bytes(ICON_NORMAL).expect("bundled tray icon");

    let _tray = TrayIconBuilder::with_id("main")
        .icon(icon)
        .icon_as_template(true)
        .tooltip("When")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                show_main_window(app);
            }
            "quit" => {
                let state = app.state::<AppState>();
                state.quit_requested.store(true, Ordering::SeqCst);
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
                show_main_window(app);
            }
        })
        .build(app)?;

    // Listen for pending count changes to update icon
    let app_handle = app.clone();
    app.listen("rss:pending-count", move |event| {
        if let Ok(count) = event.payload().parse::<usize>() {
            set_icon_active(&app_handle, count > 0);
        }
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

pub fn set_visible(app: &AppHandle, visible: bool) {
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_visible(visible);
    }
}

/// Update tray icon to show active state (colored) when there are pending matches.
pub fn set_icon_active(app: &AppHandle, active: bool) {
    if let Some(tray) = app.tray_by_id("main") {
        let (icon_bytes, as_template) = if active {
            (ICON_ACTIVE, false) // Colored icon, not a template
        } else {
            (ICON_NORMAL, true) // Normal template icon
        };

        if let Ok(icon) = Image::from_bytes(icon_bytes) {
            let _ = tray.set_icon(Some(icon));
            let _ = tray.set_icon_as_template(as_template);
        }
    }
}
