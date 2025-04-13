use aether_core::features::auth::Account;
use uuid::Uuid;

use crate::AetherLauncherResult;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("auth")
        .invoke_handler(tauri::generate_handler![
            get_accounts,
            create_offline_account,
            change_account,
            logout,
        ])
        .build()
}

#[tauri::command]
pub async fn get_accounts() -> AetherLauncherResult<Vec<Account>> {
    Ok(aether_core::api::credentials::get_accounts().await?)
}

#[tauri::command]
pub async fn create_offline_account(username: String) -> AetherLauncherResult<Uuid> {
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
