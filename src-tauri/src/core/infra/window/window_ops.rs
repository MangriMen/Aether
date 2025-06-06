use tauri::{AppHandle, Manager, WebviewWindow};

use crate::{
    core::{build_main_window, PreventExitState},
    features::app_settings::AppSettings,
};

/// Hides all windows
pub fn hide_all_windows(app_handle: &AppHandle) {
    log::info!("Hiding launcher windows");
    for_each_window(app_handle, |w| w.hide(), "hide");
}

/// Shows all windows
pub fn show_all_windows(app_handle: &AppHandle) {
    log::info!("Showing launcher windows");
    for_each_window(app_handle, |w| w.show(), "show");
}

/// Closes all windows and prevents exit handling
pub fn close_all_windows(app_handle: &AppHandle) {
    log::info!("Closing launcher windows");

    let prevent_exit = app_handle.state::<PreventExitState>();
    let mut prevent_exit = prevent_exit.lock().unwrap();
    prevent_exit.set(true);

    for_each_window(app_handle, |w| w.close(), "close");
}

/// Recreates the main window
pub fn recreate_windows(app_handle: &AppHandle, app_settings: &AppSettings) {
    log::info!("Recreating launcher windows");

    if let Err(e) = build_main_window(app_handle.clone(), app_settings.transparent, false) {
        log::error!("Failed to recreate window: {}", e);
    }

    let prevent_exit = app_handle.state::<PreventExitState>();
    let mut prevent_exit = prevent_exit.lock().unwrap();
    prevent_exit.set(false);
}

/// Helper to apply an action to all windows with logging
fn for_each_window<F>(app: &AppHandle, action: F, action_name: &str)
where
    F: Fn(&WebviewWindow) -> tauri::Result<()>,
{
    for window in app.webview_windows().values() {
        if let Err(e) = action(window) {
            log::error!("Failed to {} window: {}", action_name, e);
        }
    }
}
