use aether_core::state::LauncherState;
use daedalus::minecraft;
use tauri::AppHandle;

use crate::utils::{self, result::AetherLauncherResult};

#[tauri::command]
pub async fn initialize_state(app: AppHandle) -> AetherLauncherResult<()> {
    let user_data_dir = Some(utils::tauri::get_app_dir(&app).to_str().unwrap().to_owned());

    let settings = aether_core::state::Settings {
        launcher_dir: user_data_dir.clone(),
        metadata_dir: user_data_dir.clone(),
    };

    aether_core::state::LauncherState::init(&settings).await?;

    Ok(())
}

#[tauri::command]
pub async fn get_minecraft_version_manifest() -> AetherLauncherResult<minecraft::VersionManifest> {
    let state = LauncherState::get().await?;
    let manifest = aether_core::launcher::download_version_manifest(&state, false).await?;

    Ok(manifest)
}
