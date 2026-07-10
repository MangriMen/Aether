use std::sync::{Arc, OnceLock};

use crate::features::{
    auth::{
        ActiveAccountHelper, AuthFeature, CreateOfflineAccountUseCase,
        CreateOfflineAccountUseCasePort, GetAccountsUseCase, GetAccountsUseCasePort, LogoutUseCase,
        LogoutUseCasePort, SetActiveAccountUseCase, SetActiveAccountUseCasePort,
    },
    events::{
        Event, EventEmitter, EventsFeature, ListProgressBarsUseCase, ListProgressBarsUseCasePort,
        ProgressBarStorage, ProgressService, SharedEventEmitter,
    },
    instance::{
        ChangeContentStateUseCase, ChangeContentStateUseCasePort, CheckContentCompatibilityUseCase,
        CheckContentCompatibilityUseCasePort, ContentFileService, ContentProvider,
        CreateInstanceUseCase, CreateInstanceUseCasePort, EditInstanceIconUseCase,
        EditInstanceIconUseCasePort, EditInstanceUseCase, EditInstanceUseCasePort,
        GetContentUseCase, GetContentUseCasePort, GetInstanceUseCase, GetInstanceUseCasePort,
        ImportContentUseCase, ImportContentUseCasePort, ImportInstanceUseCase,
        ImportInstanceUseCasePort, Importer, InstallContentUseCase, InstallContentUseCasePort,
        InstallInstanceUseCase, InstanceFileService, InstanceInstallService, InstanceLaunchService,
        InstanceStorage, InstanceWatcherService, LaunchInstanceUseCase,
        LaunchInstanceWithActiveAccountUseCase, LaunchInstanceWithActiveAccountUseCasePort,
        ListContentUseCase, ListContentUseCasePort, ListContentVersionsUseCase,
        ListContentVersionsUseCasePort, ListImportersUseCase, ListImportersUseCasePort,
        ListInstancesUseCase, ListInstancesUseCasePort, ListProvidersUseCase,
        ListProvidersUseCasePort, PackStorage, RemoveContentUseCase, RemoveContentUseCasePort,
        RemoveInstanceUseCase, RemoveInstanceUseCasePort, SearchContentUseCase,
        SearchContentUseCasePort, UpdateInstanceUseCase, UpdateInstanceUseCasePort, Updater,
    },
    java::{
        DiscoverJavaUseCase, DiscoverJavaUseCasePort, EditJavaUseCase, EditJavaUseCasePort,
        GetActiveJavaInstallationsUseCase, GetActiveJavaInstallationsUseCasePort, GetJavaUseCase,
        InstallJavaUseCase, JavaFeature, JavaInstallService, JavaQueryService, ListJavaUseCase,
        ListJavaUseCasePort, RemoveJavaUseCase, RemoveJavaUseCasePort, TestJreUseCase,
        TestJreUseCasePort, infra::get_default_discovery_paths,
    },
    minecraft::{
        GetLoaderVersionManifestUseCase, GetLoaderVersionManifestUseCasePort,
        GetMinecraftLaunchCommandUseCase, GetVersionManifestUseCase, InstallMinecraftUseCase,
        LoaderVersionResolver, LoaderVersionService, MinecraftFeature, MinecraftFileHealthService,
        MinecraftHealthService, MinecraftInstallService, MinecraftLaunchCommandService,
        VersionManifestService,
    },
    plugins::{
        CheckForPluginUpdatesUseCase, CheckForPluginUpdatesUseCasePort, DisablePluginUseCase,
        EditPluginSettingsUseCase, EditPluginSettingsUseCasePort, EnablePluginUseCase,
        EnablePluginUseCasePort, ForceEnablePluginUseCase, ForceEnablePluginUseCasePort,
        GetPluginApiVersionUseCase, GetPluginApiVersionUseCasePort, GetPluginDtoUseCase,
        GetPluginDtoUseCasePort, GetPluginSettingsUseCase, GetPluginSettingsUseCasePort,
        ImportPluginsUseCase, ImportPluginsUseCasePort, ListPluginsDtoUseCase,
        ListPluginsDtoUseCasePort, LoadConfigType, PluginDisableService, PluginExtractor,
        PluginLoader, PluginLoaderRegistry, PluginProviderFactory, PluginRegistry,
        PluginSettingsStorage, PluginSourceStorage, PluginStorage, PluginSyncService,
        PluginsFeature, RemovePluginUseCase, RemovePluginUseCasePort, SyncPluginsUseCase,
        UpdatePluginUseCase, UpdatePluginUseCasePort,
    },
    process::{
        GetProcessMetadataByInstanceIdUseCase, GetProcessMetadataByInstanceIdUseCasePort,
        KillProcessUseCase, KillProcessUseCasePort, ListProcessMetadataUseCase,
        ListProcessMetadataUseCasePort, ManageProcessService, ManageProcessUseCase, ProcessFeature,
        ProcessStartService, ProcessStorage, StartProcessUseCase, TrackProcessService,
        TrackProcessUseCase, WaitForProcessUseCase, WaitForProcessUseCasePort,
    },
    settings::{
        EditDefaultInstanceSettingsUseCase, EditDefaultInstanceSettingsUseCasePort,
        EditSettingsUseCase, EditSettingsUseCasePort, GetDefaultInstanceSettingsUseCase,
        GetDefaultInstanceSettingsUseCasePort, GetSettingsUseCase, GetSettingsUseCasePort,
        LocationInfo, SettingsFeature,
    },
};
use crate::{
    features::instance::{
        ContentManagementPort, ContentProviderPort, InstanceCrudPort, InstanceLifecyclePort,
        InstanceServicesPort,
    },
    shared::{cache::domain::AssetsStorage, capability::domain::CapabilityRegistry},
};

