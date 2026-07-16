pub(crate) mod app;
pub mod domain;
pub mod infra;

// Domain (models + PluginInstance)
pub use domain::{
    ApiConfig, Compatibility, ExtractedPlugin, LoadConfig, LoadConfigType, ManifestError,
    PLUGIN_API_VERSION, PackManagerHandlers, PathMapping, Plugin, PluginCapabilities,
    PluginCheckCompatibilityParams, PluginContent, PluginContentProviderCapability, PluginError,
    PluginImportInstance, PluginInstance, PluginInternalEvent, PluginManifest,
    PluginManifestPreview, PluginMetadata, PluginPackManagerCapability, PluginSettings,
    PluginSource, PluginSourceType, PluginState, ProviderHandlers, ProviderPluginPreview,
    ProviderReleaseInfo, ProviderUpdateInfo, RuntimeConfig, SerializableOutput,
};

// App (use cases, ports, services, DTOs)
pub use app::{
    ApiConfigDto, AsCapabilityMetadata, CapabilityMetadataDto, CheckForPluginUpdatesUseCase,
    CheckForPluginUpdatesUseCasePort, ContentProviderCapabilityMetadataDto, DisablePluginUseCase,
    EditPluginSettings, EditPluginSettingsUseCase, EditPluginSettingsUseCasePort,
    EnablePluginUseCase, EnablePluginUseCasePort, ForceEnablePluginUseCase,
    ForceEnablePluginUseCasePort, GetPluginApiVersionUseCase, GetPluginApiVersionUseCasePort,
    GetPluginDtoUseCase, GetPluginDtoUseCasePort, GetPluginSettingsUseCase,
    GetPluginSettingsUseCasePort, ImportPluginsUseCase, ImportPluginsUseCasePort,
    ListPluginsDtoUseCase, ListPluginsDtoUseCasePort, LoadConfigDto, LoadConfigTypeDto,
    PackManagerCapabilityMetadataDto, PackManagerHandlersDto, PathMappingDto,
    PluginCapabilitiesDto, PluginContentProviderCapabilityDto, PluginDisableService, PluginDto,
    PluginDtoState, PluginExtractor, PluginLoader, PluginLoaderRegistry, PluginManifestDto,
    PluginMetadataDto, PluginPackManagerCapabilityDto, PluginProvider, PluginProviderFactory,
    PluginRegistry, PluginSettingsStorage, PluginSourceStorage, PluginStorage, PluginSyncService,
    PluginsFeature, ProviderHandlersDto, RemovePluginUseCase, RemovePluginUseCasePort,
    RuntimeConfigDto, SyncPluginsUseCase, UpdatePluginUseCase, UpdatePluginUseCasePort,
    write_bytes_to_temp_file,
};

// Infrastructure re-exports (commonly used by app layer)
pub use infra::{
    ExtismPluginLoader, FsPluginSettingsStorage, FsPluginSourceStorage, FsPluginStorage,
    GitHubPluginFetcher, GithubProvider, PluginInfrastructureListener, ZipPluginExtractor,
};
