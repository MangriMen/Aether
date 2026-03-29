use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityMetadata {
    /// Identifier (lowercase, kebab/underscore allowed).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Optional icon file name or URL.
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImporterCapabilityMetadata {
    #[serde(flatten)]
    pub base: CapabilityMetadata,

    /// Optional field label for the importer.
    pub field_label: Option<String>,

    pub supported_extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdaterCapabilityMetadata {
    #[serde(flatten)]
    pub base: CapabilityMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentProviderCapabilityMetadata {
    #[serde(flatten)]
    pub base: CapabilityMetadata,

    pub supports_install_atomic: bool,
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
