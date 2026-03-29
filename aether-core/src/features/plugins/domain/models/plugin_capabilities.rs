use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::features::{
    instance::{
        CapabilityMetadata, ContentProviderCapabilityMetadata, ImporterCapabilityMetadata,
        UpdaterCapabilityMetadata,
    },
    plugins::AsCapabilityMetadata,
};

/// Describes the declarative capabilities of a plugin, such as supported importers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCapabilities {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub importers: Vec<PluginImporterCapability>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub updaters: Vec<PluginUpdaterCapability>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content_providers: Vec<PluginContentProviderCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginImporterCapability {
    #[serde(flatten)]
    pub metadata: ImporterCapabilityMetadata,
    /// Plugin function name to handle this capability call.
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginUpdaterCapability {
    #[serde(flatten)]
    pub metadata: UpdaterCapabilityMetadata,
    /// Plugin function name to handle this capability call.
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginContentProviderCapability {
    #[serde(flatten)]
    pub metadata: ContentProviderCapabilityMetadata,
    /// Plugin function name to handle this capability call.
    pub search_handler: String,
    pub install_atomic_handler: String,
    pub install_modpack_handler: String,
    pub check_compatibility_handler: String,
}

impl Deref for PluginImporterCapability {
    type Target = ImporterCapabilityMetadata;
    fn deref(&self) -> &Self::Target {
        &self.metadata
    }
}

impl Deref for PluginUpdaterCapability {
    type Target = UpdaterCapabilityMetadata;
    fn deref(&self) -> &Self::Target {
        &self.metadata
    }
}

impl Deref for PluginContentProviderCapability {
    type Target = ContentProviderCapabilityMetadata;
    fn deref(&self) -> &Self::Target {
        &self.metadata
    }
}

impl AsCapabilityMetadata for PluginImporterCapability {
    fn as_metadata(&self) -> &CapabilityMetadata {
        &self.metadata.base
    }
}

impl AsCapabilityMetadata for PluginUpdaterCapability {
    fn as_metadata(&self) -> &CapabilityMetadata {
        &self.metadata.base
    }
}

impl AsCapabilityMetadata for PluginContentProviderCapability {
    fn as_metadata(&self) -> &CapabilityMetadata {
        &self.metadata.base
    }
}
