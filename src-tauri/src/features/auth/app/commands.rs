use aether_core::features::auth::Account;
use uuid::Uuid;

use crate::FrontendResult;

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
pub async fn create_offline_account(username: String) -> FrontendResult<Uuid> {
    Ok(aether_core::api::auth::create_offline_account(username).await?)
}

#[tauri::command]
pub async fn get_accounts() -> FrontendResult<Vec<Account>> {
    Ok(aether_core::api::auth::get_accounts().await?)
}

#[tauri::command]
pub async fn change_account(id: Uuid) -> FrontendResult<()> {
    Ok(aether_core::api::auth::change_account(id).await?)
}

#[tauri::command]
pub async fn logout(id: Uuid) -> FrontendResult<()> {
    Ok(aether_core::api::auth::logout(id).await?)
}
