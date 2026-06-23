mod app;
mod domain;
pub mod infra;

// Domain (models + PluginInstance)
pub use domain::{
    ApiConfig, Compatibility, ExtractedPlugin, LoadConfig, LoadConfigType, ManifestError,
    PLUGIN_API_VERSION, PathMapping, Plugin, PluginCapabilities, PluginCheckCompatibilityParams,
    PluginContent, PluginContentProviderCapability, PluginError, PluginImportInstance,
    PluginImporterCapability, PluginInstance, PluginInternalEvent, PluginManifest, PluginMetadata,
    PluginSettings, PluginState, PluginUpdaterCapability, ProviderHandlers, RuntimeConfig,
    SerializableOutput,
};

// App (use cases, ports, services, DTOs)
pub use app::{
    ApiConfigDto, AsCapabilityMetadata, CapabilityMetadataDto,
    ContentProviderCapabilityMetadataDto, DisablePluginUseCase, EditPluginSettings,
    EditPluginSettingsUseCase, EnablePluginUseCase, ForceEnablePluginUseCase,
    GetPluginApiVersionUseCase, GetPluginDtoUseCase, GetPluginSettingsUseCase,
    ImportPluginsUseCase, ImporterCapabilityMetadataDto, ListPluginsDtoUseCase, LoadConfigDto,
    LoadConfigTypeDto, PathMappingDto, PluginCapabilitiesDto, PluginContentProviderCapabilityDto,
    PluginDto, PluginDtoState, PluginExtractor, PluginImporterCapabilityDto, PluginLoader,
    PluginLoaderRegistry, PluginManifestDto, PluginMetadataDto, PluginRegistry,
    PluginSettingsStorage, PluginStorage, PluginUpdaterCapabilityDto, ProviderHandlersDto,
    RemovePluginUseCase, RuntimeConfigDto, SyncPluginsUseCase, UpdaterCapabilityMetadataDto,
};
