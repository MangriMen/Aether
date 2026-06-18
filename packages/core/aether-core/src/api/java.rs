use std::sync::Arc;

use crate::{
    core::LazyLocator,
    features::java::{
        GetJavaUseCase, InstallJava, InstallJavaUseCase, Java,
        infra::{AzulJreProvider, FsJavaInstallationService},
    },
};

#[tracing::instrument]
pub async fn install(version: u32) -> crate::Result<Java> {
    let locator = LazyLocator::get().await?;

    let jre_provider = Arc::new(AzulJreProvider::new(
        locator.get_progress_service().await,
        locator.get_request_client().await,
    ));

    let install_java_use_case = InstallJavaUseCase::new(
        locator.get_java_storage().await,
        FsJavaInstallationService,
        jre_provider,
        locator.location_info.clone(),
        locator.get_java_installation_tracker().await,
    );

    Ok(install_java_use_case
        .execute(InstallJava::new(version))
        .await?)
}

pub async fn get(version: u32) -> crate::Result<Java> {
    let locator = LazyLocator::get().await?;

    let get_java_use_case =
        GetJavaUseCase::new(locator.get_java_storage().await, FsJavaInstallationService);

    Ok(get_java_use_case.execute(version).await?)
}
