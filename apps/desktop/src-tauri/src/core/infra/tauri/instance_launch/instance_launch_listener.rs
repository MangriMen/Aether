#![allow(clippy::needless_pass_by_value)]
use aether_core::features::events::{ProcessEvent, ProcessEventType};
use tauri::AppHandle;

use super::instance_launch_handlers::{handle_instance_finish, handle_instance_launch};

/// Listens to process-related events and dispatches appropriate UI updates
pub fn instance_launch_listener(app_handle: AppHandle, event: ProcessEvent) {
    let app_handle = app_handle.clone();

    tauri::async_runtime::spawn(async move {
        match event.event {
            ProcessEventType::Launched => handle_instance_launch(app_handle).await,
            ProcessEventType::Finished => handle_instance_finish(app_handle).await,
        }
    });
}
