use std::sync::{Arc, OnceLock};

use crate::features::{
    auth::{
        ActiveAccountHelper, AuthFeature, CreateOfflineAccountUseCase,
        CreateOfflineAccountUseCasePort, CredentialsStorage, GetAccountsUseCase,
        GetAccountsUseCasePort, LogoutUseCase, LogoutUseCasePort, SetActiveAccountUseCase,
        SetActiveAccountUseCasePort,
    },
    events::{
        Event, EventEmitter, EventsFeature, ListProgressBarsUseCase, ListProgressBarsUseCasePort,
        ProgressBarStorage, ProgressService, SharedEventEmitter,
    },
    file_watcher::{FileEventHandler, FileWatcher, FileWatcherFeature},
    instance::{
        ChangeContentStateUseCase, ChangeContentStateUseCasePort, CheckContentCompatibilityUseCase,
        CheckContentCompatibilityUseCasePort, ContentFileService, ContentProvider,
        CreateInstanceUseCase, CreateInstanceUseCasePort, EditInstanceIconUseCase,
        EditInstanceIconUseCasePort, EditInstanceUseCase, EditInstanceUseCasePort,
        GetContentUseCase, GetContentUseCasePort, GetInstanceUseCase, GetInstanceUseCasePort,
        ImportContentUseCase, ImportContentUseCasePort, ImportInstanceUseCase,
        ImportInstanceUseCasePort, Importer, InstallContentUseCase, InstallContentUseCasePort,
        InstallInstanceUseCase, InstanceFeature, InstanceFileService, InstanceInstallService,
        InstanceLaunchService, InstanceStorage, InstanceWatcherService, LaunchInstanceUseCase,
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
        InstallJavaUseCase, JavaFeature, JavaInstallService, JavaInstallationService,
        JavaInstallationTracker, JavaQueryService, JavaStorage, JreProvider, ListJavaUseCase,
        ListJavaUseCasePort, RemoveJavaUseCase, RemoveJavaUseCasePort, TestJreUseCase,
        TestJreUseCasePort, infra::get_default_discovery_paths,
    },
    minecraft::{
        GetLoaderVersionManifestUseCase, GetLoaderVersionManifestUseCasePort,
        GetMinecraftLaunchCommandUseCase, GetVersionManifestUseCase, InstallMinecraftUseCase,
        LoaderVersionResolver, LoaderVersionService, MetadataStorage, MinecraftDownloader,
        MinecraftFeature, MinecraftFileHealthService, MinecraftHealthService,
        MinecraftInstallService, MinecraftLaunchCommandService, ModLoaderProcessor,
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
        DefaultInstanceSettingsStorage, EditDefaultInstanceSettingsUseCase,
        EditDefaultInstanceSettingsUseCasePort, EditSettingsUseCase, EditSettingsUseCasePort,
        GetDefaultInstanceSettingsUseCase, GetDefaultInstanceSettingsUseCasePort,
        GetSettingsUseCase, GetSettingsUseCasePort, LocationInfo, SettingsFeature, SettingsStorage,
    },
};
use crate::shared::{cache::domain::AssetsStorage, capability::domain::CapabilityRegistry};

