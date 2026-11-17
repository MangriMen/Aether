mod app;
mod domain;
pub mod infra;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
pub(crate) use domain::classify_host_function;
pub use domain::{
    AllowedHostEntryRejection, AllowedHostRejection, AllowedPath, AllowedPathInput,
    AllowedPathRejection, ApiConfig, CommandApproval, CommandApprovalAnswer,
    CommandApprovalRefusal, CommandApprovalRequest, CommandGrant, CommandIdentity, CommandJar,
    CommandLine, HostFunctionRisk, IgnoredAllowedPath, InvalidLocalHost, JarLaunch, LoadConfig,
    LoadConfigType, LocalHost, MAX_APPROVED_JAR_BYTES, ManifestError, ManifestFormatError,
    OnceOnlyReason, PathMapping, PendingCommandApproval, PluginCallError, PluginCapabilities,
    PluginCommandError, PluginContentProviderCapability, PluginDownloadError, PluginError,
    PluginHttpError, PluginImporterCapability, PluginLoadFailure, PluginManifest,
    PluginManifestPreview, PluginMetadata, PluginProviderError, PluginProviderInfo,
    PluginReleaseListing, PluginReleasePreview, PluginSettings, PluginSource, PluginSourceType,
    PluginUpdaterCapability, ProviderFailure, ProviderHandlers, ProviderPluginPreview,
    ProviderReleaseInfo, ProviderUpdateInfo, RuntimeConfig, SkippedPluginDir,
    SkippedPluginDirReason, host_function_risks, is_wide_allowed_host, manifest_has_class_path,
    parse_jar_launch,
};
pub(crate) use domain::{
    ArchiveMetadata, AsCapabilityMetadata, Compatibility, CoreMount, ExtractedPlugin, FolderMounts,
    GrantableHost, PLUGIN_API_VERSION, PLUGIN_CACHE_MOUNT, Plugin, PluginCheckCompatibilityParams,
    PluginContent, PluginHandle, PluginHosts, PluginInstance, PluginInternalEvent, PluginPath,
    PluginState, PluginStateReason, PluginVerification, core_folders, effective_allowed_hosts,
    is_blocked_ip, is_filesystem_root, is_host_allowed, partition_mountable, to_wasi_path,
    validate_allowed_host_entry,
};

pub use app::{
    ApiConfigDto, CapabilityMetadataDto, CommandApprover, ContentProviderCapabilityMetadataDto,
    DownloadedArchive, EditPluginSettings, FolderAccess, HostAccess, ImporterCapabilityMetadataDto,
    LoadConfigDto, PathMappingDto, PendingCommandApprovals, PluginCapabilitiesDto,
    PluginContentProviderCapabilityDto, PluginDto, PluginDtoState, PluginExtractor,
    PluginImporterCapabilityDto, PluginLoader, PluginLoaderRegistry, PluginManifestDto,
    PluginMetadataDto, PluginProvider, PluginProviderFactory, PluginRegistry,
    PluginSettingsStorage, PluginSourceStorage, PluginStateReasonDto, PluginStorage,
    PluginUpdaterCapabilityDto, PluginVerificationStorage, PluginsFeature, ProviderHandlersDto,
    RuntimeConfigDto, UpdaterCapabilityMetadataDto, clean_plugin_archives, download_to_temp_file,
    record_plugin_verification,
};
pub(crate) use app::{
    CheckForPluginUpdatesUseCase, CheckForPluginUpdatesUseCasePort, DisablePluginUseCase,
    EditPluginSettingsUseCase, EditPluginSettingsUseCasePort, EnablePluginUseCase,
    EnablePluginUseCasePort, ForceEnablePluginUseCase, ForceEnablePluginUseCasePort,
    GetPluginApiVersionUseCase, GetPluginApiVersionUseCasePort, GetPluginDtoUseCase,
    GetPluginDtoUseCasePort, GetPluginReadmeUseCase, GetPluginReadmeUseCasePort,
    GetPluginSettingsUseCase, GetPluginSettingsUseCasePort, GrantPluginFolderAccessUseCase,
    GrantPluginFolderAccessUseCasePort, GrantPluginHostAccessUseCase,
    GrantPluginHostAccessUseCasePort, GrantPluginLocalHostAccessUseCase,
    GrantPluginLocalHostAccessUseCasePort, ImportPluginsUseCase, ImportPluginsUseCasePort,
    InstallPluginFromProviderUseCase, InstallPluginFromProviderUseCasePort,
    ListPluginReleasesUseCase, ListPluginReleasesUseCasePort, ListPluginsDtoUseCase,
    ListPluginsDtoUseCasePort, ListSkippedPluginDirsUseCase, ListSkippedPluginDirsUseCasePort,
    PluginDisableService, PluginSyncService, PreviewPluginReleaseUseCase,
    PreviewPluginReleaseUseCasePort, ReleaseArchiveCache, ReloadPluginUseCase,
    ReloadPluginUseCasePort, RemovePluginUseCase, RemovePluginUseCasePort, SyncPluginsUseCase,
    UpdatePluginUseCase, UpdatePluginUseCasePort,
};
