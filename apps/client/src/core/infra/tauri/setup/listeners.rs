#![allow(clippy::needless_pass_by_value)]
use std::sync::Arc;

use aether_core::features::events::ProcessEvent;

use crate::{
    core::{AppSettingsStorageState, EventEmitterState, WindowManagerState},
    features::{events::DualEventEmitterExt, instance::InstanceLaunchListener},
};

pub fn setup_listeners(
    app_settings_storage: AppSettingsStorageState,
    window_manager: WindowManagerState<tauri::Wry>,
    event_emitter: EventEmitterState<tauri::Wry>,
) {
    let instance_launch_listener = Arc::new(InstanceLaunchListener::new(
        app_settings_storage.clone(),
        window_manager.clone(),
    ));

    event_emitter.on_core::<ProcessEvent, _>({
        let listener = instance_launch_listener.clone();

        move |event| {
            let listener_task = listener.clone();

            tauri::async_runtime::spawn(async move {
                listener_task.on_process_event(event).await;
            });
        }
    });
}
