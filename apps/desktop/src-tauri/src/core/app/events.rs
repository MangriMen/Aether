use tauri::{AppHandle, Manager, RunEvent};

use crate::core::PreventExitState;

pub fn handle_app_events(app: &AppHandle, event: RunEvent) {
    if let tauri::RunEvent::ExitRequested { api, .. } = event {
        let exit_on_close = app.state::<PreventExitState>();
        let exit_on_close = exit_on_close.lock().unwrap();

        if exit_on_close.is_prevented() {
            api.prevent_exit();
        }
    }
}
