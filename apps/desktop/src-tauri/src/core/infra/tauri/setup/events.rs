use tauri::{AppHandle, Manager, RunEvent};

use crate::core::PreventExitState;

pub fn handle_app_events(app: &AppHandle, event: RunEvent) {
    if let RunEvent::ExitRequested { code, api, .. } = event {
        // Don't prevent user exit requests from the app
        if code.is_none() {
            return;
        }

        let prevent_exit_state = app.state::<PreventExitState>();

        if prevent_exit_state.is_prevented() {
            api.prevent_exit();
        }
    }
}
