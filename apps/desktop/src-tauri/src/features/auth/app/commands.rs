use uuid::Uuid;

use crate::{
    FrontendResult,
    commands::{AUTH_PLUGIN_NAME, auth_commands},
    features::auth::AccountDto,
};

#[must_use]
pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new(AUTH_PLUGIN_NAME)
        .invoke_handler(auth_commands!(tauri::generate_handler!))
        .build()
}

#[must_use]
pub fn get_specta_commands<R: tauri::Runtime>() -> tauri_specta::Commands<R> {
    auth_commands!(tauri_specta::collect_commands!)
}

#[tauri::command]
#[specta::specta]
async fn create_offline_account(username: String) -> FrontendResult<AccountDto> {
    Ok(aether_core::api::auth::create_offline_account(username)
        .await
        .map_err(crate::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn list_accounts() -> FrontendResult<Vec<AccountDto>> {
    Ok(aether_core::api::auth::list_accounts()
        .await
        .map_err(crate::Error::from)?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn change_account(id: Uuid) -> FrontendResult<AccountDto> {
    Ok(aether_core::api::auth::change_account(id)
        .await
        .map_err(crate::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn logout(id: Uuid) -> FrontendResult<()> {
    Ok(aether_core::api::auth::logout(id)
        .await
        .map_err(crate::Error::from)?)
}
