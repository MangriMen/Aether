use tauri::{AppHandle, Manager, RunEvent};

use crate::core::PreventExitOnCloseState;

pub fn handle_app_event(app: &AppHandle, event: &RunEvent) {
    if let tauri::RunEvent::ExitRequested { api, .. } = event {
        let manual_exit_flag = app.state::<PreventExitOnCloseState>();
        let manual_exit_flag = manual_exit_flag.lock().unwrap();
        if manual_exit_flag.0 {
            api.prevent_exit();
        }
    }
}
