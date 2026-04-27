use log::error;
use tauri::{AppHandle, Manager};

use crate::{
    core::{
        AppSettingsStorageState, WindowManagerState,
        window_ops::{hide_all_windows, show_all_windows},
    },
    features::settings::{ActionOnInstanceLaunch, AppSettingsStorage, WindowManager},
};

/// Handles app behavior when Minecraft instance launches
pub async fn handle_instance_launch(
    app_handle: AppHandle,
    window_manager: WindowManagerState<tauri::Wry>,
) {
    let storage = app_handle.state::<AppSettingsStorageState>();

    match storage.get().await.map_err(|e| e.to_string()) {
        Ok(settings) => match settings.action_on_instance_launch {
            ActionOnInstanceLaunch::Hide => hide_all_windows(&app_handle),
            ActionOnInstanceLaunch::Close => {
                window_manager.close_windows().await.unwrap_or_else(|err| {
                    error!("Failed to close windows before instance launch: {err}");
                });
            }
            ActionOnInstanceLaunch::Nothing => {}
        },
        Err(err) => error!("Failed to handle instance launch: {err}"),
    }
}

/// Handles app behavior when Minecraft instance finishes
pub async fn handle_instance_finish(
    app_handle: AppHandle,
    window_manager: WindowManagerState<tauri::Wry>,
) {
    let storage = app_handle.state::<AppSettingsStorageState>();

    match storage.get().await.map_err(|e| e.to_string()) {
        Ok(settings) => match settings.action_on_instance_launch {
            ActionOnInstanceLaunch::Hide => show_all_windows(&app_handle),
            ActionOnInstanceLaunch::Close => window_manager
                .create_windows(&settings)
                .await
                .unwrap_or_else(|err| {
                    error!("Failed to recreate windows after instance finish: {err}");
                }),
            ActionOnInstanceLaunch::Nothing => {}
        },
        Err(err) => {
            error!("Failed to handle instance finish: {err}");
            show_all_windows(&app_handle);
        }
    }
}
