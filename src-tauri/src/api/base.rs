use aether_core::{core::LauncherState, features::events::EventState};
use tauri::AppHandle;

use crate::AetherLauncherResult;

#[tauri::command]
pub async fn initialize_state(app: AppHandle) -> AetherLauncherResult<()> {
    if LauncherState::initialized().await {
        return Ok(());
    }

    let launcher_dir = crate::utils::tauri::get_app_dir(&app);

    LauncherState::init(launcher_dir.clone(), launcher_dir).await?;
    EventState::init_with_app(app).await?;

    Ok(())
}

#[tauri::command]
pub async fn initialize_plugins() -> AetherLauncherResult<()> {
    if let Err(e) = aether_core::api::plugin::sync().await {
        log::error!("Failed to scan plugins: {}", e);
    }

    load_enabled_plugins().await?;

    Ok(())
}

#[tauri::command]
pub async fn load_enabled_plugins() -> AetherLauncherResult<()> {
    let settings = aether_core::api::settings::get().await?;

    for plugin_id in settings.enabled_plugins.iter() {
        if let Err(e) = aether_core::api::plugin::enable(plugin_id.to_string()).await {
            log::error!("Failed to load plugin {}: {}", plugin_id, e);
        }
    }

    Ok(())
}
