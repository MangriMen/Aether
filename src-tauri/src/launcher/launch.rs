use aether_core::features::events::{ProcessPayload, ProcessPayloadType};
use tauri::{AppHandle, Event, Manager, RunEvent, WebviewWindowBuilder};

use crate::state::{ActionOnInstanceLaunch, SettingsState};

pub(super) struct ManualExitFlagStateInner(pub(super) bool);
type ManualExitFlagState = std::sync::Mutex<ManualExitFlagStateInner>;

pub fn handle_instance_launch(app_handle: &AppHandle, e: Event) {
    let app_handle = app_handle.clone();
    let payload = serde_json::from_str::<ProcessPayload>(e.payload());

    tauri::async_runtime::spawn(async move {
        async fn on_instance_launch(app_handle: tauri::AppHandle) {
            let settings_state_handle = app_handle.state::<SettingsState>();
            let settings_state = settings_state_handle.lock().await;

            let action_on_launch = &settings_state.action_on_instance_launch;

            match action_on_launch {
                ActionOnInstanceLaunch::Hide => {
                    log::info!("Hide launcher");
                    for (_label, window) in app_handle.webview_windows() {
                        window.hide().unwrap();
                    }
                }
                ActionOnInstanceLaunch::Close => {
                    log::info!("Close launcher windows");

                    let manual_exit_flag = app_handle.state::<ManualExitFlagState>();

                    manual_exit_flag.lock().unwrap().0 = true;
                    for (_label, window) in app_handle.webview_windows() {
                        window.close().unwrap();
                    }
                }
                ActionOnInstanceLaunch::Nothing => {}
            }
        }

        async fn on_instance_finish(app_handle: tauri::AppHandle) {
            let settings_state_handle = app_handle.state::<SettingsState>();
            let settings_state = settings_state_handle.lock().await;

            let action_on_launch = &settings_state.action_on_instance_launch;

            match action_on_launch {
                ActionOnInstanceLaunch::Hide => {
                    log::info!("Show launcher");
                    for (_label, window) in app_handle.webview_windows() {
                        window.show().unwrap();
                    }
                }
                ActionOnInstanceLaunch::Close => {
                    log::info!("Create launcher windows");
                    for window_config in app_handle.config().app.windows.iter().filter(|w| w.create)
                    {
                        WebviewWindowBuilder::from_config(&app_handle, window_config)
                            .unwrap()
                            .build()
                            .unwrap();
                    }

                    let manual_exit_flag = app_handle.state::<ManualExitFlagState>();

                    manual_exit_flag.lock().unwrap().0 = false;
                }
                ActionOnInstanceLaunch::Nothing => {}
            }
        }

        match payload {
            Ok(payload) => match payload.event {
                ProcessPayloadType::Launched => {
                    on_instance_launch(app_handle.clone()).await;
                }
                ProcessPayloadType::Finished => {
                    on_instance_finish(app_handle.clone()).await;
                }
            },
            Err(err) => {
                log::error!("Failed to parse process event payload: {}", err);
            }
        }
    });
}

pub fn process_exit(app: &AppHandle, event: &RunEvent) {
    if let tauri::RunEvent::ExitRequested { api, .. } = event {
        let manual_exit_flag = app.state::<ManualExitFlagState>();
        let manual_exit_flag = manual_exit_flag.lock().unwrap();
        if manual_exit_flag.0 {
            api.prevent_exit();
        }
    }
}