struct UseCaseCache {
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

/// Dependency injection parameters for constructing an [`AetherContainer`].
///
/// All dependencies are expressed as trait objects or core domain types —
/// no concrete infrastructure types are imported here.
#[allow(clippy::too_many_lines)]
pub struct AetherContainerParams {
    pub credentials_storage: Arc<dyn CredentialsStorage>,
    pub settings_storage: Arc<dyn SettingsStorage>,
    pub default_instance_settings_storage: Arc<dyn DefaultInstanceSettingsStorage>,
    pub process_storage: Arc<dyn ProcessStorage>,
    pub instance_storage: Arc<dyn InstanceStorage>,
    pub pack_storage: Arc<dyn PackStorage>,
    pub content_file_service: Arc<dyn ContentFileService>,
    pub instance_file_service: Arc<dyn InstanceFileService>,
    pub java_storage: Arc<dyn JavaStorage>,
    pub java_installation_service: Arc<dyn JavaInstallationService>,
    pub java_installation_tracker: Arc<dyn JavaInstallationTracker>,
    pub jre_provider: Arc<dyn JreProvider>,
    pub metadata_storage: Arc<dyn MetadataStorage>,
    pub minecraft_downloader: Arc<dyn MinecraftDownloader>,
    pub forge_processor: Arc<dyn ModLoaderProcessor>,
    pub plugin_registry: Arc<PluginRegistry>,
    pub plugin_loader_registry: Arc<PluginLoaderRegistry>,
    pub plugin_storage: Arc<dyn PluginStorage>,
    pub plugin_source_storage: Arc<dyn PluginSourceStorage>,
    pub plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
    pub plugin_provider_factory: Arc<PluginProviderFactory>,
    pub plugin_extractor: Arc<dyn PluginExtractor>,
    pub event_emitter: SharedEventEmitter,
    pub progress_bar_storage: Arc<dyn ProgressBarStorage>,
    pub progress_service: Arc<dyn ProgressService>,
    pub location_info: Arc<LocationInfo>,
    pub instance_watcher_service: Arc<dyn InstanceWatcherService>,
    pub importers_registry: Arc<dyn CapabilityRegistry<Arc<dyn Importer>>>,
    pub updaters_registry: Arc<dyn CapabilityRegistry<Arc<dyn Updater>>>,
    pub content_provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
    pub assets_storage: Arc<dyn AssetsStorage>,
}

pub struct AetherContainer {
    cache: UseCaseCache,
    credentials_storage: Arc<dyn CredentialsStorage>,
    settings_storage: Arc<dyn SettingsStorage>,
    default_instance_settings_storage: Arc<dyn DefaultInstanceSettingsStorage>,
    process_storage: Arc<dyn ProcessStorage>,
    instance_storage: Arc<dyn InstanceStorage>,
    pack_storage: Arc<dyn PackStorage>,
    content_file_service: Arc<dyn ContentFileService>,
    instance_file_service: Arc<dyn InstanceFileService>,
    java_storage: Arc<dyn JavaStorage>,
    java_installation_service: Arc<dyn JavaInstallationService>,
    java_installation_tracker: Arc<dyn JavaInstallationTracker>,
    jre_provider: Arc<dyn JreProvider>,
    metadata_storage: Arc<dyn MetadataStorage>,
    minecraft_downloader: Arc<dyn MinecraftDownloader>,
    forge_processor: Arc<dyn ModLoaderProcessor>,
    plugin_registry: Arc<PluginRegistry>,
    plugin_loader_registry: Arc<PluginLoaderRegistry>,
    plugin_storage: Arc<dyn PluginStorage>,
    plugin_source_storage: Arc<dyn PluginSourceStorage>,
    plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
    plugin_provider_factory: Arc<PluginProviderFactory>,
    plugin_extractor: Arc<dyn PluginExtractor>,
    event_emitter: SharedEventEmitter,
    progress_bar_storage: Arc<dyn ProgressBarStorage>,
    progress_service: Arc<dyn ProgressService>,
    location_info: Arc<LocationInfo>,
    instance_watcher_service: Arc<dyn InstanceWatcherService>,
    importers_registry: Arc<dyn CapabilityRegistry<Arc<dyn Importer>>>,
    updaters_registry: Arc<dyn CapabilityRegistry<Arc<dyn Updater>>>,
    content_provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
    assets_storage: Arc<dyn AssetsStorage>,
}

impl AetherContainer {
    /// Construct a new [`AetherContainer`] from pre-built dependencies.
    ///
    /// This constructor does not import any concrete infrastructure types —
    /// all dependencies are provided as trait objects via [`AetherContainerParams`].
    pub fn new(params: AetherContainerParams) -> Arc<Self> {
        Arc::new(Self {
            cache: UseCaseCache::new(),
            credentials_storage: params.credentials_storage,
            settings_storage: params.settings_storage,
            default_instance_settings_storage: params.default_instance_settings_storage,
            process_storage: params.process_storage,
            instance_storage: params.instance_storage,
            pack_storage: params.pack_storage,
            content_file_service: params.content_file_service,
            instance_file_service: params.instance_file_service,
            java_storage: params.java_storage,
            java_installation_service: params.java_installation_service,
            java_installation_tracker: params.java_installation_tracker,
            jre_provider: params.jre_provider,
            metadata_storage: params.metadata_storage,
            minecraft_downloader: params.minecraft_downloader,
            forge_processor: params.forge_processor,
            plugin_registry: params.plugin_registry,
            plugin_loader_registry: params.plugin_loader_registry,
            plugin_storage: params.plugin_storage,
            plugin_source_storage: params.plugin_source_storage,
            plugin_settings_storage: params.plugin_settings_storage,
            plugin_provider_factory: params.plugin_provider_factory,
            plugin_extractor: params.plugin_extractor,
            event_emitter: params.event_emitter,
            progress_bar_storage: params.progress_bar_storage,
            progress_service: params.progress_service,
            location_info: params.location_info,
            instance_watcher_service: params.instance_watcher_service,
            importers_registry: params.importers_registry,
            updaters_registry: params.updaters_registry,
            content_provider_registry: params.content_provider_registry,
            assets_storage: params.assets_storage,
        })
    }

