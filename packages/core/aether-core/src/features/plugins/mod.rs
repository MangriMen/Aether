pub(crate) mod app;
pub mod domain;
pub mod infra;

// Domain (models + PluginInstance)
pub use domain::{
    ApiConfig, Compatibility, ExtractedPlugin, LoadConfig, LoadConfigType, ManifestError,
    PLUGIN_API_VERSION, PathMapping, Plugin, PluginCapabilities, PluginCheckCompatibilityParams,
    PluginContent, PluginContentProviderCapability, PluginError, PluginImportInstance,
    PluginImporterCapability, PluginInstance, PluginInternalEvent, PluginManifest,
    PluginManifestPreview, PluginMetadata, PluginSettings, PluginSource, PluginSourceType,
    PluginState, PluginUpdaterCapability, ProviderHandlers, ProviderPluginPreview,
    ProviderReleaseInfo, ProviderUpdateInfo, RuntimeConfig, SerializableOutput,
};

// App (use cases, ports, services, DTOs)
pub use app::{
    ApiConfigDto, AsCapabilityMetadata, CapabilityMetadataDto, CheckForPluginUpdatesUseCase,
    ContentProviderCapabilityMetadataDto, DisablePluginUseCase, EditPluginSettings,
    EditPluginSettingsUseCase, EnablePluginUseCase, ForceEnablePluginUseCase,
    GetPluginApiVersionUseCase, GetPluginDtoUseCase, GetPluginSettingsUseCase,
    ImportPluginsUseCase, ImporterCapabilityMetadataDto, ListPluginsDtoUseCase, LoadConfigDto,
    LoadConfigTypeDto, PathMappingDto, PluginCapabilitiesDto, PluginContentProviderCapabilityDto,
    PluginDto, PluginDtoState, PluginExtractor, PluginImporterCapabilityDto, PluginLoader,
    PluginLoaderRegistry, PluginManifestDto, PluginMetadataDto, PluginProvider,
    PluginProviderFactory, PluginRegistry, PluginSettingsStorage, PluginSourceStorage,
    PluginStorage, PluginSyncService, PluginUpdaterCapabilityDto, ProviderHandlersDto,
    RemovePluginUseCase, RuntimeConfigDto, SyncPluginsUseCase, UpdatePluginUseCase,
    UpdaterCapabilityMetadataDto, write_bytes_to_temp_file,
};

// Infrastructure re-exports (commonly used by app layer)
pub use infra::{
    ExtismPluginLoader, FsPluginSettingsStorage, FsPluginSourceStorage, FsPluginStorage,
    GitHubPluginFetcher, PluginInfrastructureListener, ZipPluginExtractor,
};
