use register_schema::RegisterSchema;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::v0::{ContentProviderCapabilityMetadataDto, PackManagerCapabilityMetadataDto};

/// Describes the declarative capabilities of a plugin.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, RegisterSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
#[schema_category("plugin_api")]
#[schema_name("plugin_capabilities")]
pub struct PluginCapabilitiesDto {
    /// Optional URI pointing to the JSON Schema for this capabilities file.
    /// Ignored during parsing — reserved for editor tooling and validation.
    #[serde(rename = "$schema", default, skip_serializing)]
    #[schemars(skip)]
    pub dollar_schema: Option<String>,

    /// List of content providers provided by the plugin.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content_providers: Vec<PluginContentProviderCapabilityDto>,

    /// List of pack managers provided by the plugin.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pack_managers: Vec<PluginPackManagerCapabilityDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, RegisterSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
#[schema_category("plugin_api")]
#[schema_name("plugin_content_provider_capability")]
pub struct PluginContentProviderCapabilityDto {
    #[serde(flatten)]
    pub metadata: ContentProviderCapabilityMetadataDto,

    /// Set of functions provided by the plugin to handle content operations.
    pub handlers: ProviderHandlersDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, RegisterSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
#[schema_category("plugin_api")]
#[schema_name("plugin_pack_manager_capability")]
pub struct PluginPackManagerCapabilityDto {
    #[serde(flatten)]
    pub metadata: PackManagerCapabilityMetadataDto,

    /// Set of functions provided by the plugin to handle pack operations.
    pub handlers: PackManagerHandlersDto,
}

/// Names of plugin functions that implement specific content provider logic.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, RegisterSchema)]
#[serde(rename_all = "camelCase")]
#[schema_category("plugin_api")]
#[schema_name("provider_handlers")]
pub struct ProviderHandlersDto {
    pub search: String,
    pub get_content: String,
    pub list_version: Option<String>,
    pub install_atomic: String,
    pub install_modpack: Option<String>,
    pub check_compatibility: Option<String>,
}

/// Names of plugin functions that implement specific pack manager logic.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, RegisterSchema)]
#[serde(rename_all = "camelCase")]
#[schema_category("plugin_api")]
#[schema_name("pack_manager_handlers")]
pub struct PackManagerHandlersDto {
    pub install: String,
    pub update: Option<String>,
    pub check_updates: Option<String>,
    /// Optional function to resolve pack metadata (name, game version, loader)
    /// from a pack source before instance creation.
    pub resolve_metadata: Option<String>,
}
