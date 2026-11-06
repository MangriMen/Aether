mod app;
mod domain;
pub mod infra;

#[cfg(test)]
mod tests;

pub use domain::{
    AllowedHostRejection, AllowedPath, AllowedPathInput, AllowedPathRejection, ApiConfig,
    IgnoredAllowedPath, LoadConfig, LoadConfigType, PathMapping, PluginCapabilities,
    PluginContentProviderCapability, PluginError, PluginImporterCapability, PluginManifest,
    PluginManifestPreview, PluginMetadata, PluginProviderError, PluginSettings, PluginSource,
    PluginSourceType, PluginUpdaterCapability, ProviderHandlers, ProviderPluginPreview,
    ProviderUpdateInfo, RuntimeConfig, SkippedPluginDir, SkippedPluginDirReason,
};
pub(crate) use domain::{
    AsCapabilityMetadata, Compatibility, CoreMount, ExtractedPlugin, FolderMounts, GrantableHost,
    ManifestError, PLUGIN_API_VERSION, PLUGIN_CACHE_MOUNT, Plugin, PluginCallError,
    PluginCheckCompatibilityParams, PluginContent, PluginHandle, PluginImportInstance,
    PluginInstance, PluginInternalEvent, PluginPath, PluginState, PluginStateReason,
    PluginVerification, core_folders, effective_allowed_hosts, is_blocked_ip, is_filesystem_root,
    is_host_allowed, partition_mountable, to_wasi_path,
};

pub use app::{
    ApiConfigDto, CapabilityMetadataDto, ContentProviderCapabilityMetadataDto, EditPluginSettings,
    FolderAccess, HostAccess, ImporterCapabilityMetadataDto, LoadConfigDto, PathMappingDto,
    PluginCapabilitiesDto, PluginContentProviderCapabilityDto, PluginDto, PluginDtoState,
    PluginExtractor, PluginImporterCapabilityDto, PluginLoader, PluginLoaderRegistry,
    PluginManifestDto, PluginMetadataDto, PluginProvider, PluginProviderFactory, PluginRegistry,
    PluginSettingsStorage, PluginSourceStorage, PluginStateReasonDto, PluginStorage,
    PluginUpdaterCapabilityDto, PluginVerificationStorage, PluginsFeature, ProviderHandlersDto,
    ProviderRelease, RuntimeConfigDto, UpdaterCapabilityMetadataDto, download_to_temp_file,
    record_plugin_verification,
};
pub(crate) use app::{
    CheckForPluginUpdatesUseCase, CheckForPluginUpdatesUseCasePort, DisablePluginUseCase,
    EditPluginSettingsUseCase, EditPluginSettingsUseCasePort, EnablePluginUseCase,
    EnablePluginUseCasePort, ForceEnablePluginUseCase, ForceEnablePluginUseCasePort,
    GetPluginApiVersionUseCase, GetPluginApiVersionUseCasePort, GetPluginDtoUseCase,
    GetPluginDtoUseCasePort, GetPluginSettingsUseCase, GetPluginSettingsUseCasePort,
    GrantPluginFolderAccessUseCase, GrantPluginFolderAccessUseCasePort,
    GrantPluginHostAccessUseCase, GrantPluginHostAccessUseCasePort, ImportPluginsUseCase,
    ImportPluginsUseCasePort, InstallPluginFromProviderUseCase,
    InstallPluginFromProviderUseCasePort, ListPluginsDtoUseCase, ListPluginsDtoUseCasePort,
    ListSkippedPluginDirsUseCase, ListSkippedPluginDirsUseCasePort, PluginDisableService,
    PluginSyncService, ReloadPluginUseCase, ReloadPluginUseCasePort, RemovePluginUseCase,
    RemovePluginUseCasePort, SyncPluginsUseCase, UpdatePluginUseCase, UpdatePluginUseCasePort,
};
