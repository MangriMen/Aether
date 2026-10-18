use aether_core::features::plugins::{
    PluginCapabilities, PluginContentProviderCapability, PluginImporterCapability,
    PluginUpdaterCapability, ProviderHandlers,
};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::instance::infra::{
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

    /// Optional function to check if a specific content item is compatible with current environment.
    pub check_compatibility: Option<String>,

    /// Optional function resolving a reference to the provider's platform to one file.
    pub resolve_file: Option<String>,

    /// Optional function describing everything to install for a piece of content.
    pub resolve_install_plan: Option<String>,

    /// Optional function describing a pack without resolving its files.
    pub resolve_pack_metadata: Option<String>,

    /// Optional function describing projects to show installed content by.
    pub get_contents_metadata: Option<String>,

    /// Optional function giving the version numbers of installed files.
    pub get_versions_metadata: Option<String>,

    /// Optional function naming the authors of projects apart from their metadata.
    pub get_authors: Option<String>,
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
            check_compatibility: m.check_compatibility,
            resolve_file: m.resolve_file,
            resolve_install_plan: m.resolve_install_plan,
            resolve_pack_metadata: m.resolve_pack_metadata,
            get_contents_metadata: m.get_contents_metadata,
            get_versions_metadata: m.get_versions_metadata,
            get_authors: m.get_authors,
        }
    }
}
