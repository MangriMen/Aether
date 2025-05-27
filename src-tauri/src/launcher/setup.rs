use aether_core::features::events::LauncherEvent;
use tauri::{App, Listener, Manager};

use crate::state::load_settings;

use super::{build_main_window, ManualExitFlagStateInner};

pub fn setup_app(app: &mut App) -> tauri::Result<()> {
    let loaded_settings = load_settings(app.handle().clone());

    app.manage(crate::state::AppState::default());
    app.manage(tokio::sync::Mutex::new(loaded_settings));
    app.manage(std::sync::Mutex::new(ManualExitFlagStateInner(false)));

    let app_handle = app.handle().clone();
    app.listen(LauncherEvent::Process.as_str(), move |e| {
        super::handle_instance_launch(&app_handle, e);
    });

    let app_handle = app.handle().clone();
    build_main_window(app_handle, loaded_settings.transparent, false)?;

    Ok(())
}
