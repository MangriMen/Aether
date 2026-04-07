use std::sync::Arc;

use crate::{
    core::{
        domain::{LazyLocator, ProgressServiceType},
        LauncherState,
    },
    features::{
        auth::Credentials,
        events::infra::TauriEventEmitter,
        instance::{
            app::{
                InstallInstanceUseCase, LaunchInstanceUseCase,
                LaunchInstanceWithActiveAccountUseCase,
            },
            infra::{EventEmittingInstanceStorage, FsInstanceStorage},
        },
        java::{
            app::{GetJavaUseCase, InstallJavaUseCase},
            infra::{AzulJreProvider, FsJavaInstallationService, FsJavaStorage},
        },
        minecraft::{
            app::{
                GetMinecraftLaunchCommandUseCase, GetVersionManifestUseCase,
                InstallMinecraftUseCase,
            },
            infra::{
                AssetsService, CachedMetadataStorage, ClientService, LibrariesService,
                MinecraftDownloadResolver, MinecraftDownloadService, MinecraftMetadataResolver,
                ModrinthMetadataStorage,
            },
            LoaderVersionResolver,
        },
        process::{
            app::{
                GetProcessMetadataByInstanceIdUseCase, ManageProcessUseCase, StartProcessUseCase,
                TrackProcessUseCase,
            },
            infra::InMemoryProcessStorage,
            MinecraftProcessMetadata,
        },
        settings::infra::FsDefaultInstanceSettingsStorage,
    },
    libs::request_client::ReqwestClient,
    shared::FileCache,
};

async fn get_launch_instance_use_case(
    state: &LauncherState,
    lazy_locator: &LazyLocator,
) -> LaunchInstanceUseCase<
    EventEmittingInstanceStorage<TauriEventEmitter, FsInstanceStorage>,
    CachedMetadataStorage<
        FileCache<MinecraftMetadataResolver>,
        ModrinthMetadataStorage<ReqwestClient<ProgressServiceType>>,
    >,
    InMemoryProcessStorage,
    FsDefaultInstanceSettingsStorage,
    TauriEventEmitter,
    MinecraftDownloadService<
        ReqwestClient<ProgressServiceType>,
        ProgressServiceType,
        FileCache<MinecraftDownloadResolver>,
        FileCache<MinecraftDownloadResolver>,
    >,
    ProgressServiceType,
    FsJavaInstallationService,
    FsJavaStorage,
    AzulJreProvider<ProgressServiceType, ReqwestClient<ProgressServiceType>>,
> {
    let loader_version_resolver = Arc::new(LoaderVersionResolver::new(
        lazy_locator.get_metadata_storage().await,
    ));

    let get_version_manifest_use_case = Arc::new(GetVersionManifestUseCase::new(
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

    let get_loader_manifest_use_case = Arc::new(GetVersionManifestUseCase::new(
        lazy_locator.get_metadata_storage().await,
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

    let get_process_by_instance_id_use_case = Arc::new(GetProcessMetadataByInstanceIdUseCase::new(
        lazy_locator.get_process_storage().await,
    ));

    let track_process_use_case = Arc::new(TrackProcessUseCase::new(
        lazy_locator.get_process_storage().await,
        lazy_locator.get_instance_storage().await,
    ));

    let manage_process_use_case = Arc::new(ManageProcessUseCase::new(
        lazy_locator.get_event_emitter().await,
        lazy_locator.get_process_storage().await,
        track_process_use_case,
        state.location_info.clone(),
    ));

    let start_process_use_case = Arc::new(StartProcessUseCase::new(
        lazy_locator.get_event_emitter().await,
        lazy_locator.get_process_storage().await,
        manage_process_use_case,
    ));

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

    let get_minecraft_launch_command_use_case = GetMinecraftLaunchCommandUseCase::new(
        loader_version_resolver,
        get_version_manifest_use_case,
        minecraft_download_service,
        FsJavaInstallationService,
        get_java_use_case.clone(),
        state.location_info.clone(),
    );

    LaunchInstanceUseCase::new(
        lazy_locator.get_instance_storage().await,
        lazy_locator.get_default_instance_settings_storage().await,
        state.location_info.clone(),
        get_process_by_instance_id_use_case,
        install_instance_use_case,
        get_minecraft_launch_command_use_case,
        start_process_use_case,
    )
}

#[tracing::instrument]
pub async fn run(instance_id: String) -> crate::Result<MinecraftProcessMetadata> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    let launch_instance_use_case = get_launch_instance_use_case(&state, &lazy_locator).await;

    Ok(LaunchInstanceWithActiveAccountUseCase::new(
        lazy_locator.get_credentials_storage().await,
        launch_instance_use_case,
    )
    .execute(instance_id)
    .await?)
}

#[tracing::instrument]
pub async fn run_credentials(
    instance_id: String,
    credentials: Credentials,
) -> crate::Result<MinecraftProcessMetadata> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    Ok(get_launch_instance_use_case(&state, &lazy_locator)
        .await
        .execute(instance_id, credentials)
        .await?)
}
