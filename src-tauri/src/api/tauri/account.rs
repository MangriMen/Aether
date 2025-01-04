use aether_core::state::AccountState;
use uuid::Uuid;

use crate::utils::result::AetherLauncherResult;

#[tauri::command]
pub async fn get_account_state() -> AetherLauncherResult<AccountState> {
    Ok(aether_core::api::credentials::get_accounts().await?)
}

#[tauri::command]
pub async fn create_offline_account(username: String) -> AetherLauncherResult<()> {
    Ok(aether_core::api::credentials::create_offline_account(&username).await?)
}

#[tauri::command]
pub async fn change_account(id: Uuid) -> AetherLauncherResult<()> {
    Ok(aether_core::api::credentials::change_account(&id).await?)
}

#[tauri::command]
pub async fn logout(id: Uuid) -> AetherLauncherResult<()> {
    Ok(aether_core::api::credentials::logout(&id).await?)
}
