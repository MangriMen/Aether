use std::sync::Arc;

use crate::{
    core::LazyLocator,
    features::{
        instance::{
            Instance,
            app::{
                CreateInstanceUseCase, EditInstance, EditInstanceUseCase, GetInstanceUseCase,
                InstallInstanceUseCase, ListInstancesUseCase, NewInstance, RemoveInstanceUseCase,
                UpdateInstanceUseCase,
            },
        },
        java::{
            app::{GetJavaUseCase, InstallJavaUseCase},
            infra::{AzulJreProvider, FsJavaInstallationService},
        },
        minecraft::{
            LoaderVersionResolver,
            app::{GetVersionManifestUseCase, InstallMinecraftUseCase},
            infra::{
                AssetsService, ClientService, LibrariesService, MinecraftDownloadResolver,
                MinecraftDownloadService,
            },
        },
    },
    shared::FileCache,
};

#[tracing::instrument]
pub async fn create(new_instance: NewInstance) -> crate::Result<String> {
    let locator = LazyLocator::get().await?;

    let loader_version_resolver = Arc::new(LoaderVersionResolver::new(
        locator.get_metadata_storage().await,
    ));

    let get_loader_manifest_use_case = Arc::new(GetVersionManifestUseCase::new(
        locator.get_metadata_storage().await,
    ));

    let minecraft_cache = Arc::new(FileCache::new(MinecraftDownloadResolver::new(
        locator.location_info.clone(),
    )));

    let client_service = ClientService::new(
        locator.get_progress_service().await,
        locator.get_request_client().await,
        minecraft_cache.clone(),
    );

    let assets_service = AssetsService::new(
        locator.get_progress_service().await,
        locator.get_request_client().await,
        locator.location_info.clone(),
        minecraft_cache.clone(),
    );
    let libraries_service = LibrariesService::new(
        locator.get_progress_service().await,
        locator.get_request_client().await,
        locator.location_info.clone(),
    );
    let minecraft_download_service = MinecraftDownloadService::new(
        client_service,
        assets_service,
        libraries_service,
        locator.get_request_client().await,
        locator.get_progress_service().await,
        minecraft_cache.clone(),
    );

    let get_java_use_case = Arc::new(GetJavaUseCase::new(
        locator.get_java_storage().await,
        FsJavaInstallationService,
    ));

    let jre_provider = Arc::new(AzulJreProvider::new(
        locator.get_progress_service().await,
        locator.get_request_client().await,
    ));

    let install_java_use_case = Arc::new(InstallJavaUseCase::new(
        locator.get_java_storage().await,
        FsJavaInstallationService,
        jre_provider,
        locator.location_info.clone(),
        locator.get_java_installation_tracker().await,
    ));

    let install_minecraft_use_case = Arc::new(InstallMinecraftUseCase::new(
        locator.get_progress_service().await,
        loader_version_resolver.clone(),
        get_loader_manifest_use_case.clone(),
        locator.location_info.clone(),
        minecraft_download_service,
        FsJavaInstallationService,
        get_java_use_case.clone(),
        install_java_use_case.clone(),
    ));

    let install_instance_use_case = Arc::new(InstallInstanceUseCase::new(
        locator.get_instance_storage().await,
        install_minecraft_use_case,
        locator.get_progress_service().await,
        locator.location_info.clone(),
    ));

    Ok(CreateInstanceUseCase::new(
        locator.get_instance_storage().await,
        loader_version_resolver,
        install_instance_use_case,
        locator.location_info.clone(),
        locator.get_event_emitter().await,
        locator.get_instance_watcher_service().await?,
    )
    .execute(new_instance)
    .await?)
}

#[tracing::instrument]
pub async fn install(instance_id: String, force: bool) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    let loader_version_resolver = Arc::new(LoaderVersionResolver::new(
        locator.get_metadata_storage().await,
    ));

    let get_loader_manifest_use_case = Arc::new(GetVersionManifestUseCase::new(
        locator.get_metadata_storage().await,
    ));

    let minecraft_cache = Arc::new(FileCache::new(MinecraftDownloadResolver::new(
        locator.location_info.clone(),
    )));

    let client_service = ClientService::new(
        locator.get_progress_service().await,
        locator.get_request_client().await,
        minecraft_cache.clone(),
    );

    let assets_service = AssetsService::new(
        locator.get_progress_service().await,
        locator.get_request_client().await,
        locator.location_info.clone(),
        minecraft_cache.clone(),
    );
    let libraries_service = LibrariesService::new(
        locator.get_progress_service().await,
        locator.get_request_client().await,
        locator.location_info.clone(),
    );
    let minecraft_download_service = MinecraftDownloadService::new(
        client_service,
        assets_service,
        libraries_service,
        locator.get_request_client().await,
        locator.get_progress_service().await,
        minecraft_cache.clone(),
    );

    let get_java_use_case = Arc::new(GetJavaUseCase::new(
        locator.get_java_storage().await,
        FsJavaInstallationService,
    ));

    let jre_provider = Arc::new(AzulJreProvider::new(
        locator.get_progress_service().await,
        locator.get_request_client().await,
    ));

    let install_java_use_case = Arc::new(InstallJavaUseCase::new(
        locator.get_java_storage().await,
        FsJavaInstallationService,
        jre_provider,
        locator.location_info.clone(),
        locator.get_java_installation_tracker().await,
    ));

    let install_minecraft_use_case = Arc::new(InstallMinecraftUseCase::new(
        locator.get_progress_service().await,
        loader_version_resolver.clone(),
        get_loader_manifest_use_case.clone(),
        locator.location_info.clone(),
        minecraft_download_service,
        FsJavaInstallationService,
        get_java_use_case.clone(),
        install_java_use_case.clone(),
    ));

    Ok(InstallInstanceUseCase::new(
        locator.get_instance_storage().await,
        install_minecraft_use_case,
        locator.get_progress_service().await,
        locator.location_info.clone(),
    )
    .execute(instance_id, force)
    .await?)
}

#[tracing::instrument]
pub async fn update(instance_id: String) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    Ok(UpdateInstanceUseCase::new(
        locator.get_instance_storage().await,
        locator.get_updaters_registry().await,
    )
    .execute(instance_id)
    .await?)
}

pub async fn list() -> crate::Result<Vec<Instance>> {
    let locator = LazyLocator::get().await?;

    Ok(
        ListInstancesUseCase::new(locator.get_instance_storage().await)
            .execute()
            .await?,
    )
}

pub async fn get(instance_id: String) -> crate::Result<Instance> {
    let locator = LazyLocator::get().await?;

    Ok(
        GetInstanceUseCase::new(locator.get_instance_storage().await)
            .execute(instance_id)
            .await?,
    )
}

#[tracing::instrument]
pub async fn edit(instance_id: String, edit_instance: EditInstance) -> crate::Result<Instance> {
    let locator = LazyLocator::get().await?;

    Ok(
        EditInstanceUseCase::new(locator.get_instance_storage().await)
            .execute(instance_id, edit_instance)
            .await?,
    )
}

#[tracing::instrument]
pub async fn remove(instance_id: String) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    Ok(RemoveInstanceUseCase::new(
        locator.get_instance_storage().await,
        locator.get_instance_watcher_service().await?,
    )
    .execute(instance_id)
    .await?)
}
