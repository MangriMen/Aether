use register_schema::RegisterSchema;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::v0::{
    ContentProviderCapabilityMetadataDto, ImporterCapabilityMetadataDto,
    UpdaterCapabilityMetadataDto,
};

/// Describes the declarative capabilities of a plugin, such as supported importers.
// Published as `schemas/plugin-capabilities.schema.json`; see `PluginManifestDto` for why
// the schema names drop the `Dto` suffix and why this note is not a doc comment.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, RegisterSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields, rename = "PluginCapabilities")]
#[schema_category("plugin_api")]
#[schema_name("PluginCapabilities")]
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
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields, rename = "PluginImporterCapability")]
pub struct PluginImporterCapabilityDto {
    #[serde(flatten)]
    pub metadata: ImporterCapabilityMetadataDto,

    /// Plugin function name to handle this capability call.
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields, rename = "PluginUpdaterCapability")]
pub struct PluginUpdaterCapabilityDto {
    #[serde(flatten)]
    pub metadata: UpdaterCapabilityMetadataDto,

    /// Plugin function name to handle this capability call.
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields, rename = "PluginContentProviderCapability")]
pub struct PluginContentProviderCapabilityDto {
    #[serde(flatten)]
    pub metadata: ContentProviderCapabilityMetadataDto,

    /// Set of functions provided by the plugin to handle content operations.
    pub handlers: ProviderHandlersDto,
}

/// Names of plugin functions that implement specific content provider logic.
///
/// Which of the optional operations a provider supports follows from which handlers it names:
/// an operation without a handler is reported as unsupported.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "ProviderHandlers")]
pub struct ProviderHandlersDto {
    /// Function to search for content within the provider.
    pub search: String,
    /// Function to get detailed information about a specific piece of content.
    pub get_content: String,
    /// Optional function to list available versions of a specific content item.
    pub list_version: Option<String>,
    /// Optional function to check if a specific content item is compatible with current environment.
    pub check_compatibility: Option<String>,
    /// Optional function telling which file a `providerRef` of the provider's `platform` stands
    /// for. Takes `ResolveFileParamsDto`, returns `ResolvedFileDto`. Required with `platform`.
    pub resolve_file: Option<String>,
    /// Optional function describing everything to install for a piece of content, a single mod
    /// and a whole modpack alike. Takes `ResolveInstallPlanParamsDto`, returns `InstallPlanDto`.
    pub resolve_install_plan: Option<String>,
    /// Optional function describing a pack without resolving its files: the preview before
    /// installing and the check for updates. Takes `PackSourceDto`, returns `PackMetadataDto`.
    pub resolve_pack_metadata: Option<String>,
    /// Optional function describing projects to show installed content by (name, icon, author).
    /// Takes a list of project ids, returns a list of `ContentMetadataDto`. Without it the
    /// provider's content is shown by file alone.
    pub get_contents_metadata: Option<String>,
    /// Optional function giving the version number of installed files. Takes a list of version
    /// ids, returns a list of `ContentVersionMetadataDto`. Without it the files are shown
    /// without a version.
    pub get_versions_metadata: Option<String>,
}
