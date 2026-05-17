use std::sync::Arc;

use aether_core::{
    core::{LauncherState, domain::LazyLocator},
    features::java::{
        app::{InstallJavaUseCase, ListJavaUseCase, TestJreUseCase},
        infra::{AzulJreProvider, FsJavaInstallationService},
    },
};

use crate::{
    FrontendResult,
    features::java::{JavaDto, TestJreVersionDto},
    shared::commands::{JAVA_PLUGIN_NAME, java_commands},
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
    let lazy_locator = LazyLocator::get().await.map_err(crate::Error::from)?;

    Ok(ListJavaUseCase::new(lazy_locator.get_java_storage().await)
        .execute()
        .await
        .map_err(aether_core::Error::from)
        .map_err(crate::Error::from)?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn install(version: u32) -> FrontendResult<JavaDto> {
    let lazy_locator = LazyLocator::get().await.map_err(crate::Error::from)?;
    let state = LauncherState::get().await.map_err(crate::Error::from)?;

    let jre_provider = Arc::new(AzulJreProvider::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
    ));

    Ok(InstallJavaUseCase::new(
        lazy_locator.get_java_storage().await,
        FsJavaInstallationService,
        jre_provider,
        state.location_info.clone(),
    )
    .execute(version)
    .await
    .map_err(aether_core::Error::from)
    .map_err(crate::Error::from)?
    .into())
}

#[tauri::command]
#[specta::specta]
async fn test(test_version: TestJreVersionDto) -> FrontendResult<JavaDto> {
    Ok(TestJreUseCase::new(Arc::new(FsJavaInstallationService {}))
        .execute(test_version.into())
        .await
        .map_err(aether_core::Error::from)
        .map_err(crate::Error::from)?
        .into())
}