struct UseCaseCache {
    list_instances_uc: OnceLock<Arc<dyn crate::features::instance::ListInstancesUseCasePort>>,
    get_instance_uc: OnceLock<Arc<dyn crate::features::instance::GetInstanceUseCasePort>>,
    edit_instance_uc: OnceLock<Arc<dyn crate::features::instance::EditInstanceUseCasePort>>,
    edit_instance_icon_uc:
        OnceLock<Arc<dyn crate::features::instance::EditInstanceIconUseCasePort>>,
    remove_instance_uc: OnceLock<Arc<dyn crate::features::instance::RemoveInstanceUseCasePort>>,
    update_instance_uc: OnceLock<Arc<dyn crate::features::instance::UpdateInstanceUseCasePort>>,
    start_process_uc: OnceLock<Arc<dyn ProcessStartService>>,
    track_process_uc: OnceLock<Arc<dyn TrackProcessService>>,
    manage_process_uc: OnceLock<Arc<dyn ManageProcessService>>,
    install_instance_uc: OnceLock<Arc<dyn InstanceInstallService>>,
    disable_plugin_uc: OnceLock<Arc<dyn PluginDisableService>>,
    sync_plugins_uc: OnceLock<Arc<dyn PluginSyncService>>,
    health_service: OnceLock<Arc<dyn MinecraftHealthService>>,
    launch_command_service: OnceLock<Arc<dyn MinecraftLaunchCommandService>>,
    launch_instance_uc: OnceLock<Arc<dyn InstanceLaunchService>>,
    minecraft_install_uc: OnceLock<Arc<dyn MinecraftInstallService>>,
}

impl UseCaseCache {
    fn new() -> Self {
        Self {
            list_instances_uc: OnceLock::new(),
            get_instance_uc: OnceLock::new(),
            edit_instance_uc: OnceLock::new(),
            edit_instance_icon_uc: OnceLock::new(),
            remove_instance_uc: OnceLock::new(),
            update_instance_uc: OnceLock::new(),
            start_process_uc: OnceLock::new(),
            track_process_uc: OnceLock::new(),
            manage_process_uc: OnceLock::new(),
            install_instance_uc: OnceLock::new(),
            disable_plugin_uc: OnceLock::new(),
            sync_plugins_uc: OnceLock::new(),
            health_service: OnceLock::new(),
            launch_command_service: OnceLock::new(),
            launch_instance_uc: OnceLock::new(),
            minecraft_install_uc: OnceLock::new(),
        }
    }
}

pub struct StorageParams {
    pub credentials_storage: Arc<dyn crate::features::auth::CredentialsStorage>,
    pub settings_storage: Arc<dyn crate::features::settings::SettingsStorage>,
    pub default_instance_settings_storage:
        Arc<dyn crate::features::settings::DefaultInstanceSettingsStorage>,
    pub process_storage: Arc<dyn ProcessStorage>,
    pub instance_storage: Arc<dyn InstanceStorage>,
    pub pack_storage: Arc<dyn PackStorage>,
    pub content_file_service: Arc<dyn ContentFileService>,
    pub instance_file_service: Arc<dyn InstanceFileService>,
    pub java_storage: Arc<dyn crate::features::java::JavaStorage>,
    pub java_installation_service: Arc<dyn crate::features::java::JavaInstallationService>,
    pub java_installation_tracker: Arc<dyn crate::features::java::JavaInstallationTracker>,
    pub jre_provider: Arc<dyn crate::features::java::JreProvider>,
    pub metadata_storage: Arc<dyn crate::features::minecraft::MetadataStorage>,
    pub assets_storage: Arc<dyn AssetsStorage>,
}

