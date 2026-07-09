use aether_core::features::auth::AuthFeature;
use tauri::State;
use uuid::Uuid;

use crate::{
    FrontendResult,
    core::ContainerState,
    features::auth::infra::tauri::dtos::AccountDto,
    shared::{
        IdempotencyManager, RequestId, TauriIdempotencyExt,
        commands::{AUTH_PLUGIN_NAME, auth_commands},
    },
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
async fn create_offline_account(
    username: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<AccountDto> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = container.0.clone();
    Ok(container
        .create_offline_account_use_case()
        .execute(username)
        .await
        .map_err(aether_core::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn list_accounts(container: State<'_, ContainerState>) -> FrontendResult<Vec<AccountDto>> {
    let container = container.0.clone();
    Ok(container
        .get_accounts_use_case()
        .execute()
        .await
        .map_err(aether_core::Error::from)?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn change_account(
    id: Uuid,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<AccountDto> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = container.0.clone();
    Ok(container
        .set_active_account_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn logout(
    id: Uuid,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = container.0.clone();
    Ok(container
        .logout_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?)
}
