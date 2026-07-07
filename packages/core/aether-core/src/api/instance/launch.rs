use std::sync::Arc;

use crate::{
    core::LazyLocator,
    features::{
        auth::Credential,
        instance::{
            InstallInstanceUseCase, InstanceInstallService, InstanceLaunchService,
            LaunchInstanceUseCase, LaunchInstanceWithActiveAccountUseCase,
        },
        java::{GetJavaUseCase, InstallJavaUseCase, infra::AzulJreProvider},
        minecraft::{
            GetMinecraftLaunchCommandUseCase, GetVersionManifestUseCase, InstallMinecraftUseCase,
            LoaderVersionResolver, MinecraftFileHealthService, MinecraftHealthService,
            MinecraftInstallService, MinecraftLaunchCommandService,
            infra::{
                AssetsService, ClientService, ForgeProcessor, LibrariesService,
                MinecraftDownloadResolver, MinecraftDownloadService,
            },
        },
        process::{
            ManageProcessUseCase, MinecraftProcessMetadata, ProcessStartService,
            StartProcessUseCase, TrackProcessUseCase,
        },
    },
    shared::cache::infra::FileCache,
};

#[allow(clippy::too_many_lines)]
async fn get_launch_instance_use_case(locator: &LazyLocator) -> LaunchInstanceUseCase {
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
        locator.get_java_installation_service().await,
    ));

    let jre_provider = Arc::new(AzulJreProvider::new(
        locator.get_progress_service().await,
        locator.get_request_client().await,
    ));

    let install_java_use_case = Arc::new(InstallJavaUseCase::new(
        locator.get_java_storage().await,
        locator.get_java_installation_service().await,
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

    let install_minecraft_service: Arc<dyn MinecraftInstallService> =
        Arc::new(InstallMinecraftUseCase::new(
            loader_version_resolver.clone(),
            get_loader_manifest_use_case.clone(),
            Arc::new(minecraft_download_service),
            forge_processor,
            locator.get_java_installation_service().await,
            get_java_use_case.clone(),
            install_java_use_case.clone(),
        ));

    let instance_install_service: Arc<dyn InstanceInstallService> =
        Arc::new(InstallInstanceUseCase::new(
            locator.get_instance_storage().await,
            install_minecraft_service,
            locator.get_progress_service().await,
            locator.location_info.clone(),
        ));

    let track_process_service = Arc::new(TrackProcessUseCase::new(
        locator.get_process_storage().await,
        locator.get_instance_storage().await,
    ));

    let manage_process_service = Arc::new(ManageProcessUseCase::new(
        locator.get_event_emitter().await,
        locator.get_process_storage().await,
        track_process_service,
        locator.location_info.clone(),
    ));

    let process_start_service: Arc<dyn ProcessStartService> = Arc::new(StartProcessUseCase::new(
        locator.get_event_emitter().await,
        locator.get_process_storage().await,
        manage_process_service,
    ));

    let minecraft_download_service2 = MinecraftDownloadService::new(
        ClientService::new(
            locator.get_progress_service().await,
            locator.get_request_client().await,
            minecraft_cache.clone(),
        ),
        AssetsService::new(
            locator.get_progress_service().await,
            locator.get_request_client().await,
            locator.location_info.clone(),
            minecraft_cache.clone(),
        ),
        LibrariesService::new(
            locator.get_progress_service().await,
            locator.get_request_client().await,
            locator.location_info.clone(),
        ),
        locator.get_request_client().await,
        locator.get_progress_service().await,
        minecraft_cache.clone(),
    );

    let minecraft_health_service: Arc<dyn MinecraftHealthService> =
        Arc::new(MinecraftFileHealthService::new(
            loader_version_resolver.clone(),
            get_version_manifest_use_case.clone(),
            Arc::new(minecraft_download_service2),
            get_java_use_case.clone(),
            locator.location_info.clone(),
        ));

    let minecraft_download_service3 = MinecraftDownloadService::new(
        ClientService::new(
            locator.get_progress_service().await,
            locator.get_request_client().await,
            minecraft_cache.clone(),
        ),
        AssetsService::new(
            locator.get_progress_service().await,
            locator.get_request_client().await,
            locator.location_info.clone(),
            minecraft_cache.clone(),
        ),
        LibrariesService::new(
            locator.get_progress_service().await,
            locator.get_request_client().await,
            locator.location_info.clone(),
        ),
        locator.get_request_client().await,
        locator.get_progress_service().await,
        minecraft_cache,
    );

    let minecraft_launch_command_service: Arc<dyn MinecraftLaunchCommandService> =
        Arc::new(GetMinecraftLaunchCommandUseCase::new(
            loader_version_resolver,
            get_version_manifest_use_case,
            Arc::new(minecraft_download_service3),
            locator.get_java_installation_service().await,
            get_java_use_case.clone(),
            install_java_use_case.clone(),
            locator.location_info.clone(),
        ));

    LaunchInstanceUseCase::new(
        locator.get_plugin_registry().await,
        locator.get_instance_storage().await,
        locator.get_default_instance_settings_storage().await,
        locator.location_info.clone(),
        locator.get_process_storage().await,
        instance_install_service,
        minecraft_health_service,
        minecraft_launch_command_service,
        process_start_service,
    )
}

#[tracing::instrument]
pub async fn run(instance_id: String) -> crate::Result<MinecraftProcessMetadata> {
    let locator = LazyLocator::get().await?;

    let launch_instance_service: Arc<dyn InstanceLaunchService> =
        Arc::new(get_launch_instance_use_case(&locator).await);

    Ok(LaunchInstanceWithActiveAccountUseCase::new(
        locator.get_credentials_storage().await,
        launch_instance_service,
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
