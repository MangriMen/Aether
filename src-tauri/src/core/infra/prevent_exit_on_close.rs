use aether_core::features::events::{ProcessEvent, ProcessEventType};
use tauri::{AppHandle, Event, Manager};

use crate::{
    core::{build_main_window, PreventExitOnCloseState},
    features::app_settings::{ActionOnInstanceLaunch, AppSettings, AppSettingsState},
};

/// Handles window behavior when Minecraft instance launches/finishes
pub fn prevent_exit_on_close_listener(app_handle: AppHandle, e: Event) {
    let app_handle = app_handle.clone();

    tauri::async_runtime::spawn(async move {
        match parse_process_event(&e) {
            Ok(event) => match event.event {
                ProcessEventType::Launched => handle_instance_launch(app_handle.clone()).await,
                ProcessEventType::Finished => handle_instance_finish(app_handle.clone()).await,
            },
            Err(err) => log::error!("Failed to parse process event: {}", err),
        }
    });
}

/// Parses the process event from the event payload
fn parse_process_event(e: &Event) -> Result<ProcessEvent, serde_json::Error> {
    serde_json::from_str::<ProcessEvent>(e.payload())
}

/// Handles window behavior when instance launches
async fn handle_instance_launch(app_handle: AppHandle) {
    let app_settings = app_handle.state::<AppSettingsState>();
    let app_settings = app_settings.lock().await;

    match app_settings.action_on_instance_launch {
        ActionOnInstanceLaunch::Hide => hide_all_windows(&app_handle),
        ActionOnInstanceLaunch::Close => close_all_windows(&app_handle),
        ActionOnInstanceLaunch::Nothing => {}
    }
}

/// Handles window behavior when instance finishes
async fn handle_instance_finish(app_handle: AppHandle) {
    let app_settings = app_handle.state::<AppSettingsState>();
    let app_settings = app_settings.lock().await;

    match app_settings.action_on_instance_launch {
        ActionOnInstanceLaunch::Hide => show_all_windows(&app_handle),
        ActionOnInstanceLaunch::Close => recreate_windows(&app_handle, &app_settings),
        ActionOnInstanceLaunch::Nothing => {}
    }
}

/// Hides all application windows
fn hide_all_windows(app_handle: &AppHandle) {
    log::info!("Hiding launcher windows");
    for window in app_handle.webview_windows().values() {
        if let Err(e) = window.hide() {
            log::error!("Failed to hide window: {}", e);
        }
    }
}

/// Shows all application windows
fn show_all_windows(app_handle: &AppHandle) {
    log::info!("Showing launcher windows");
    for window in app_handle.webview_windows().values() {
        if let Err(e) = window.show() {
            log::error!("Failed to show window: {}", e);
        }
    }
}

/// Closes all application windows and sets exit flag
fn close_all_windows(app_handle: &AppHandle) {
    log::info!("Closing launcher windows");

    let exit_flag = app_handle.state::<PreventExitOnCloseState>();
    exit_flag.lock().unwrap().0 = true;

    for window in app_handle.webview_windows().values() {
        if let Err(e) = window.close() {
            log::error!("Failed to close window: {}", e);
        }
    }
}

/// Recreates windows based on app configuration
fn recreate_windows(app_handle: &AppHandle, app_settings: &AppSettings) {
    log::info!("Recreating launcher windows");

    if let Err(e) = build_main_window(app_handle.clone(), app_settings.transparent, false) {
        log::error!("Failed to recreate window: {}", e);
    }

    let exit_flag = app_handle.state::<PreventExitOnCloseState>();
    exit_flag.lock().unwrap().0 = false;
}
