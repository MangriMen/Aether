use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::v0::{
    ContentProviderCapabilityMetadataDto, ContentSourceCapabilityMetadataDto,
    ImporterCapabilityMetadataDto, UpdaterCapabilityMetadataDto,
};

/// Describes the declarative capabilities of a plugin.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct PluginCapabilitiesDto {
    /// Optional URI pointing to the JSON Schema for this capabilities file.
    /// Ignored during parsing — reserved for editor tooling and validation.
    #[serde(rename = "$schema", default, skip_serializing)]
    #[schemars(skip)]
    pub dollar_schema: Option<String>,

    /// List of supported modpack importers provided by the plugin.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub importers: Vec<PluginImporterCapabilityDto>,

    /// List of supported modpack updaters provided by the plugin.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub updaters: Vec<PluginUpdaterCapabilityDto>,

    /// List of content providers provided by the plugin.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content_providers: Vec<PluginContentProviderCapabilityDto>,

    /// List of content sources provided by the plugin (new unified capability).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content_sources: Vec<PluginContentSourceCapabilityDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct PluginImporterCapabilityDto {
    #[serde(flatten)]
    pub metadata: ImporterCapabilityMetadataDto,

    /// Plugin function name to handle this capability call.
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct PluginUpdaterCapabilityDto {
    #[serde(flatten)]
    pub metadata: UpdaterCapabilityMetadataDto,

    /// Plugin function name to handle this capability call.
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct PluginContentProviderCapabilityDto {
    #[serde(flatten)]
    pub metadata: ContentProviderCapabilityMetadataDto,

    /// Set of functions provided by the plugin to handle content operations.
    pub handlers: ProviderHandlersDto,
}

/// Names of plugin functions that implement specific content provider logic.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProviderHandlersDto {
    pub search: String,
    pub get_content: String,
    pub list_version: Option<String>,
    pub install_atomic: String,
    pub install_modpack: Option<String>,
    pub check_compatibility: Option<String>,
}

/// New unified content source capability (replaces `ContentProvider`, `Importer`, `Updater`).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct PluginContentSourceCapabilityDto {
    #[serde(flatten)]
    pub metadata: ContentSourceCapabilityMetadataDto,

    /// Set of functions provided by the plugin to handle content source operations.
    pub handlers: ContentSourceHandlersDto,
}

/// Names of plugin functions that implement specific content source logic.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContentSourceHandlersDto {
    pub search: String,
    pub get_content: String,
    pub list_version: Option<String>,
    pub get_version_info: String,
    pub check_compatibility: Option<String>,
}
