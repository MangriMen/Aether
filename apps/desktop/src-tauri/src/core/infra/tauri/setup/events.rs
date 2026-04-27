use log::trace;
use tauri::{AppHandle, Manager, RunEvent};

use crate::core::PreventExitState;

pub fn handle_app_events(app: &AppHandle, event: RunEvent) {
    if let RunEvent::ExitRequested { code, api, .. } = event {
        trace!("Exit requested: {code:?}");

        let prevent_exit_state = app.state::<PreventExitState>();

        trace!("Prevent exit state: {}", prevent_exit_state.is_prevented());

        if prevent_exit_state.is_prevented() {
            api.prevent_exit();
        }
    }
}