    pub fn progress_bar_storage(&self) -> Arc<dyn ProgressBarStorage> {
        self.progress_bar_storage.clone()
    }

    pub fn event_emitter(&self) -> SharedEventEmitter {
        self.event_emitter.clone()
    }

    fn loader_version_resolver(&self) -> Arc<dyn LoaderVersionService> {
        Arc::new(LoaderVersionResolver::new(self.metadata_storage.clone()))
    }

    fn get_version_manifest_use_case(&self) -> Arc<dyn VersionManifestService> {
        Arc::new(GetVersionManifestUseCase::new(
            self.metadata_storage.clone(),
        ))
    }

    fn get_java_use_case(&self) -> Arc<dyn JavaQueryService> {
        Arc::new(GetJavaUseCase::new(
            self.java_storage.clone(),
            self.java_installation_service.clone(),
        ))
    }

    fn install_java_use_case(&self) -> Arc<dyn JavaInstallService> {
        Arc::new(InstallJavaUseCase::new(
            self.java_storage.clone(),
            self.java_installation_service.clone(),
            self.jre_provider.clone(),
            self.location_info.clone(),
            self.java_installation_tracker.clone(),
        ))
    }

    fn minecraft_install_use_case(&self) -> Arc<dyn MinecraftInstallService> {
        self.cache
            .minecraft_install_uc
            .get_or_init(|| {
                Arc::new(InstallMinecraftUseCase::new(
                    self.loader_version_resolver(),
                    self.get_version_manifest_use_case(),
                    self.minecraft_downloader.clone(),
                    self.forge_processor.clone(),
                    self.java_installation_service.clone(),
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
                    self.instance_storage.clone(),
                    self.minecraft_install_use_case(),
                    self.progress_service.clone(),
                    self.location_info.clone(),
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
                    self.minecraft_downloader.clone(),
                    self.get_java_use_case(),
                    self.location_info.clone(),
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
                    self.minecraft_downloader.clone(),
                    self.java_installation_service.clone(),
                    self.get_java_use_case(),
                    self.install_java_use_case(),
                    self.location_info.clone(),
                ))
            })
            .clone()
    }