pub struct MinecraftParams {
    pub minecraft_downloader: Arc<dyn crate::features::minecraft::MinecraftDownloader>,
    pub mod_loader_processor: Arc<dyn crate::features::minecraft::ModLoaderProcessor>,
}

pub struct PluginParams {
    pub registry: Arc<PluginRegistry>,
    pub loader_registry: Arc<PluginLoaderRegistry>,
    pub storage: Arc<dyn PluginStorage>,
    pub source_storage: Arc<dyn PluginSourceStorage>,
    pub settings_storage: Arc<dyn PluginSettingsStorage>,
    pub provider_factory: Arc<PluginProviderFactory>,
    pub extractor: Arc<dyn PluginExtractor>,
}

#[allow(clippy::too_many_lines)]
pub struct AetherContainerParams {
    pub storage: StorageParams,
    pub minecraft: MinecraftParams,
    pub plugins: PluginParams,
    pub event_emitter: SharedEventEmitter,
    pub progress_bar_storage: Arc<dyn ProgressBarStorage>,
    pub progress_service: Arc<dyn ProgressService>,
    pub location_info: Arc<LocationInfo>,
    pub instance_watcher_service: Arc<dyn InstanceWatcherService>,
    pub importers_registry: Arc<dyn CapabilityRegistry<Arc<dyn Importer>>>,
    pub updaters_registry: Arc<dyn CapabilityRegistry<Arc<dyn Updater>>>,
    pub content_provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
}

pub struct AetherContainer {
    cache: UseCaseCache,
    params: AetherContainerParams,
}

impl AetherContainer {
    pub fn new(params: AetherContainerParams) -> Arc<Self> {
        Arc::new(Self {
            cache: UseCaseCache::new(),
            params,
        })
    }

    pub fn storage(&self) -> &StorageParams {
        &self.params.storage
    }
    pub fn minecraft(&self) -> &MinecraftParams {
        &self.params.minecraft
    }
    pub fn plugins(&self) -> &PluginParams {
        &self.params.plugins
    }
    pub fn event_emitter(&self) -> SharedEventEmitter {
        self.params.event_emitter.clone()
    }
    pub fn location_info(&self) -> Arc<LocationInfo> {
        self.params.location_info.clone()
    }
    pub fn progress_bar_storage(&self) -> Arc<dyn ProgressBarStorage> {
        self.params.progress_bar_storage.clone()
    }
    pub fn progress_service(&self) -> Arc<dyn ProgressService> {
        self.params.progress_service.clone()
    }
    pub fn instance_watcher_service(&self) -> Arc<dyn InstanceWatcherService> {
        self.params.instance_watcher_service.clone()
    }
    pub fn importers_registry(&self) -> Arc<dyn CapabilityRegistry<Arc<dyn Importer>>> {
        self.params.importers_registry.clone()
    }
    pub fn updaters_registry(&self) -> Arc<dyn CapabilityRegistry<Arc<dyn Updater>>> {
        self.params.updaters_registry.clone()
    }
    pub fn content_provider_registry(
        &self,
    ) -> Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>> {
        self.params.content_provider_registry.clone()
    }

    fn loader_version_resolver(&self) -> Arc<dyn LoaderVersionService> {
        Arc::new(LoaderVersionResolver::new(
            self.storage().metadata_storage.clone(),
        ))
    }

    fn get_version_manifest_use_case(&self) -> Arc<dyn VersionManifestService> {
        Arc::new(GetVersionManifestUseCase::new(
            self.storage().metadata_storage.clone(),
        ))
    }

    fn get_java_use_case(&self) -> Arc<dyn JavaQueryService> {
        Arc::new(GetJavaUseCase::new(
            self.storage().java_storage.clone(),
            self.storage().java_installation_service.clone(),
        ))
    }

    fn install_java_use_case(&self) -> Arc<dyn JavaInstallService> {
        Arc::new(InstallJavaUseCase::new(
            self.storage().java_storage.clone(),
            self.storage().java_installation_service.clone(),
            self.storage().jre_provider.clone(),
            self.location_info().clone(),
            self.storage().java_installation_tracker.clone(),
        ))
    }

