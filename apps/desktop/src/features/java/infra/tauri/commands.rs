use std::{collections::HashSet, path::PathBuf};

use aether_core::{core::app::AetherContainer, features::java::JavaFeature};
use tauri::State;

use crate::{
    FrontendResult,
    features::java::{EditJavaDto, InstallJavaDto, JavaDto},
    shared::{
        IdempotencyManager, RequestId, TauriIdempotencyExt,
        commands::{JAVA_PLUGIN_NAME, java_commands},
    },
};

#[must_use]
pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new(JAVA_PLUGIN_NAME)
        .invoke_handler(java_commands!(tauri::generate_handler!))
        .build()
}

#[must_use]
pub fn get_specta_commands<R: tauri::Runtime>() -> tauri_specta::Commands<R> {
    java_commands!(tauri_specta::collect_commands!)
}

#[tauri::command]
#[specta::specta]
async fn list() -> FrontendResult<Vec<JavaDto>> {
    let container = AetherContainer::get();

    Ok(container
        .list_java_use_case()
        .execute()
        .await
        .map_err(aether_core::Error::from)?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn edit(
    edit_java: EditJavaDto,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<JavaDto> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();

    Ok(container
        .edit_java_use_case()
        .execute(edit_java.into())
        .await
        .map_err(aether_core::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn remove(
    version: u32,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();

    Ok(container
        .remove_java_use_case()
        .execute(version)
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn install(
    install_java: InstallJavaDto,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<JavaDto> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();

    Ok(container
        .install_java_use_case()
        .execute(install_java.into())
        .await
        .map_err(aether_core::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn test_jre(path: PathBuf) -> FrontendResult<JavaDto> {
    let container = AetherContainer::get();

    Ok(container
        .test_jre_use_case()
        .execute(path)
        .await
        .map_err(aether_core::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn discover() -> FrontendResult<Vec<JavaDto>> {
    let container = AetherContainer::get();

    Ok(container
        .discover_java_use_case()
        .execute()
        .await
        .map_err(aether_core::Error::from)?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn get_active_installations() -> FrontendResult<HashSet<u32>> {
    let container = AetherContainer::get();

    Ok(container
        .get_active_java_installations_use_case()
        .execute()
        .await)
}
