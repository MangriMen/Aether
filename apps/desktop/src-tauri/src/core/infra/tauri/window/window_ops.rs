use log::warn;
use tauri::{AppHandle, Manager, WebviewWindow};

use crate::{
    core::{MAIN_WINDOW_LABEL, PreventExitState, build_main_window},
    features::settings::AppSettings,
};

/// Hides all windows
pub fn hide_all_windows(app_handle: &AppHandle) {
    log::info!("Hiding launcher windows");
    for_each_window(app_handle, tauri::WebviewWindow::hide, "hide");
}

/// Shows all windows
pub fn show_all_windows(app_handle: &AppHandle) {
    log::info!("Showing launcher windows");
    for_each_window(app_handle, tauri::WebviewWindow::show, "show");

    if let Some(main_window) = app_handle.get_webview_window(MAIN_WINDOW_LABEL)
        && let Err(err) = main_window.set_focus()
    {
        warn!("Failed to focus main window: {err}");
    }
}

/// Closes all windows and prevents exit handling
pub fn close_all_windows<R: tauri::Runtime>(app_handle: &AppHandle<R>) {
    log::info!("Closing launcher windows");

    let prevent_exit_state = app_handle.state::<PreventExitState>();
    prevent_exit_state.set_prevented(true);

    for_each_window(app_handle, tauri::WebviewWindow::close, "close");
}

/// Recreates the main window
pub fn recreate_windows<R: tauri::Runtime>(app_handle: &AppHandle<R>, app_settings: &AppSettings) {
    log::info!("Recreating launcher windows");

    if let Err(e) = build_main_window(app_handle.clone(), app_settings.transparent, false) {
        log::error!("Failed to recreate window: {e}");
    }

    let prevent_exit_state = app_handle.state::<PreventExitState>();
    prevent_exit_state.set_prevented(false);
}

/// Helper to apply an action to all windows with logging
fn for_each_window<R: tauri::Runtime, F>(app: &AppHandle<R>, action: F, action_name: &str)
where
    F: Fn(&WebviewWindow<R>) -> tauri::Result<()>,
{
    for (label, window) in &app.webview_windows() {
        if let Err(e) = action(window) {
            log::error!("Failed to {action_name} window '{label}': {e}");
        }
    }
}
