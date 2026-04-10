use log::error;
use tauri::{AppHandle, Manager};

use crate::{
    core::window_ops::{close_all_windows, hide_all_windows, recreate_windows, show_all_windows},
    features::settings::{ActionOnInstanceLaunch, AppSettingsStorage, AppSettingsStorageState},
};

/// Handles app behavior when Minecraft instance launches
pub async fn handle_instance_launch(app_handle: AppHandle) {
    let app_settings_storage = app_handle.state::<AppSettingsStorageState>();
    let app_settings = app_settings_storage.get().await;

    match app_settings {
        Ok(app_settings) => match app_settings.action_on_instance_launch {
            ActionOnInstanceLaunch::Hide => hide_all_windows(&app_handle),
            ActionOnInstanceLaunch::Close => close_all_windows(&app_handle),
            ActionOnInstanceLaunch::Nothing => {}
        },
        Err(err) => {
            error!("Failed to get app settings: {}", err);
        }
    }
}

/// Handles app behavior when Minecraft instance finishes
pub async fn handle_instance_finish(app_handle: AppHandle) {
    let app_settings_storage = app_handle.state::<AppSettingsStorageState>();
    let app_settings = app_settings_storage.get().await;

    match app_settings {
        Ok(app_settings) => match app_settings.action_on_instance_launch {
            ActionOnInstanceLaunch::Hide => show_all_windows(&app_handle),
            ActionOnInstanceLaunch::Close => recreate_windows(&app_handle, &app_settings),
            ActionOnInstanceLaunch::Nothing => {}
        },
        Err(err) => {
            error!("Failed to get app settings: {}", err);
        }
    }
}
