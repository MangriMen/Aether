use aether_core::features::plugins::{
    PluginCapabilities, PluginContentProviderCapability, PluginImporterCapability,
    PluginUpdaterCapability, ProviderHandlers,
};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::instance::{
    ContentProviderCapabilityMetadataDto, ImporterCapabilityMetadataDto,
    UpdaterCapabilityMetadataDto,
};

/// Describes the declarative capabilities of a plugin, such as supported importers.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PluginCapabilitiesDto {
    /// List of supported modpack importers provided by the plugin.
    #[serde(default)]
    pub importers: Vec<PluginImporterCapabilityDto>,

    /// List of supported modpack updaters provided by the plugin.
    #[serde(default)]
    pub updaters: Vec<PluginUpdaterCapabilityDto>,

    /// List of content providers provided by the plugin.
    #[serde(default)]
    pub content_providers: Vec<PluginContentProviderCapabilityDto>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PluginImporterCapabilityDto {
    #[serde(flatten)]
    pub metadata: ImporterCapabilityMetadataDto,

    /// Plugin function name to handle this capability call.
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PluginUpdaterCapabilityDto {
    #[serde(flatten)]
    pub metadata: UpdaterCapabilityMetadataDto,

    /// Plugin function name to handle this capability call.
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PluginContentProviderCapabilityDto {
    #[serde(flatten)]
    pub metadata: ContentProviderCapabilityMetadataDto,

    /// Set of functions provided by the plugin to handle content operations.
    pub handlers: ProviderHandlersDto,
}

/// Names of plugin functions that implement specific content provider logic.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProviderHandlersDto {
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

impl From<PluginCapabilities> for PluginCapabilitiesDto {
    fn from(m: PluginCapabilities) -> Self {
        Self {
            importers: m.importers.into_iter().map(Into::into).collect(),
            updaters: m.updaters.into_iter().map(Into::into).collect(),
            content_providers: m.content_providers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PluginImporterCapability> for PluginImporterCapabilityDto {
    fn from(m: PluginImporterCapability) -> Self {
        Self {
            metadata: m.metadata.into(),
            handler: m.handler,
        }
    }
}

impl From<PluginUpdaterCapability> for PluginUpdaterCapabilityDto {
    fn from(m: PluginUpdaterCapability) -> Self {
        Self {
            metadata: m.metadata.into(),
            handler: m.handler,
        }
    }
}

impl From<PluginContentProviderCapability> for PluginContentProviderCapabilityDto {
    fn from(m: PluginContentProviderCapability) -> Self {
        Self {
            metadata: m.metadata.into(),
            handlers: m.handlers.into(),
        }
    }
}

impl From<ProviderHandlers> for ProviderHandlersDto {
    fn from(m: ProviderHandlers) -> Self {
        Self {
            search: m.search,
            get_content: m.get_content,
            list_version: m.list_version,
            install_atomic: m.install_atomic,
            install_modpack: m.install_modpack,
            check_compatibility: m.check_compatibility,
        }
    }
}