    fn launch_instance_use_case(&self) -> Arc<dyn InstanceLaunchService> {
        self.cache
            .launch_instance_uc
            .get_or_init(|| {
                let manage = self.manage_process_service();
                let start: Arc<dyn ProcessStartService> = Arc::new(StartProcessUseCase::new(
                    self.event_emitter.clone(),
                    self.process_storage.clone(),
                    manage,
                ));
                Arc::new(LaunchInstanceUseCase::new(
                    self.plugin_registry.clone(),
                    self.instance_storage.clone(),
                    self.default_instance_settings_storage.clone(),
                    self.location_info.clone(),
                    self.process_storage.clone(),
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
                    self.process_storage.clone(),
                    self.instance_storage.clone(),
                ))
            })
            .clone()
    }

    fn manage_process_service(&self) -> Arc<dyn ManageProcessService> {
        self.cache
            .manage_process_uc
            .get_or_init(|| {
                Arc::new(ManageProcessUseCase::new(
                    self.event_emitter.clone(),
                    self.process_storage.clone(),
                    self.track_process_service(),
                    self.location_info.clone(),
                ))
            })
            .clone()
    }

    fn process_start_service(&self) -> Arc<dyn ProcessStartService> {
        self.cache
            .start_process_uc
            .get_or_init(|| {
                Arc::new(StartProcessUseCase::new(
                    self.event_emitter.clone(),
                    self.process_storage.clone(),
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
                    self.plugin_registry.clone(),
                    self.plugin_loader_registry.clone(),
                    self.settings_storage.clone(),
                ))
            })
            .clone()
    }