    fn minecraft_install_use_case(&self) -> Arc<dyn MinecraftInstallService> {
        self.cache
            .minecraft_install_uc
            .get_or_init(|| {
                Arc::new(InstallMinecraftUseCase::new(
                    self.loader_version_resolver(),
                    self.get_version_manifest_use_case(),
                    self.minecraft().minecraft_downloader.clone(),
                    self.minecraft().mod_loader_processor.clone(),
                    self.storage().java_installation_service.clone(),
                    self.get_java_use_case(),
                    self.install_java_use_case(),
                ))
            })
            .clone()
    }

    fn instance_install_service(&self) -> Arc<dyn InstanceInstallService> {
        self.cache
            .install_instance_uc
            .get_or_init(|| {
                Arc::new(InstallInstanceUseCase::new(
                    self.storage().instance_storage.clone(),
                    self.minecraft_install_use_case(),
                    self.progress_service().clone(),
                    self.location_info().clone(),
                ))
            })
            .clone()
    }

    fn health_service(&self) -> Arc<dyn MinecraftHealthService> {
        self.cache
            .health_service
            .get_or_init(|| {
                Arc::new(MinecraftFileHealthService::new(
                    self.loader_version_resolver(),
                    self.get_version_manifest_use_case(),
                    self.minecraft().minecraft_downloader.clone(),
                    self.get_java_use_case(),
                    self.location_info().clone(),
                ))
            })
            .clone()
    }

    fn launch_command_service(&self) -> Arc<dyn MinecraftLaunchCommandService> {
        self.cache
            .launch_command_service
            .get_or_init(|| {
                Arc::new(GetMinecraftLaunchCommandUseCase::new(
                    self.loader_version_resolver(),
                    self.get_version_manifest_use_case(),
                    self.minecraft().minecraft_downloader.clone(),
                    self.storage().java_installation_service.clone(),
                    self.get_java_use_case(),
                    self.install_java_use_case(),
                    self.location_info().clone(),
                ))
            })
            .clone()
    }

    fn launch_instance_use_case(&self) -> Arc<dyn InstanceLaunchService> {
        self.cache
            .launch_instance_uc
            .get_or_init(|| {
                let start = self.process_start_service();
                Arc::new(LaunchInstanceUseCase::new(
                    self.plugins().registry.clone(),
                    self.storage().instance_storage.clone(),
                    self.storage().default_instance_settings_storage.clone(),
                    self.location_info().clone(),
                    self.storage().process_storage.clone(),
                    self.instance_install_service(),
                    self.health_service(),
                    self.launch_command_service(),
                    start,
                ))
            })
            .clone()
    }

    fn track_process_service(&self) -> Arc<dyn TrackProcessService> {
        self.cache
            .track_process_uc
            .get_or_init(|| {
                Arc::new(TrackProcessUseCase::new(
                    self.storage().process_storage.clone(),
                    self.storage().instance_storage.clone(),
                ))
            })
            .clone()
    }

    fn manage_process_service(&self) -> Arc<dyn ManageProcessService> {
        self.cache
            .manage_process_uc
            .get_or_init(|| {
                Arc::new(ManageProcessUseCase::new(
                    self.event_emitter().clone(),
                    self.storage().process_storage.clone(),
                    self.track_process_service(),
                    self.location_info().clone(),
                ))
            })
            .clone()
    }

    fn process_start_service(&self) -> Arc<dyn ProcessStartService> {
        self.cache
            .start_process_uc
            .get_or_init(|| {
                Arc::new(StartProcessUseCase::new(
                    self.event_emitter().clone(),
                    self.storage().process_storage.clone(),
                    self.manage_process_service(),
                ))
            })
            .clone()
    }

    fn disable_plugin_service(&self) -> Arc<dyn PluginDisableService> {
        self.cache
            .disable_plugin_uc
            .get_or_init(|| {
                Arc::new(DisablePluginUseCase::new(
                    self.plugins().registry.clone(),
                    self.plugins().loader_registry.clone(),
                    self.storage().settings_storage.clone(),
                ))
            })
            .clone()
    }

    fn sync_plugins_service(&self) -> Arc<dyn PluginSyncService> {
        self.cache
            .sync_plugins_uc
            .get_or_init(|| {
                Arc::new(SyncPluginsUseCase::new(
                    self.plugins().storage.clone(),
                    self.plugins().registry.clone(),
                    self.disable_plugin_service(),
                    self.event_emitter().clone(),
                ))
            })
            .clone()
    }
}

