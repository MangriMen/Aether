mod models;
mod plugin_handle;
mod plugin_instance;

#[cfg(test)]
pub(crate) use models::classify_host_function;
pub use models::{
    AllowedHostChange, AllowedHostEntryRejection, AllowedHostRejection, AllowedPath,
    AllowedPathInput, AllowedPathRejection, ApiConfig, ArchiveMetadata, CapabilityChange,
    CapabilityKind, CommandApproval, CommandApprovalAnswer, CommandApprovalRefusal,
    CommandApprovalRequest, CommandGrant, CommandIdentity, CommandJar, CommandLine, Compatibility,
    ExtractedPlugin, FolderMounts, HostFunctionChange, HostFunctionRisk, IgnoredAllowedPath,
    InvalidLocalHost, JarLaunch, LoadConfig, LoadConfigType, LocalHost, ManifestError,
    ManifestFormatError, OnceOnlyReason, PathMapping, PendingCommandApproval, Plugin,
    PluginCallError, PluginCapabilities, PluginCheckCompatibilityParams, PluginCommandError,
    PluginContent, PluginContentProviderCapability, PluginDownloadError, PluginError,
    PluginHttpError, PluginImporterCapability, PluginInternalEvent, PluginLoadFailure,
    PluginManifest, PluginManifestPreview, PluginMetadata, PluginProviderError, PluginProviderInfo,
    PluginReleaseListing, PluginReleasePreview, PluginSettings, PluginSource, PluginSourceType,
    PluginState, PluginStateReason, PluginUpdateDiff, PluginUpdatePreview, PluginUpdaterCapability,
    PluginVerification, ProviderFailure, ProviderHandlers, ProviderPluginPreview,
    ProviderReleaseInfo, ProviderUpdateInfo, RuntimeConfig, SkippedPluginDir,
    SkippedPluginDirReason, host_function_risks, is_wide_allowed_host, manifest_has_class_path,
    parse_jar_launch,
};
pub(crate) use models::{
    AsCapabilityMetadata, CoreMount, GrantableHost, PLUGIN_API_VERSION, PLUGIN_CACHE_MOUNT,
    PluginHosts, PluginPath, core_folders, effective_allowed_hosts, is_blocked_ip,
    is_filesystem_root, is_host_allowed, partition_mountable, to_wasi_path,
    validate_allowed_host_entry,
};
pub use plugin_handle::PluginHandle;
pub use plugin_instance::PluginInstance;
