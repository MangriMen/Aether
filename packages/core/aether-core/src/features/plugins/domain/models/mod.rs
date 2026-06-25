mod constants;
mod extracted_plugin;
mod manifest_error;
mod plugin;
mod plugin_capabilities;
mod plugin_dtos;
mod plugin_error;
mod plugin_internal_event;
mod plugin_manifest;
mod plugin_provider;
mod plugin_settings;
mod plugin_source;
mod serializable_output;

pub use constants::PLUGIN_API_VERSION;
pub use extracted_plugin::{ExtractedPlugin, PluginContent};
pub use manifest_error::ManifestError;
pub use plugin::{Plugin, PluginState};
pub use plugin_capabilities::{
    PluginCapabilities, PluginContentProviderCapability, PluginImporterCapability,
    PluginUpdaterCapability, ProviderHandlers,
};
pub use plugin_dtos::{PluginCheckCompatibilityParams, PluginImportInstance};
pub use plugin_error::PluginError;
pub use plugin_internal_event::PluginInternalEvent;
pub use plugin_manifest::{
    ApiConfig, Compatibility, LoadConfig, LoadConfigType, PathMapping, PluginManifest,
    PluginMetadata, RuntimeConfig,
};
pub use plugin_provider::{
    PluginSourceType, ProviderPluginPreview, ProviderReleaseInfo, ProviderUpdateInfo,
};
pub use plugin_settings::PluginSettings;
pub use plugin_source::{
    GitHubPluginPreview, GitHubReleaseInfo, PluginManifestPreview, PluginSource, PluginUpdateInfo,
};
pub use serializable_output::SerializableOutput;
