use std::ops::Deref;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct CapabilityMetadata {
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ImporterCapabilityMetadata {
    #[serde(flatten)]
    pub base: CapabilityMetadata,

    ///Optional field label shown in the importer UI.
    pub field_label: Option<String>,

    /// List of supported file extensions, e.g., ["zip", "mrpack"].
    pub supported_extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct UpdaterCapabilityMetadata {
    #[serde(flatten)]
    pub base: CapabilityMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ContentProviderCapabilityMetadata {
    #[serde(flatten)]
    pub base: CapabilityMetadata,

    /// Whether the provider supports installing individual items (e.g., a single mod or resource pack).
    pub supports_install_atomic: bool,

    /// Whether the provider supports installing complex modpacks or curated collections.
    pub supports_install_modpacks: bool,
}

impl Deref for ImporterCapabilityMetadata {
    type Target = CapabilityMetadata;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl Deref for UpdaterCapabilityMetadata {
    type Target = CapabilityMetadata;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl Deref for ContentProviderCapabilityMetadata {
    type Target = CapabilityMetadata;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
