use aether_core::state::MinecraftProcessMetadata;

use crate::AetherLauncherResult;

#[tauri::command]
pub async fn get_running_minecraft_instances() -> AetherLauncherResult<Vec<MinecraftProcessMetadata>>
{
    Ok(aether_core::api::process::get_all().await?)
}

#[tauri::command]
pub async fn get_minecraft_instance_process(
    id: String,
) -> AetherLauncherResult<Vec<MinecraftProcessMetadata>> {
    Ok(aether_core::api::process::get_by_instance_id(&id).await?)
}
