use log::error;
use tauri::{AppHandle, Manager};

use crate::{
    core::{
        AppSettingsStorageState,
        window_ops::{close_all_windows, hide_all_windows, recreate_windows, show_all_windows},
    },
    features::settings::{ActionOnInstanceLaunch, AppSettings, AppSettingsStorage},
};

/// Handles app behavior when Minecraft instance launches
pub async fn handle_instance_launch(app_handle: AppHandle) {
    match get_settings(app_handle.clone()).await {
        Ok(settings) => match settings.action_on_instance_launch {
            ActionOnInstanceLaunch::Hide => hide_all_windows(&app_handle),
            ActionOnInstanceLaunch::Close => close_all_windows(&app_handle),
            ActionOnInstanceLaunch::Nothing => {}
        },
        Err(err) => error!("Failed to handle instance launch: {err}"),
    }
}

/// Handles app behavior when Minecraft instance finishes
pub async fn handle_instance_finish(app_handle: AppHandle) {
    match get_settings(app_handle.clone()).await {
        Ok(settings) => match settings.action_on_instance_launch {
            ActionOnInstanceLaunch::Hide => show_all_windows(&app_handle),
            ActionOnInstanceLaunch::Close => recreate_windows(&app_handle, &settings),
            ActionOnInstanceLaunch::Nothing => {}
        },
        Err(err) => {
            error!("Failed to handle instance finish: {err}");
            show_all_windows(&app_handle);
        }
    }
}

async fn get_settings<R: tauri::Runtime>(app_handle: AppHandle<R>) -> Result<AppSettings, String> {
    let storage = app_handle.state::<AppSettingsStorageState>();
    storage.get().await.map_err(|e| e.to_string())
}
