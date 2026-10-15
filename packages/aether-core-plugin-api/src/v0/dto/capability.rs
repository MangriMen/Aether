use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct CapabilityMetadataDto {
    /// Identifier for the capability (lowercase, kebab/underscore allowed).
    #[schemars(regex(pattern = r"^[a-z0-9][a-z0-9_\-]*$"))]
    pub id: String,

    /// Display name of the capability.
    pub name: String,

    /// Optional detailed description of what this capability does.
    pub description: Option<String>,

    /// Optional icon file name or URL for the UI.
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ImporterCapabilityMetadataDto {
    #[serde(flatten)]
    pub base: CapabilityMetadataDto,

    /// Optional field label shown in the importer UI.
    pub field_label: Option<String>,

    /// List of supported file extensions, e.g., [`zip`, `mrpack`].
    pub supported_extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct UpdaterCapabilityMetadataDto {
    #[serde(flatten)]
    pub base: CapabilityMetadataDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ContentProviderCapabilityMetadataDto {
    #[serde(flatten)]
    pub base: CapabilityMetadataDto,

    /// The platform whose content this provider serves (`modrinth`, `curseforge`): what an
    /// install plan's `providerRef` names to point at content without naming a plugin. Several
    /// providers may serve one platform; the launcher picks which one is asked.
    ///
    /// A provider declaring a platform must implement `resolveFile`. Leave it out when the
    /// provider hosts no content of its own and cannot be the target of a `providerRef`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(regex(pattern = r"^[a-z0-9][a-z0-9_\-]*$"))]
    pub platform: Option<String>,
}
