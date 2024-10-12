use aether_core::state::{Instance, LauncherState};
use daedalus::minecraft;
use tauri::AppHandle;

use crate::{
    models::minecraft::InstanceCreateDto,
    utils::{self, result::AetherLauncherResult},
};

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

#[tauri::command]
pub async fn create_minecraft_instance(
    instance_create_dto: InstanceCreateDto,
) -> AetherLauncherResult<String> {
    let path = aether_core::api::instance::instance_create(
        instance_create_dto.name,
        instance_create_dto.game_version,
        instance_create_dto.mod_loader,
        instance_create_dto.loader_version,
        instance_create_dto.icon_path,
        instance_create_dto.linked_data,
        instance_create_dto.skip_install_profile,
    )
    .await?;

    Ok(path)
}

#[tauri::command]
pub async fn get_minecraft_instances() -> AetherLauncherResult<Vec<Instance>> {
    Ok(aether_core::api::instance::get_instances().await?)
}

#[tauri::command]
pub async fn launch_minecraft_instance(name: String) -> AetherLauncherResult<()> {
    aether_core::api::instance::run(&name).await?;

    Ok(())
}
