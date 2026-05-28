use std::sync::Arc;

use crate::{
    core::{LazyLocator, domain::ProgressServiceType},
    features::{
        auth::domain::Credential,
        instance::{
            InstallInstanceUseCase, LaunchInstanceUseCase, LaunchInstanceWithActiveAccountUseCase,
            infra::{EventEmittingInstanceStorage, SqliteInstanceStorage},
        },
        java::{
            GetJavaUseCase, InstallJavaUseCase,
            infra::{
                AzulJreProvider, FsJavaInstallationService, MemoryJavaInstallationTracker,
                SqliteJavaStorage,
            },
        },
        minecraft::{
            GetMinecraftLaunchCommandUseCase, GetVersionManifestUseCase, InstallMinecraftUseCase,
            LoaderVersionResolver,
            infra::{
                AssetsService, CachedMetadataStorage, ClientService, ForgeProcessor,
                LibrariesService, MinecraftDownloadResolver, MinecraftDownloadService,
                ModrinthMetadataStorage,
            },
        },
        process::{
            GetProcessMetadataByInstanceIdUseCase, ManageProcessUseCase, MinecraftProcessMetadata,
            StartProcessUseCase, TrackProcessUseCase, infra::InMemoryProcessStorage,
        },
        settings::infra::SqliteDefaultInstanceSettingsStorage,
    },
    libs::request_client::ReqwestClient,
    shared::{FileCache, SqliteCache},
};

#[allow(clippy::too_many_lines)]
async fn get_launch_instance_use_case(
    locator: &LazyLocator,
) -> LaunchInstanceUseCase<
    EventEmittingInstanceStorage<SqliteInstanceStorage>,
    CachedMetadataStorage<SqliteCache, ModrinthMetadataStorage<ReqwestClient<ProgressServiceType>>>,
    InMemoryProcessStorage,
    SqliteDefaultInstanceSettingsStorage,
    MinecraftDownloadService<
        ReqwestClient<ProgressServiceType>,
        ProgressServiceType,
        FileCache<MinecraftDownloadResolver>,
        FileCache<MinecraftDownloadResolver>,
    >,
    ForgeProcessor<ProgressServiceType>,
    ProgressServiceType,
    FsJavaInstallationService,
    SqliteJavaStorage,
    AzulJreProvider<ProgressServiceType, ReqwestClient<ProgressServiceType>>,
    MemoryJavaInstallationTracker,
> {
    let loader_version_resolver = Arc::new(LoaderVersionResolver::new(
        locator.get_metadata_storage().await,
    ));

    let get_version_manifest_use_case = Arc::new(GetVersionManifestUseCase::new(
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

    let get_loader_manifest_use_case = Arc::new(GetVersionManifestUseCase::new(
        locator.get_metadata_storage().await,
    ));

    let forge_processor = Arc::new(ForgeProcessor::new(
        locator.get_progress_service().await,
        locator.location_info.clone(),
    ));

    let install_minecraft_use_case = Arc::new(InstallMinecraftUseCase::new(
        loader_version_resolver.clone(),
        get_loader_manifest_use_case.clone(),
        minecraft_download_service,
        forge_processor,
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

    let get_process_by_instance_id_use_case = Arc::new(GetProcessMetadataByInstanceIdUseCase::new(
        locator.get_process_storage().await,
    ));

    let track_process_use_case = Arc::new(TrackProcessUseCase::new(
        locator.get_process_storage().await,
        locator.get_instance_storage().await,
    ));

    let manage_process_use_case = Arc::new(ManageProcessUseCase::new(
        locator.get_event_emitter().await,
        locator.get_process_storage().await,
        track_process_use_case,
        locator.location_info.clone(),
    ));

    let start_process_use_case = Arc::new(StartProcessUseCase::new(
        locator.get_event_emitter().await,
        locator.get_process_storage().await,
        manage_process_use_case,
    ));

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

    let get_minecraft_launch_command_use_case = GetMinecraftLaunchCommandUseCase::new(
        loader_version_resolver,
        get_version_manifest_use_case,
        minecraft_download_service,
        FsJavaInstallationService,
        get_java_use_case.clone(),
        locator.location_info.clone(),
    );

    LaunchInstanceUseCase::new(
        locator.get_instance_storage().await,
        locator.get_default_instance_settings_storage().await,
        locator.location_info.clone(),
        get_process_by_instance_id_use_case,
        install_instance_use_case,
        get_minecraft_launch_command_use_case,
        start_process_use_case,
    )
}

#[tracing::instrument]
pub async fn run(instance_id: String) -> crate::Result<MinecraftProcessMetadata> {
    let locator = LazyLocator::get().await?;

    let launch_instance_use_case = get_launch_instance_use_case(&locator).await;

    Ok(LaunchInstanceWithActiveAccountUseCase::new(
        locator.get_credentials_storage().await,
        launch_instance_use_case,
    )
    .execute(instance_id)
    .await?)
}

#[tracing::instrument]
pub async fn run_credentials(
    instance_id: String,
    credentials: Credential,
) -> crate::Result<MinecraftProcessMetadata> {
    let locator = LazyLocator::get().await?;

    Ok(get_launch_instance_use_case(&locator)
        .await
        .execute(instance_id, credentials)
        .await?)
}
