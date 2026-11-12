mod app;
mod domain;
pub mod infra;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) mod test_support;

pub use domain::{
    AllowedHostEntryRejection, AllowedHostRejection, AllowedPath, AllowedPathInput,
    AllowedPathRejection, ApiConfig, IgnoredAllowedPath, LoadConfig, LoadConfigType, PathMapping,
    PluginCapabilities, PluginContentProviderCapability, PluginError, PluginImporterCapability,
    PluginManifest, PluginManifestPreview, PluginMetadata, PluginProviderError, PluginProviderInfo,
    PluginReleasePreview, PluginSettings, PluginSource, PluginSourceType, PluginUpdaterCapability,
    ProviderHandlers, ProviderPluginPreview, ProviderReleaseInfo, ProviderUpdateInfo,
    RuntimeConfig, SkippedPluginDir, SkippedPluginDirReason,
};
pub(crate) use domain::{
    ArchiveMetadata, AsCapabilityMetadata, Compatibility, CoreMount, ExtractedPlugin, FolderMounts,
    GrantableHost, ManifestError, PLUGIN_API_VERSION, PLUGIN_CACHE_MOUNT, Plugin, PluginCallError,
    PluginCheckCompatibilityParams, PluginContent, PluginHandle, PluginImportInstance,
    PluginInstance, PluginInternalEvent, PluginPath, PluginState, PluginStateReason,
    PluginVerification, core_folders, effective_allowed_hosts, is_blocked_ip, is_filesystem_root,
    is_host_allowed, partition_mountable, to_wasi_path, validate_allowed_host_entry,
};

pub use app::{
    ApiConfigDto, CapabilityMetadataDto, ContentProviderCapabilityMetadataDto, DownloadedArchive,
    EditPluginSettings, FolderAccess, HostAccess, ImporterCapabilityMetadataDto, LoadConfigDto,
    PathMappingDto, PluginCapabilitiesDto, PluginContentProviderCapabilityDto, PluginDto,
    PluginDtoState, PluginExtractor, PluginImporterCapabilityDto, PluginLoader,
    PluginLoaderRegistry, PluginManifestDto, PluginMetadataDto, PluginProvider,
    PluginProviderFactory, PluginRegistry, PluginSettingsStorage, PluginSourceStorage,
    PluginStateReasonDto, PluginStorage, PluginUpdaterCapabilityDto, PluginVerificationStorage,
    PluginsFeature, ProviderHandlersDto, RuntimeConfigDto, UpdaterCapabilityMetadataDto,
    clean_plugin_archives, download_to_temp_file, record_plugin_verification,
};
pub(crate) use app::{
    CheckForPluginUpdatesUseCase, CheckForPluginUpdatesUseCasePort, DisablePluginUseCase,
    EditPluginSettingsUseCase, EditPluginSettingsUseCasePort, EnablePluginUseCase,
    EnablePluginUseCasePort, ForceEnablePluginUseCase, ForceEnablePluginUseCasePort,
    GetPluginApiVersionUseCase, GetPluginApiVersionUseCasePort, GetPluginDtoUseCase,
    GetPluginDtoUseCasePort, GetPluginReadmeUseCase, GetPluginReadmeUseCasePort,
    GetPluginSettingsUseCase, GetPluginSettingsUseCasePort, GrantPluginFolderAccessUseCase,
    GrantPluginFolderAccessUseCasePort, GrantPluginHostAccessUseCase,
    GrantPluginHostAccessUseCasePort, ImportPluginsUseCase, ImportPluginsUseCasePort,
    InstallPluginFromProviderUseCase, InstallPluginFromProviderUseCasePort, ListPluginsDtoUseCase,
    ListPluginsDtoUseCasePort, ListSkippedPluginDirsUseCase, ListSkippedPluginDirsUseCasePort,
    PluginDisableService, PluginSyncService, PreviewPluginReleaseUseCase,
    PreviewPluginReleaseUseCasePort, ReleaseArchiveCache, ReloadPluginUseCase,
    ReloadPluginUseCasePort, RemovePluginUseCase, RemovePluginUseCasePort, SyncPluginsUseCase,
    UpdatePluginUseCase, UpdatePluginUseCasePort,
};