    fn sync_plugins_service(&self) -> Arc<dyn PluginSyncService> {
        self.cache
            .sync_plugins_uc
            .get_or_init(|| {
                Arc::new(SyncPluginsUseCase::new(
                    self.plugin_storage.clone(),
                    self.plugin_registry.clone(),
                    self.disable_plugin_service(),
                    self.event_emitter.clone(),
                ))
            })
            .clone()
    }
}

// Feature trait implementations

impl SettingsFeature for AetherContainer {
    fn get_settings_use_case(&self) -> Arc<dyn GetSettingsUseCasePort> {
        Arc::new(GetSettingsUseCase::new(self.settings_storage.clone()))
    }
    fn get_default_instance_settings_use_case(
        &self,
    ) -> Arc<dyn GetDefaultInstanceSettingsUseCasePort> {
        Arc::new(GetDefaultInstanceSettingsUseCase::new(
            self.default_instance_settings_storage.clone(),
        ))
    }
    fn edit_settings_use_case(&self) -> Arc<dyn EditSettingsUseCasePort> {
        Arc::new(EditSettingsUseCase::new(self.settings_storage.clone()))
    }
    fn edit_default_instance_settings_use_case(
        &self,
    ) -> Arc<dyn EditDefaultInstanceSettingsUseCasePort> {
        Arc::new(EditDefaultInstanceSettingsUseCase::new(
            self.default_instance_settings_storage.clone(),
        ))
    }
    fn settings_storage(&self) -> Arc<dyn SettingsStorage> {
        self.settings_storage.clone()
    }
    fn default_instance_settings_storage(&self) -> Arc<dyn DefaultInstanceSettingsStorage> {
        self.default_instance_settings_storage.clone()
    }
    fn location_info(&self) -> Arc<LocationInfo> {
        self.location_info.clone()
    }
}

impl AuthFeature for AetherContainer {
    fn create_offline_account_use_case(&self) -> Arc<dyn CreateOfflineAccountUseCasePort> {
        Arc::new(CreateOfflineAccountUseCase::new(
            self.credentials_storage.clone(),
        ))
    }
    fn get_accounts_use_case(&self) -> Arc<dyn GetAccountsUseCasePort> {
        Arc::new(GetAccountsUseCase::new(self.credentials_storage.clone()))
    }
    fn logout_use_case(&self) -> Arc<dyn LogoutUseCasePort> {
        Arc::new(LogoutUseCase::new(self.credentials_storage.clone()))
    }
    fn set_active_account_use_case(&self) -> Arc<dyn SetActiveAccountUseCasePort> {
        Arc::new(SetActiveAccountUseCase::new(
            self.credentials_storage.clone(),
        ))
    }
    fn active_account_helper(&self) -> Arc<ActiveAccountHelper> {
        Arc::new(ActiveAccountHelper)
    }
}

impl JavaFeature for AetherContainer {
    fn discover_java_use_case(&self) -> Arc<dyn DiscoverJavaUseCasePort> {
        Arc::new(DiscoverJavaUseCase::new(
            self.java_installation_service.clone(),
            get_default_discovery_paths(),
        ))
    }
    fn edit_java_use_case(&self) -> Arc<dyn EditJavaUseCasePort> {
        Arc::new(EditJavaUseCase::new(
            self.java_storage.clone(),
            self.java_installation_service.clone(),
        ))
    }
    fn get_active_java_installations_use_case(
        &self,
    ) -> Arc<dyn GetActiveJavaInstallationsUseCasePort> {
        Arc::new(GetActiveJavaInstallationsUseCase::new(
            self.java_installation_tracker.clone(),
        ))
    }
    fn list_java_use_case(&self) -> Arc<dyn ListJavaUseCasePort> {
        Arc::new(ListJavaUseCase::new(self.java_storage.clone()))
    }
    fn remove_java_use_case(&self) -> Arc<dyn RemoveJavaUseCasePort> {
        Arc::new(RemoveJavaUseCase::new(self.java_storage.clone()))
    }
    fn test_jre_use_case(&self) -> Arc<dyn TestJreUseCasePort> {
        Arc::new(TestJreUseCase::new(self.java_installation_service.clone()))
    }
    fn get_java_use_case(&self) -> Arc<dyn JavaQueryService> {
        self.get_java_use_case()
    }
    fn install_java_use_case(&self) -> Arc<dyn JavaInstallService> {
        self.install_java_use_case()
    }
    fn java_installation_service(&self) -> Arc<dyn JavaInstallationService> {
        self.java_installation_service.clone()
    }
    fn java_installation_tracker(&self) -> Arc<dyn JavaInstallationTracker> {
        self.java_installation_tracker.clone()
    }
    fn java_storage(&self) -> Arc<dyn JavaStorage> {
        self.java_storage.clone()
    }
    fn jre_provider(&self) -> Arc<dyn JreProvider> {
        self.jre_provider.clone()
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
            self.metadata_storage.clone(),
        ))
    }
    fn loader_version_service(&self) -> Arc<dyn LoaderVersionService> {
        self.loader_version_resolver()
    }
    fn metadata_storage(&self) -> Arc<dyn MetadataStorage> {
        self.metadata_storage.clone()
    }
    fn minecraft_downloader(&self) -> Arc<dyn MinecraftDownloader> {
        self.minecraft_downloader.clone()
    }
    fn minecraft_health_service(&self) -> Arc<dyn MinecraftHealthService> {
        self.health_service()
    }
    fn mod_loader_processor(&self) -> Arc<dyn ModLoaderProcessor> {
        self.forge_processor.clone()
    }
}

