use uuid::Uuid;

use crate::{features::auth::AccountDto, FrontendResult};

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
async fn create_offline_account(username: String) -> FrontendResult<AccountDto> {
    Ok(aether_core::api::auth::create_offline_account(username)
        .await?
        .into())
}

#[tauri::command]
async fn get_accounts() -> FrontendResult<Vec<AccountDto>> {
    Ok(aether_core::api::auth::get_accounts()
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[tauri::command]
async fn change_account(id: Uuid) -> FrontendResult<AccountDto> {
    Ok(aether_core::api::auth::change_account(id).await?.into())
}

#[tauri::command]
async fn logout(id: Uuid) -> FrontendResult<()> {
    Ok(aether_core::api::auth::logout(id).await?)
}
