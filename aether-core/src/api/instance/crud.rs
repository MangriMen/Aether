use std::sync::Arc;

use crate::{
    core::{domain::LazyLocator, LauncherState},
    features::{
        instance::{
            app::{
                CreateInstanceUseCase, EditInstance, EditInstanceUseCase, GetInstanceUseCase,
                InstallInstanceUseCase, ListInstancesUseCase, NewInstance, RemoveInstanceUseCase,
                UpdateInstanceUseCase,
            },
            Instance,
        },
        java::{
            app::{GetJavaUseCase, InstallJavaUseCase},
            infra::{AzulJreProvider, FsJavaInstallationService},
        },
        minecraft::{
            app::{GetVersionManifestUseCase, InstallMinecraftUseCase},
            infra::{
                AssetsService, ClientService, LibrariesService, MinecraftDownloadResolver,
                MinecraftDownloadService,
            },
            LoaderVersionResolver,
        },
    },
    shared::FileCache,
};

#[tracing::instrument]
pub async fn create(new_instance: NewInstance) -> crate::Result<String> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    let loader_version_resolver = Arc::new(LoaderVersionResolver::new(
        lazy_locator.get_metadata_storage().await,
    ));

    let get_loader_manifest_use_case = Arc::new(GetVersionManifestUseCase::new(
        lazy_locator.get_metadata_storage().await,
    ));

    let minecraft_cache = Arc::new(FileCache::new(MinecraftDownloadResolver::new(
        state.location_info.clone(),
    )));

    let client_service = ClientService::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
        minecraft_cache.clone(),
    );

    let assets_service = AssetsService::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
        state.location_info.clone(),
        minecraft_cache.clone(),
    );
    let libraries_service = LibrariesService::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
        state.location_info.clone(),
    );
    let minecraft_download_service = MinecraftDownloadService::new(
        client_service,
        assets_service,
        libraries_service,
        lazy_locator.get_request_client().await,
        lazy_locator.get_progress_service().await,
        minecraft_cache.clone(),
    );

    let get_java_use_case = Arc::new(GetJavaUseCase::new(
        lazy_locator.get_java_storage().await,
        FsJavaInstallationService,
    ));

    let jre_provider = Arc::new(AzulJreProvider::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
    ));

    let install_java_use_case = Arc::new(InstallJavaUseCase::new(
        lazy_locator.get_java_storage().await,
        FsJavaInstallationService,
        jre_provider,
        state.location_info.clone(),
    ));

    let install_minecraft_use_case = Arc::new(InstallMinecraftUseCase::new(
        lazy_locator.get_progress_service().await,
        loader_version_resolver.clone(),
        get_loader_manifest_use_case.clone(),
        state.location_info.clone(),
        minecraft_download_service,
        FsJavaInstallationService,
        get_java_use_case.clone(),
        install_java_use_case.clone(),
    ));

    let install_instance_use_case = Arc::new(InstallInstanceUseCase::new(
        lazy_locator.get_instance_storage().await,
        install_minecraft_use_case,
        lazy_locator.get_progress_service().await,
        state.location_info.clone(),
    ));

    Ok(CreateInstanceUseCase::new(
        lazy_locator.get_instance_storage().await,
        loader_version_resolver,
        install_instance_use_case,
        state.location_info.clone(),
        lazy_locator.get_event_emitter().await,
        lazy_locator.get_instance_watcher_service().await?,
    )
    .execute(new_instance)
    .await?)
}

#[tracing::instrument]
pub async fn install(instance_id: String, force: bool) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    let loader_version_resolver = Arc::new(LoaderVersionResolver::new(
        lazy_locator.get_metadata_storage().await,
    ));

    let get_loader_manifest_use_case = Arc::new(GetVersionManifestUseCase::new(
        lazy_locator.get_metadata_storage().await,
    ));

    let minecraft_cache = Arc::new(FileCache::new(MinecraftDownloadResolver::new(
        state.location_info.clone(),
    )));

    let client_service = ClientService::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
        minecraft_cache.clone(),
    );

    let assets_service = AssetsService::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
        state.location_info.clone(),
        minecraft_cache.clone(),
    );
    let libraries_service = LibrariesService::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
        state.location_info.clone(),
    );
    let minecraft_download_service = MinecraftDownloadService::new(
        client_service,
        assets_service,
        libraries_service,
        lazy_locator.get_request_client().await,
        lazy_locator.get_progress_service().await,
        minecraft_cache.clone(),
    );

    let get_java_use_case = Arc::new(GetJavaUseCase::new(
        lazy_locator.get_java_storage().await,
        FsJavaInstallationService,
    ));

    let jre_provider = Arc::new(AzulJreProvider::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
    ));

    let install_java_use_case = Arc::new(InstallJavaUseCase::new(
        lazy_locator.get_java_storage().await,
        FsJavaInstallationService,
        jre_provider,
        state.location_info.clone(),
    ));

    let install_minecraft_use_case = Arc::new(InstallMinecraftUseCase::new(
        lazy_locator.get_progress_service().await,
        loader_version_resolver.clone(),
        get_loader_manifest_use_case.clone(),
        state.location_info.clone(),
        minecraft_download_service,
        FsJavaInstallationService,
        get_java_use_case.clone(),
        install_java_use_case.clone(),
    ));

    Ok(InstallInstanceUseCase::new(
        lazy_locator.get_instance_storage().await,
        install_minecraft_use_case,
        lazy_locator.get_progress_service().await,
        state.location_info.clone(),
    )
    .execute(instance_id, force)
    .await?)
}

#[tracing::instrument]
pub async fn update(instance_id: String) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(UpdateInstanceUseCase::new(
        lazy_locator.get_instance_storage().await,
        lazy_locator.get_updaters_registry().await,
    )
    .execute(instance_id)
    .await?)
}

pub async fn list() -> crate::Result<Vec<Instance>> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        ListInstancesUseCase::new(lazy_locator.get_instance_storage().await)
            .execute()
            .await?,
    )
}

pub async fn get(instance_id: String) -> crate::Result<Instance> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        GetInstanceUseCase::new(lazy_locator.get_instance_storage().await)
            .execute(instance_id)
            .await?,
    )
}

#[tracing::instrument]
pub async fn edit(instance_id: String, edit_instance: EditInstance) -> crate::Result<Instance> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        EditInstanceUseCase::new(lazy_locator.get_instance_storage().await)
            .execute(instance_id, edit_instance)
            .await?,
    )
}

#[tracing::instrument]
pub async fn remove(instance_id: String) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(RemoveInstanceUseCase::new(
        lazy_locator.get_instance_storage().await,
        lazy_locator.get_instance_watcher_service().await?,
    )
    .execute(instance_id)
    .await?)
}