// Feature trait implementations

impl SettingsFeature for AetherContainer {
    fn get_settings_use_case(&self) -> Arc<dyn GetSettingsUseCasePort> {
        Arc::new(GetSettingsUseCase::new(
            self.storage().settings_storage.clone(),
        ))
    }
    fn get_default_instance_settings_use_case(
        &self,
    ) -> Arc<dyn GetDefaultInstanceSettingsUseCasePort> {
        Arc::new(GetDefaultInstanceSettingsUseCase::new(
            self.storage().default_instance_settings_storage.clone(),
        ))
    }
    fn edit_settings_use_case(&self) -> Arc<dyn EditSettingsUseCasePort> {
        Arc::new(EditSettingsUseCase::new(
            self.storage().settings_storage.clone(),
        ))
    }
    fn edit_default_instance_settings_use_case(
        &self,
    ) -> Arc<dyn EditDefaultInstanceSettingsUseCasePort> {
        Arc::new(EditDefaultInstanceSettingsUseCase::new(
            self.storage().default_instance_settings_storage.clone(),
        ))
    }
    fn location_info(&self) -> Arc<LocationInfo> {
        self.location_info().clone()
    }
}

impl AuthFeature for AetherContainer {
    fn create_offline_account_use_case(&self) -> Arc<dyn CreateOfflineAccountUseCasePort> {
        Arc::new(CreateOfflineAccountUseCase::new(
            self.storage().credentials_storage.clone(),
        ))
    }
    fn get_accounts_use_case(&self) -> Arc<dyn GetAccountsUseCasePort> {
        Arc::new(GetAccountsUseCase::new(
            self.storage().credentials_storage.clone(),
        ))
    }
    fn logout_use_case(&self) -> Arc<dyn LogoutUseCasePort> {
        Arc::new(LogoutUseCase::new(
            self.storage().credentials_storage.clone(),
        ))
    }
    fn set_active_account_use_case(&self) -> Arc<dyn SetActiveAccountUseCasePort> {
        Arc::new(SetActiveAccountUseCase::new(
            self.storage().credentials_storage.clone(),
        ))
    }
    fn active_account_helper(&self) -> Arc<ActiveAccountHelper> {
        Arc::new(ActiveAccountHelper)
    }
}

impl JavaFeature for AetherContainer {
    fn discover_java_use_case(&self) -> Arc<dyn DiscoverJavaUseCasePort> {
        Arc::new(DiscoverJavaUseCase::new(
            self.storage().java_installation_service.clone(),
            get_default_discovery_paths(),
        ))
    }
    fn edit_java_use_case(&self) -> Arc<dyn EditJavaUseCasePort> {
        Arc::new(EditJavaUseCase::new(
            self.storage().java_storage.clone(),
            self.storage().java_installation_service.clone(),
        ))
    }
    fn get_active_java_installations_use_case(
        &self,
    ) -> Arc<dyn GetActiveJavaInstallationsUseCasePort> {
        Arc::new(GetActiveJavaInstallationsUseCase::new(
            self.storage().java_installation_tracker.clone(),
        ))
    }
    fn list_java_use_case(&self) -> Arc<dyn ListJavaUseCasePort> {
        Arc::new(ListJavaUseCase::new(self.storage().java_storage.clone()))
    }
    fn remove_java_use_case(&self) -> Arc<dyn RemoveJavaUseCasePort> {
        Arc::new(RemoveJavaUseCase::new(self.storage().java_storage.clone()))
    }
    fn test_jre_use_case(&self) -> Arc<dyn TestJreUseCasePort> {
        Arc::new(TestJreUseCase::new(
            self.storage().java_installation_service.clone(),
        ))
    }
    fn get_java_use_case(&self) -> Arc<dyn JavaQueryService> {
        self.get_java_use_case()
    }
    fn install_java_use_case(&self) -> Arc<dyn JavaInstallService> {
        self.install_java_use_case()
    }
}

