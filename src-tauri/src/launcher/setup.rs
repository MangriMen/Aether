use aether_core::features::events::LauncherEvent;
use tauri::{App, Listener, Manager};

use crate::state::load_settings;

use super::ManualExitFlagStateInner;

pub fn setup_app(app: &mut App) -> tauri::Result<()> {
    app.manage(crate::state::AppState::default());
    app.manage(tokio::sync::Mutex::new(load_settings(app.handle().clone())));
    app.manage(std::sync::Mutex::new(ManualExitFlagStateInner(false)));

    let app_handle = app.handle().clone();

    app.listen(LauncherEvent::Process.as_str(), move |e| {
        super::handle_instance_launch(&app_handle, e);
    });

    Ok(())
}