impl InstanceFeature for AetherContainer {
    fn create_instance_use_case(&self) -> Arc<dyn CreateInstanceUseCasePort> {
        Arc::new(CreateInstanceUseCase::new(
            self.instance_storage.clone(),
            self.loader_version_resolver(),
            self.instance_install_service(),
            self.location_info.clone(),
            self.event_emitter.clone(),
            self.instance_watcher_service.clone(),
            self.instance_file_service.clone(),
        ))
    }
    fn get_instance_use_case(&self) -> Arc<dyn GetInstanceUseCasePort> {
        Arc::new(GetInstanceUseCase::new(self.instance_storage.clone()))
    }
    fn list_instances_use_case(&self) -> Arc<dyn ListInstancesUseCasePort> {
        Arc::new(ListInstancesUseCase::new(self.instance_storage.clone()))
    }
    fn edit_instance_use_case(&self) -> Arc<dyn EditInstanceUseCasePort> {
        Arc::new(EditInstanceUseCase::new(self.instance_storage.clone()))
    }
    fn edit_instance_icon_use_case(&self) -> Arc<dyn EditInstanceIconUseCasePort> {
        Arc::new(EditInstanceIconUseCase::new(
            self.instance_storage.clone(),
            self.assets_storage.clone(),
        ))
    }
    fn remove_instance_use_case(&self) -> Arc<dyn RemoveInstanceUseCasePort> {
        Arc::new(RemoveInstanceUseCase::new(
            self.instance_storage.clone(),
            self.instance_watcher_service.clone(),
            self.instance_file_service.clone(),
            self.pack_storage.clone(),
        ))
    }
    fn update_instance_use_case(&self) -> Arc<dyn UpdateInstanceUseCasePort> {
        Arc::new(UpdateInstanceUseCase::new(
            self.instance_storage.clone(),
            self.updaters_registry.clone(),
        ))
    }
    fn import_instance_use_case(&self) -> Arc<dyn ImportInstanceUseCasePort> {
        Arc::new(ImportInstanceUseCase::new(self.importers_registry.clone()))
    }
    fn list_importers_use_case(&self) -> Arc<dyn ListImportersUseCasePort> {
        Arc::new(ListImportersUseCase::new(self.importers_registry.clone()))
    }
    fn launch_instance_with_active_account_use_case(
        &self,
    ) -> Arc<dyn LaunchInstanceWithActiveAccountUseCasePort> {
        Arc::new(LaunchInstanceWithActiveAccountUseCase::new(
            self.credentials_storage.clone(),
            self.launch_instance_use_case(),
        ))
    }
    fn change_content_state_use_case(&self) -> Arc<dyn ChangeContentStateUseCasePort> {
        Arc::new(ChangeContentStateUseCase::new(
            self.event_emitter.clone(),
            self.content_file_service.clone(),
        ))
    }
    fn import_content_use_case(&self) -> Arc<dyn ImportContentUseCasePort> {
        Arc::new(ImportContentUseCase::new(
            self.event_emitter.clone(),
            self.pack_storage.clone(),
            self.location_info.clone(),
        ))
    }
    fn list_content_use_case(&self) -> Arc<dyn ListContentUseCasePort> {
        Arc::new(ListContentUseCase::new(
            self.pack_storage.clone(),
            self.location_info.clone(),
        ))
    }
    fn remove_content_use_case(&self) -> Arc<dyn RemoveContentUseCasePort> {
        Arc::new(RemoveContentUseCase::new(
            self.event_emitter.clone(),
            self.pack_storage.clone(),
            self.content_file_service.clone(),
        ))
    }
    fn search_content_use_case(&self) -> Arc<dyn SearchContentUseCasePort> {
        Arc::new(SearchContentUseCase::new(
            self.content_provider_registry.clone(),
        ))
    }
    fn get_content_use_case(&self) -> Arc<dyn GetContentUseCasePort> {
        Arc::new(GetContentUseCase::new(
            self.content_provider_registry.clone(),
        ))
    }
    fn install_content_use_case(&self) -> Arc<dyn InstallContentUseCasePort> {
        Arc::new(InstallContentUseCase::new(
            self.pack_storage.clone(),
            self.content_provider_registry.clone(),
            self.content_file_service.clone(),
        ))
    }
    fn check_content_compatibility_use_case(
        &self,
    ) -> Arc<dyn CheckContentCompatibilityUseCasePort> {
        Arc::new(CheckContentCompatibilityUseCase::new(
            self.content_provider_registry.clone(),
            self.instance_storage.clone(),
        ))
    }
    fn list_content_versions_use_case(&self) -> Arc<dyn ListContentVersionsUseCasePort> {
        Arc::new(ListContentVersionsUseCase::new(
            self.content_provider_registry.clone(),
        ))
    }
    fn list_providers_use_case(&self) -> Arc<dyn ListProvidersUseCasePort> {
        Arc::new(ListProvidersUseCase::new(
            self.content_provider_registry.clone(),
        ))
    }
    fn instance_storage(&self) -> Arc<dyn InstanceStorage> {
        self.instance_storage.clone()
    }
    fn pack_storage(&self) -> Arc<dyn PackStorage> {
        self.pack_storage.clone()
    }
    fn instance_install_service(&self) -> Arc<dyn InstanceInstallService> {
        self.instance_install_service()
    }
    fn instance_launch_service(&self) -> Arc<dyn InstanceLaunchService> {
        self.launch_instance_use_case()
    }
    fn instance_watcher_service(&self) -> Arc<dyn InstanceWatcherService> {
        self.instance_watcher_service.clone()
    }
    fn instance_file_service(&self) -> Arc<dyn InstanceFileService> {
        self.instance_file_service.clone()
    }
    fn content_file_service(&self) -> Arc<dyn ContentFileService> {
        self.content_file_service.clone()
    }
}

