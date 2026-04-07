use aether_core::features::events::LauncherEvent;
use tauri::{App, AppHandle, Listener, Manager};

use crate::{
    core::{build_main_window, instance_launch_listener, PreventExitStateInner},
    features::settings::{load_settings, set_window_effect, AppSettings},
};

pub(super) fn init_app(app: &mut App) -> tauri::Result<()> {
    let app_handle = app.handle();

    let app_settings = load_settings(app_handle.clone());

    init_app_state(app_handle.clone(), &app_settings);
    init_app_window(app_handle.clone(), &app_settings)?;
    init_instance_launch_listener(app_handle.clone());

    Ok(())
}

fn init_app_state(app_handle: AppHandle, app_settings: &AppSettings) {
    app_handle.manage(tokio::sync::Mutex::new(*app_settings));
    app_handle.manage(std::sync::Mutex::new(PreventExitStateInner::new(false)));
}

fn init_app_window(app_handle: AppHandle, settings: &AppSettings) -> tauri::Result<()> {
    build_main_window(app_handle.clone(), settings.transparent, false)?;
    set_window_effect(app_handle.clone(), settings.window_effect)?;
    Ok(())
}

// Prevent app exit when window closes after instance launched depends on app settings
fn init_instance_launch_listener(app_handle: AppHandle) {
    let app_handle_inner = app_handle.clone();
    app_handle.listen(LauncherEvent::Process.as_str(), move |e| {
        instance_launch_listener(app_handle_inner.clone(), e);
    });
}
