use aether_core::features::events::{ProcessEvent, ProcessEventType};
use tauri::{AppHandle, Event};

use super::instance_launch_handlers::{handle_instance_finish, handle_instance_launch};

/// Listens to process-related events and dispatches appropriate UI updates
pub fn instance_launch_listener(app_handle: AppHandle, e: Event) {
    let app_handle = app_handle.clone();

    tauri::async_runtime::spawn(async move {
        match parse_instance_launch_event(&e) {
            Ok(event) => match event.event {
                ProcessEventType::Launched => handle_instance_launch(app_handle.clone()).await,
                ProcessEventType::Finished => handle_instance_finish(app_handle.clone()).await,
            },
            Err(err) => log::error!("Failed to parse process event: {}", err),
        }
    });
}

/// Parses a ProcessEvent from Tauri event
fn parse_instance_launch_event(e: &Event) -> Result<ProcessEvent, serde_json::Error> {
    serde_json::from_str::<ProcessEvent>(e.payload())
}
