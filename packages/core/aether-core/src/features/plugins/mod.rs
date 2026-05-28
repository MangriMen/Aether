mod app;
mod domain;
pub(crate) mod infra;

// Domain (models + PluginInstance/PluginInstanceExt)
pub use domain::{
    ApiConfig, ExtractedPlugin, LoadConfig, LoadConfigType, ManifestError, PLUGIN_API_VERSION,
    PathMapping, Plugin, PluginCapabilities, PluginCheckCompatibilityParams, PluginContent,
    PluginContentProviderCapability, PluginError, PluginImportInstance, PluginImporterCapability,
    PluginInstance, PluginInstanceExt, PluginInternalEvent, PluginManifest, PluginMetadata,
    PluginSettings, PluginState, PluginUpdaterCapability, ProviderHandlers, RuntimeConfig,
    SerializableOutput,
};

// App (use cases, ports, services, DTOs)
pub use app::{
    AsCapabilityMetadata, DisablePluginUseCase, EditPluginSettings, EditPluginSettingsUseCase,
    EnablePluginUseCase, GetPluginApiVersionUseCase, GetPluginDtoUseCase, GetPluginSettingsUseCase,
    ImportPluginsUseCase, ListPluginsDtoUseCase, PluginDto, PluginDtoState, PluginExtractor,
    PluginLoader, PluginLoaderRegistry, PluginRegistry, PluginSettingsStorage, PluginStorage,
    RemovePluginUseCase, SyncPluginsUseCase,
};