impl ProcessFeature for AetherContainer {
    fn wait_for_process_use_case(&self) -> Arc<dyn WaitForProcessUseCasePort> {
        Arc::new(WaitForProcessUseCase::new(self.process_storage.clone()))
    }
    fn kill_process_use_case(&self) -> Arc<dyn KillProcessUseCasePort> {
        Arc::new(KillProcessUseCase::new(self.process_storage.clone()))
    }
    fn list_process_metadata_use_case(&self) -> Arc<dyn ListProcessMetadataUseCasePort> {
        Arc::new(ListProcessMetadataUseCase::new(
            self.process_storage.clone(),
        ))
    }
    fn get_process_metadata_by_instance_id_use_case(
        &self,
    ) -> Arc<dyn GetProcessMetadataByInstanceIdUseCasePort> {
        Arc::new(GetProcessMetadataByInstanceIdUseCase::new(
            self.process_storage.clone(),
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
    fn process_storage(&self) -> Arc<dyn ProcessStorage> {
        self.process_storage.clone()
    }
}

impl PluginsFeature for AetherContainer {
    fn check_for_plugin_updates_use_case(&self) -> Arc<dyn CheckForPluginUpdatesUseCasePort> {
        Arc::new(CheckForPluginUpdatesUseCase::new(
            self.plugin_source_storage.clone(),
            self.plugin_provider_factory.clone(),
        ))
    }
    fn edit_plugin_settings_use_case(&self) -> Arc<dyn EditPluginSettingsUseCasePort> {
        Arc::new(EditPluginSettingsUseCase::new(
            self.plugin_settings_storage.clone(),
        ))
    }
    fn enable_plugin_use_case(&self) -> Arc<dyn EnablePluginUseCasePort> {
        Arc::new(EnablePluginUseCase::new(
            self.plugin_registry.clone(),
            self.plugin_loader_registry.clone(),
            self.plugin_settings_storage.clone(),
            self.settings_storage.clone(),
        ))
    }
    fn force_enable_plugin_use_case(&self) -> Arc<dyn ForceEnablePluginUseCasePort> {
        Arc::new(ForceEnablePluginUseCase::new(
            self.plugin_registry.clone(),
            self.plugin_loader_registry.clone(),
            self.plugin_settings_storage.clone(),
            self.settings_storage.clone(),
        ))
    }
    fn get_plugin_api_version_use_case(&self) -> Arc<dyn GetPluginApiVersionUseCasePort> {
        Arc::new(GetPluginApiVersionUseCase::default())
    }
    fn get_plugin_dto_use_case(&self) -> Arc<dyn GetPluginDtoUseCasePort> {
        Arc::new(GetPluginDtoUseCase::new(self.plugin_registry.clone()))
    }
    fn get_plugin_settings_use_case(&self) -> Arc<dyn GetPluginSettingsUseCasePort> {
        Arc::new(GetPluginSettingsUseCase::new(
            self.plugin_settings_storage.clone(),
        ))
    }
    fn import_plugins_use_case(&self) -> Arc<dyn ImportPluginsUseCasePort> {
        Arc::new(ImportPluginsUseCase::new(
            self.plugin_extractor.clone(),
            self.plugin_storage.clone(),
            self.sync_plugins_service(),
        ))
    }
    fn list_plugins_dto_use_case(&self) -> Arc<dyn ListPluginsDtoUseCasePort> {
        Arc::new(ListPluginsDtoUseCase::new(self.plugin_registry.clone()))
    }
    fn remove_plugin_use_case(&self) -> Arc<dyn RemovePluginUseCasePort> {
        Arc::new(RemovePluginUseCase::new(
            self.plugin_storage.clone(),
            self.sync_plugins_service(),
        ))
    }
    fn update_plugin_use_case(&self) -> Arc<dyn UpdatePluginUseCasePort> {
        Arc::new(UpdatePluginUseCase::new(
            self.plugin_extractor.clone(),
            self.plugin_storage.clone(),
            self.plugin_source_storage.clone(),
            self.plugin_provider_factory.clone(),
        ))
    }
    fn plugin_registry(&self) -> Arc<PluginRegistry> {
        self.plugin_registry.clone()
    }
    fn plugin_loader_registry(&self) -> Arc<PluginLoaderRegistry> {
        self.plugin_loader_registry.clone()
    }
    fn plugin_provider_factory(&self) -> Arc<PluginProviderFactory> {
        self.plugin_provider_factory.clone()
    }
    fn plugin_source_storage(&self) -> Arc<dyn PluginSourceStorage> {
        self.plugin_source_storage.clone()
    }
    fn plugin_loader(&self) -> Arc<dyn PluginLoader> {
        self.plugin_loader_registry
            .get(&LoadConfigType::Extism)
            .expect("ExtismPluginLoader must be registered")
            .clone()
    }
    fn plugin_storage(&self) -> Arc<dyn PluginStorage> {
        self.plugin_storage.clone()
    }
    fn plugin_settings_storage(&self) -> Arc<dyn PluginSettingsStorage> {
        self.plugin_settings_storage.clone()
    }
    fn plugin_extractor(&self) -> Arc<dyn PluginExtractor> {
        self.plugin_extractor.clone()
    }
    fn sync_plugins_use_case(&self) -> Arc<dyn PluginSyncService> {
        self.sync_plugins_service()
    }
    fn plugin_sync_service(&self) -> Arc<dyn PluginSyncService> {
        self.sync_plugins_service()
    }
    fn disable_plugin_use_case(&self) -> Arc<dyn PluginDisableService> {
        self.disable_plugin_service()
    }
    fn plugin_disable_service(&self) -> Arc<dyn PluginDisableService> {
        self.disable_plugin_service()
    }
}

impl EventsFeature for AetherContainer {
    fn list_progress_bars_use_case(&self) -> Arc<dyn ListProgressBarsUseCasePort> {
        Arc::new(ListProgressBarsUseCase::new(
            self.progress_bar_storage.clone(),
        ))
    }
    fn event_emitter(&self) -> Arc<dyn EventEmitter<Event>> {
        self.event_emitter.clone()
    }
    fn progress_service(&self) -> Arc<dyn ProgressService> {
        self.progress_service.clone()
    }
}

impl FileWatcherFeature for AetherContainer {
    fn file_watcher(&self) -> Arc<dyn FileWatcher> {
        panic!("FileWatcher is not directly exposed; use InstanceWatcherService instead")
    }
    fn file_event_handler(&self) -> Arc<dyn FileEventHandler> {
        panic!("FileEventHandler is not directly exposed")
    }
}
