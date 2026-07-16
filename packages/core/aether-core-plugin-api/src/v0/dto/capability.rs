use register_schema::RegisterSchema;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, RegisterSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
#[schema_category("plugin_api")]
#[schema_name("capability_metadata")]
pub struct CapabilityMetadataDto {
    /// Identifier for the capability (lowercase, kebab/underscore allowed).
    #[schemars(regex(pattern = r"^[a-z0-9_\-]+$"))]
    pub id: String,

    /// Display name of the capability.
    pub name: String,

    /// Optional detailed description of what this capability does.
    pub description: Option<String>,

    /// Optional icon file name or URL for the UI.
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, RegisterSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
#[schema_category("plugin_api")]
#[schema_name("content_provider_capability_metadata")]
pub struct ContentProviderCapabilityMetadataDto {
    #[serde(flatten)]
    pub base: CapabilityMetadataDto,

    /// Whether the provider supports installing individual items.
    pub supports_install_atomic: bool,

    /// Whether the provider supports installing complex modpacks or curated collections.
    pub supports_install_modpacks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, RegisterSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
#[schema_category("plugin_api")]
#[schema_name("pack_manager_capability_metadata")]
pub struct PackManagerCapabilityMetadataDto {
    #[serde(flatten)]
    pub base: CapabilityMetadataDto,

    /// Whether this manager supports installing packs.
    pub supports_install: bool,

    /// Whether this manager supports updating packs.
    pub supports_update: bool,

    /// Whether this manager supports checking for updates.
    pub supports_check_updates: bool,

    /// Optional field label shown in the pack manager UI.
    pub field_label: Option<String>,

    /// List of supported file extensions, e.g., [`zip`, `mrpack`].
    pub supported_extensions: Vec<String>,
}
