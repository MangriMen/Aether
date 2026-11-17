mod allowed_host;
mod allowed_path;
mod command_approval;
mod constants;
mod extracted_plugin;
mod host_function_risk;
mod manifest_error;
mod plugin;
mod plugin_call_error;
mod plugin_capabilities;
mod plugin_dtos;
mod plugin_error;
mod plugin_file_path;
mod plugin_internal_event;
mod plugin_load_failure;
mod plugin_manifest;
mod plugin_path;
mod plugin_provider;
mod plugin_provider_error;
mod plugin_settings;
mod plugin_source;
mod plugin_verification;
mod skipped_plugin_dir;

pub use allowed_host::{AllowedHostEntryRejection, AllowedHostRejection, is_wide_allowed_host};
pub(crate) use allowed_host::{
    GrantableHost, effective_allowed_hosts, is_blocked_ip, is_host_allowed,
    validate_allowed_host_entry,
};
pub use allowed_path::{
    AllowedPath, AllowedPathInput, AllowedPathRejection, FolderMounts, IgnoredAllowedPath,
};
pub(crate) use allowed_path::{
    CoreMount, PLUGIN_CACHE_MOUNT, core_folders, is_filesystem_root, partition_mountable,
    to_wasi_path,
};
pub use command_approval::{
    CommandApproval, CommandApprovalAnswer, CommandApprovalRefusal, CommandApprovalRequest,
    CommandGrant, CommandIdentity, CommandJar, JarLaunch, MAX_APPROVED_JAR_BYTES, OnceOnlyReason,
    PendingCommandApproval, manifest_has_class_path, parse_jar_launch,
};
pub(crate) use constants::PLUGIN_API_VERSION;
pub use extracted_plugin::{ArchiveMetadata, ExtractedPlugin, PluginContent};
#[cfg(test)]
pub(crate) use host_function_risk::classify_host_function;
pub use host_function_risk::{HostFunctionRisk, host_function_risks};
pub use manifest_error::ManifestError;
pub use plugin::{Plugin, PluginState, PluginStateReason};
pub use plugin_call_error::PluginCallError;
pub(crate) use plugin_capabilities::AsCapabilityMetadata;
pub use plugin_capabilities::{
    PluginCapabilities, PluginContentProviderCapability, PluginImporterCapability,
    PluginUpdaterCapability, ProviderHandlers,
};
pub use plugin_dtos::PluginCheckCompatibilityParams;
pub use plugin_error::PluginError;
pub use plugin_file_path::PluginFilePath;
pub use plugin_internal_event::PluginInternalEvent;
pub use plugin_load_failure::PluginLoadFailure;
pub use plugin_manifest::{
    ApiConfig, Compatibility, LoadConfig, LoadConfigType, PathMapping, PluginManifest,
    PluginMetadata, RuntimeConfig,
};
pub(crate) use plugin_path::PluginPath;
pub use plugin_provider::{
    PluginProviderInfo, PluginReleaseListing, PluginReleasePreview, PluginSourceType,
    ProviderPluginPreview, ProviderReleaseInfo, ProviderUpdateInfo,
};
pub use plugin_provider_error::{ManifestFormatError, PluginProviderError, ProviderFailure};
pub use plugin_settings::PluginSettings;
pub use plugin_source::{PluginManifestPreview, PluginSource};
pub use plugin_verification::PluginVerification;
pub use skipped_plugin_dir::{SkippedPluginDir, SkippedPluginDirReason};
