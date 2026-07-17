pub(crate) mod app;
pub mod domain;
pub mod infra;

// Domain (models + PluginInstance)
pub use domain::{
    ApiConfig, Compatibility, ContentSourceHandlers, ExtractedPlugin, LoadConfig, LoadConfigType,
    ManifestError, PLUGIN_API_VERSION, PathMapping, Plugin, PluginCapabilities,
    PluginCheckCompatibilityParams, PluginContent, PluginContentProviderCapability,
    PluginContentSourceCapability, PluginError, PluginImportInstance, PluginImporterCapability,
    PluginInstance, PluginInternalEvent, PluginManifest, PluginManifestPreview, PluginMetadata,
    PluginSettings, PluginSource, PluginSourceType, PluginState, PluginUpdaterCapability,
    ProviderHandlers, ProviderPluginPreview, ProviderReleaseInfo, ProviderUpdateInfo,
    RuntimeConfig, SerializableOutput,
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
    ImporterCapabilityMetadataDto, ListPluginsDtoUseCase, ListPluginsDtoUseCasePort, LoadConfigDto,
    LoadConfigTypeDto, PathMappingDto, PluginCapabilitiesDto, PluginContentProviderCapabilityDto,
    PluginDisableService, PluginDto, PluginDtoState, PluginExtractor, PluginImporterCapabilityDto,
    PluginLoader, PluginLoaderRegistry, PluginManifestDto, PluginMetadataDto, PluginProvider,
    PluginProviderFactory, PluginRegistry, PluginSettingsStorage, PluginSourceStorage,
    PluginStorage, PluginSyncService, PluginUpdaterCapabilityDto, PluginsFeature,
    ProviderHandlersDto, RemovePluginUseCase, RemovePluginUseCasePort, RuntimeConfigDto,
    SyncPluginsUseCase, UpdatePluginUseCase, UpdatePluginUseCasePort, UpdaterCapabilityMetadataDto,
    write_bytes_to_temp_file,
};

// Infrastructure re-exports (commonly used by app layer)
pub use infra::{
    ExtismPluginLoader, FsPluginSettingsStorage, FsPluginSourceStorage, FsPluginStorage,
    GitHubPluginFetcher, GithubProvider, PluginInfrastructureListener, ZipPluginExtractor,
};
