use std::ops::Deref;

use register_schema::RegisterSchema;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::features::{
    instance::{
        CapabilityMetadata, ContentProviderCapabilityMetadata, ImporterCapabilityMetadata,
        UpdaterCapabilityMetadata,
    },
    plugins::AsCapabilityMetadata,
};

/// Describes the declarative capabilities of a plugin, such as supported importers.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, RegisterSchema)]
#[schema_category("plugin_api")]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct PluginCapabilities {
    /// List of supported modpack importers provided by the plugin.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub importers: Vec<PluginImporterCapability>,

    /// List of supported modpack updaters provided by the plugin.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub updaters: Vec<PluginUpdaterCapability>,

    /// List of content providers provided by the plugin.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content_providers: Vec<PluginContentProviderCapability>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct PluginImporterCapability {
    #[serde(flatten)]
    pub metadata: ImporterCapabilityMetadata,

    /// Plugin function name to handle this capability call.
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct PluginUpdaterCapability {
    #[serde(flatten)]
    pub metadata: UpdaterCapabilityMetadata,

    /// Plugin function name to handle this capability call.
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct PluginContentProviderCapability {
    #[serde(flatten)]
    pub metadata: ContentProviderCapabilityMetadata,

    /// Set of functions provided by the plugin to handle content operations.
    pub handlers: ProviderHandlers,
}

/// Names of plugin functions that implement specific content provider logic.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProviderHandlers {
    /// Function to search for content within the provider.
    pub search: String,

    /// Function to get detailed information about a specific piece of content.
    pub get_content: String,

    /// Optional function to list available versions of a specific content item.
    pub list_version: Option<String>,

    /// Function to handle the download and installation of a single item.
    pub install_atomic: String,

    /// Optional function to handle the installation of full modpacks.
    pub install_modpack: Option<String>,

    /// Optional function to check if a specific content item is compatible with current environment.
    pub check_compatibility: Option<String>,
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