impl MinecraftFeature for AetherContainer {
    fn get_version_manifest_use_case(&self) -> Arc<dyn VersionManifestService> {
        self.get_version_manifest_use_case()
    }
    fn install_minecraft_use_case(&self) -> Arc<dyn MinecraftInstallService> {
        self.minecraft_install_use_case()
    }
    fn get_minecraft_launch_command_use_case(&self) -> Arc<dyn MinecraftLaunchCommandService> {
        self.launch_command_service()
    }
    fn get_loader_version_manifest_use_case(&self) -> Arc<dyn GetLoaderVersionManifestUseCasePort> {
        Arc::new(GetLoaderVersionManifestUseCase::new(
            self.storage().metadata_storage.clone(),
        ))
    }
    fn loader_version_service(&self) -> Arc<dyn LoaderVersionService> {
        self.loader_version_resolver()
    }
    fn minecraft_health_service(&self) -> Arc<dyn MinecraftHealthService> {
        self.health_service()
    }
}

impl InstanceCrudPort for AetherContainer {
    fn create_instance_use_case(&self) -> Arc<dyn CreateInstanceUseCasePort> {
        Arc::new(CreateInstanceUseCase::new(
            self.storage().instance_storage.clone(),
            self.loader_version_resolver(),
            self.instance_install_service(),
            self.location_info().clone(),
            self.event_emitter().clone(),
            self.instance_watcher_service().clone(),
        ))
    }
    fn get_instance_use_case(&self) -> Arc<dyn GetInstanceUseCasePort> {
        self.cache
            .get_instance_uc
            .get_or_init(|| {
                Arc::new(GetInstanceUseCase::new(
                    self.storage().instance_storage.clone(),
                ))
            })
            .clone()
    }
    fn list_instances_use_case(&self) -> Arc<dyn ListInstancesUseCasePort> {
        self.cache
            .list_instances_uc
            .get_or_init(|| {
                Arc::new(ListInstancesUseCase::new(
                    self.storage().instance_storage.clone(),
                ))
            })
            .clone()
    }
    fn edit_instance_use_case(&self) -> Arc<dyn EditInstanceUseCasePort> {
        self.cache
            .edit_instance_uc
            .get_or_init(|| {
                Arc::new(EditInstanceUseCase::new(
                    self.storage().instance_storage.clone(),
                ))
            })
            .clone()
    }
    fn edit_instance_icon_use_case(&self) -> Arc<dyn EditInstanceIconUseCasePort> {
        self.cache
            .edit_instance_icon_uc
            .get_or_init(|| {
                Arc::new(EditInstanceIconUseCase::new(
                    self.storage().instance_storage.clone(),
                    self.storage().assets_storage.clone(),
                ))
            })
            .clone()
    }
    fn remove_instance_use_case(&self) -> Arc<dyn RemoveInstanceUseCasePort> {
        self.cache
            .remove_instance_uc
            .get_or_init(|| {
                Arc::new(RemoveInstanceUseCase::new(
                    self.storage().instance_storage.clone(),
                    self.instance_watcher_service().clone(),
                    self.storage().instance_file_service.clone(),
                    self.storage().pack_storage.clone(),
                ))
            })
            .clone()
    }
    fn update_instance_use_case(&self) -> Arc<dyn UpdateInstanceUseCasePort> {
        self.cache
            .update_instance_uc
            .get_or_init(|| {
                Arc::new(UpdateInstanceUseCase::new(
                    self.storage().instance_storage.clone(),
                    self.updaters_registry().clone(),
                ))
            })
            .clone()
    }
}

impl InstanceLifecyclePort for AetherContainer {
    fn import_instance_use_case(&self) -> Arc<dyn ImportInstanceUseCasePort> {
        Arc::new(ImportInstanceUseCase::new(
            self.importers_registry().clone(),
        ))
    }
    fn list_importers_use_case(&self) -> Arc<dyn ListImportersUseCasePort> {
        Arc::new(ListImportersUseCase::new(self.importers_registry().clone()))
    }
    fn launch_instance_with_active_account_use_case(
        &self,
    ) -> Arc<dyn LaunchInstanceWithActiveAccountUseCasePort> {
        Arc::new(LaunchInstanceWithActiveAccountUseCase::new(
            self.storage().credentials_storage.clone(),
            self.launch_instance_use_case(),
        ))
    }
}

impl ContentManagementPort for AetherContainer {
    fn change_content_state_use_case(&self) -> Arc<dyn ChangeContentStateUseCasePort> {
        Arc::new(ChangeContentStateUseCase::new(
            self.event_emitter().clone(),
            self.storage().content_file_service.clone(),
        ))
    }
    fn import_content_use_case(&self) -> Arc<dyn ImportContentUseCasePort> {
        Arc::new(ImportContentUseCase::new(
            self.event_emitter().clone(),
            self.storage().pack_storage.clone(),
            self.location_info().clone(),
        ))
    }
    fn list_content_use_case(&self) -> Arc<dyn ListContentUseCasePort> {
        Arc::new(ListContentUseCase::new(
            self.storage().pack_storage.clone(),
            self.location_info().clone(),
        ))
    }
    fn remove_content_use_case(&self) -> Arc<dyn RemoveContentUseCasePort> {
        Arc::new(RemoveContentUseCase::new(
            self.event_emitter().clone(),
            self.storage().pack_storage.clone(),
            self.storage().content_file_service.clone(),
        ))
    }
}

