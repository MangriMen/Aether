use std::{path::Path, sync::Arc};

use aether_core::core::LauncherState;
use tauri::AppHandle;

use crate::{features::events::TauriEventEmitter, shared, FrontendResult};

#[tauri::command]
pub async fn initialize_state(app: AppHandle) -> FrontendResult<()> {
    if LauncherState::initialized().await {
        return Ok(());
    }

    let launcher_dir = shared::get_app_dir(&app);
    let event_emitter = Arc::new(TauriEventEmitter::new(app));
    LauncherState::init(launcher_dir.clone(), launcher_dir, event_emitter).await?;

    Ok(())
}

#[tauri::command]
pub async fn initialize_plugins() -> FrontendResult<()> {
    aether_core::api::plugin::sync().await?;
    load_enabled_plugins().await?;
    Ok(())
}

#[tauri::command]
pub async fn load_enabled_plugins() -> FrontendResult<()> {
    let settings = aether_core::api::settings::get().await?;

    for plugin_id in settings.enabled_plugins().iter() {
        if let Err(e) = aether_core::api::plugin::enable(plugin_id.to_string()).await {
            log::error!("Failed to load plugin {}: {}", plugin_id, e);
        }
    }

    Ok(())
}

#[tauri::command]
pub fn reveal_in_explorer(path: String, exact: bool) -> FrontendResult<()> {
    Ok(shared::reveal_in_explorer(Path::new(&path), exact)?)
}
