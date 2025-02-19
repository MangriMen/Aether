use crate::AetherLauncherResult;

#[tauri::command]
pub async fn call_plugin(id: String, data: String) -> AetherLauncherResult<()> {
    Ok(aether_core::api::plugin::call(&id, &data).await?)
}