impl ContentProviderPort for AetherContainer {
    fn search_content_use_case(&self) -> Arc<dyn SearchContentUseCasePort> {
        Arc::new(SearchContentUseCase::new(
            self.content_provider_registry().clone(),
        ))
    }
    fn get_content_use_case(&self) -> Arc<dyn GetContentUseCasePort> {
        Arc::new(GetContentUseCase::new(
            self.content_provider_registry().clone(),
        ))
    }
    fn install_content_use_case(&self) -> Arc<dyn InstallContentUseCasePort> {
        Arc::new(InstallContentUseCase::new(
            self.storage().pack_storage.clone(),
            self.content_provider_registry().clone(),
            self.storage().content_file_service.clone(),
        ))
    }
    fn check_content_compatibility_use_case(
        &self,
    ) -> Arc<dyn CheckContentCompatibilityUseCasePort> {
        Arc::new(CheckContentCompatibilityUseCase::new(
            self.content_provider_registry().clone(),
            self.storage().instance_storage.clone(),
        ))
    }
    fn list_content_versions_use_case(&self) -> Arc<dyn ListContentVersionsUseCasePort> {
        Arc::new(ListContentVersionsUseCase::new(
            self.content_provider_registry().clone(),
        ))
    }
    fn list_providers_use_case(&self) -> Arc<dyn ListProvidersUseCasePort> {
        Arc::new(ListProvidersUseCase::new(
            self.content_provider_registry().clone(),
        ))
    }
}

impl InstanceServicesPort for AetherContainer {
    fn instance_install_service(&self) -> Arc<dyn InstanceInstallService> {
        self.instance_install_service()
    }
    fn instance_launch_service(&self) -> Arc<dyn InstanceLaunchService> {
        self.launch_instance_use_case()
    }
    fn instance_watcher_service(&self) -> Arc<dyn InstanceWatcherService> {
        self.instance_watcher_service().clone()
    }
    fn instance_file_service(&self) -> Arc<dyn InstanceFileService> {
        self.storage().instance_file_service.clone()
    }
    fn content_file_service(&self) -> Arc<dyn ContentFileService> {
        self.storage().content_file_service.clone()
    }
}

impl ProcessFeature for AetherContainer {
    fn wait_for_process_use_case(&self) -> Arc<dyn WaitForProcessUseCasePort> {
        Arc::new(WaitForProcessUseCase::new(
            self.storage().process_storage.clone(),
        ))
    }
    fn kill_process_use_case(&self) -> Arc<dyn KillProcessUseCasePort> {
        Arc::new(KillProcessUseCase::new(
            self.storage().process_storage.clone(),
        ))
    }
    fn list_process_metadata_use_case(&self) -> Arc<dyn ListProcessMetadataUseCasePort> {
        Arc::new(ListProcessMetadataUseCase::new(
            self.storage().process_storage.clone(),
        ))
    }
    fn get_process_metadata_by_instance_id_use_case(
        &self,
    ) -> Arc<dyn GetProcessMetadataByInstanceIdUseCasePort> {
        Arc::new(GetProcessMetadataByInstanceIdUseCase::new(
            self.storage().process_storage.clone(),
        ))
    }
    fn process_start_service(&self) -> Arc<dyn ProcessStartService> {
        self.process_start_service()
    }
    fn track_process_service(&self) -> Arc<dyn TrackProcessService> {
        self.track_process_service()
    }
    fn manage_process_service(&self) -> Arc<dyn ManageProcessService> {
        self.manage_process_service()
    }
}

