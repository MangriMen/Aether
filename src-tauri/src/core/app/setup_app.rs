use aether_core::features::events::LauncherEvent;
use tauri::{App, AppHandle, Listener, Manager};

use crate::{
    core::{build_main_window, prevent_exit_on_close_listener, PreventExitOnCloseStateInner},
    features::app_settings::{load_settings, set_window_effect, AppSettings},
};

pub fn setup_app(app: &mut App) -> tauri::Result<()> {
    let app_handle = app.handle();

    let app_settings = load_settings(app_handle.clone());

    setup_app_state(app_handle.clone(), &app_settings);
    setup_app_window(app_handle.clone(), &app_settings)?;
    setup_prevent_window_on_close_listener(app_handle.clone());

    Ok(())
}

fn setup_app_state(app_handle: AppHandle, app_settings: &AppSettings) {
    app_handle.manage(tokio::sync::Mutex::new(*app_settings));
    app_handle.manage(std::sync::Mutex::new(PreventExitOnCloseStateInner(false)));
}

fn setup_app_window(app_handle: AppHandle, settings: &AppSettings) -> tauri::Result<()> {
    build_main_window(app_handle.clone(), settings.transparent, false)?;
    set_window_effect(app_handle.clone(), settings.window_effect)?;
    Ok(())
}

fn setup_prevent_window_on_close_listener(app_handle: AppHandle) {
    let app_handle_inner = app_handle.clone();
    app_handle.listen(LauncherEvent::Process.as_str(), move |e| {
        prevent_exit_on_close_listener(app_handle_inner.clone(), e);
    });
}
