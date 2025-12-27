use aether_core::features::auth::AccountData;
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
async fn create_offline_account(username: String) -> FrontendResult<AccountData> {
    Ok(aether_core::api::auth::create_offline_account(username).await?)
}

#[tauri::command]
async fn get_accounts() -> FrontendResult<Vec<AccountData>> {
    Ok(aether_core::api::auth::get_accounts().await?)
}

#[tauri::command]
async fn change_account(id: Uuid) -> FrontendResult<AccountData> {
    Ok(aether_core::api::auth::change_account(id).await?)
}

#[tauri::command]
async fn logout(id: Uuid) -> FrontendResult<()> {
    Ok(aether_core::api::auth::logout(id).await?)
}