impl PluginsFeature for AetherContainer {
    fn check_for_plugin_updates_use_case(&self) -> Arc<dyn CheckForPluginUpdatesUseCasePort> {
        Arc::new(CheckForPluginUpdatesUseCase::new(
            self.plugins().source_storage.clone(),
            self.plugins().provider_factory.clone(),
        ))
    }
    fn edit_plugin_settings_use_case(&self) -> Arc<dyn EditPluginSettingsUseCasePort> {
        Arc::new(EditPluginSettingsUseCase::new(
            self.plugins().settings_storage.clone(),
        ))
    }
    fn enable_plugin_use_case(&self) -> Arc<dyn EnablePluginUseCasePort> {
        Arc::new(EnablePluginUseCase::new(
            self.plugins().registry.clone(),
            self.plugins().loader_registry.clone(),
            self.plugins().settings_storage.clone(),
            self.storage().settings_storage.clone(),
        ))
    }
    fn force_enable_plugin_use_case(&self) -> Arc<dyn ForceEnablePluginUseCasePort> {
        Arc::new(ForceEnablePluginUseCase::new(
            self.plugins().registry.clone(),
            self.plugins().loader_registry.clone(),
            self.plugins().settings_storage.clone(),
            self.storage().settings_storage.clone(),
        ))
    }
    fn get_plugin_api_version_use_case(&self) -> Arc<dyn GetPluginApiVersionUseCasePort> {
        Arc::new(GetPluginApiVersionUseCase::default())
    }
    fn get_plugin_dto_use_case(&self) -> Arc<dyn GetPluginDtoUseCasePort> {
        Arc::new(GetPluginDtoUseCase::new(self.plugins().registry.clone()))
    }
    fn get_plugin_settings_use_case(&self) -> Arc<dyn GetPluginSettingsUseCasePort> {
        Arc::new(GetPluginSettingsUseCase::new(
            self.plugins().settings_storage.clone(),
        ))
    }
    fn import_plugins_use_case(&self) -> Arc<dyn ImportPluginsUseCasePort> {
        Arc::new(ImportPluginsUseCase::new(
            self.plugins().extractor.clone(),
            self.plugins().storage.clone(),
            self.sync_plugins_service(),
        ))
    }
    fn list_plugins_dto_use_case(&self) -> Arc<dyn ListPluginsDtoUseCasePort> {
        Arc::new(ListPluginsDtoUseCase::new(self.plugins().registry.clone()))
    }
    fn remove_plugin_use_case(&self) -> Arc<dyn RemovePluginUseCasePort> {
        Arc::new(RemovePluginUseCase::new(
            self.plugins().storage.clone(),
            self.sync_plugins_service(),
        ))
    }
    fn update_plugin_use_case(&self) -> Arc<dyn UpdatePluginUseCasePort> {
        Arc::new(UpdatePluginUseCase::new(
            self.plugins().extractor.clone(),
            self.plugins().storage.clone(),
            self.plugins().source_storage.clone(),
            self.plugins().provider_factory.clone(),
        ))
    }
    fn plugin_registry(&self) -> Arc<PluginRegistry> {
        self.plugins().registry.clone()
    }
    fn plugin_loader_registry(&self) -> Arc<PluginLoaderRegistry> {
        self.plugins().loader_registry.clone()
    }
    fn plugin_provider_factory(&self) -> Arc<PluginProviderFactory> {
        self.plugins().provider_factory.clone()
    }
    fn plugin_source_storage(&self) -> Arc<dyn PluginSourceStorage> {
        self.plugins().source_storage.clone()
    }
    fn plugin_loader(&self) -> Option<Arc<dyn PluginLoader>> {
        self.plugins()
            .loader_registry
            .get(&LoadConfigType::Extism)
            .ok()
            .cloned()
    }
    fn plugin_storage(&self) -> Arc<dyn PluginStorage> {
        self.plugins().storage.clone()
    }
    fn plugin_settings_storage(&self) -> Arc<dyn PluginSettingsStorage> {
        self.plugins().settings_storage.clone()
    }
    fn plugin_extractor(&self) -> Arc<dyn PluginExtractor> {
        self.plugins().extractor.clone()
    }
    fn sync_plugins_service(&self) -> Arc<dyn PluginSyncService> {
        self.sync_plugins_service()
    }
    fn disable_plugin_service(&self) -> Arc<dyn PluginDisableService> {
        self.disable_plugin_service()
    }
}

impl EventsFeature for AetherContainer {
    fn list_progress_bars_use_case(&self) -> Arc<dyn ListProgressBarsUseCasePort> {
        Arc::new(ListProgressBarsUseCase::new(
            self.progress_bar_storage().clone(),
        ))
    }
    fn event_emitter(&self) -> Arc<dyn EventEmitter<Event>> {
        self.event_emitter().clone()
    }
    fn progress_service(&self) -> Arc<dyn ProgressService> {
        self.progress_service().clone()
    }
}
