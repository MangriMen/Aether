use aether_core::features::plugins::{
    PackManagerHandlers, PluginCapabilities, PluginContentProviderCapability,
    PluginPackManagerCapability, ProviderHandlers,
};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::instance::infra::{
    ContentProviderCapabilityMetadataDto, PackManagerCapabilityMetadataDto,
};

/// Describes the declarative capabilities of a plugin, such as supported content providers and pack managers.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PluginCapabilitiesDto {
    /// List of content providers provided by the plugin.
    #[serde(default)]
    pub content_providers: Vec<PluginContentProviderCapabilityDto>,

    /// List of pack managers provided by the plugin.
    #[serde(default)]
    pub pack_managers: Vec<PluginPackManagerCapabilityDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PluginContentProviderCapabilityDto {
    #[serde(flatten)]
    pub metadata: ContentProviderCapabilityMetadataDto,

    /// Set of functions provided by the plugin to handle content operations.
    pub handlers: ProviderHandlersDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PluginPackManagerCapabilityDto {
    #[serde(flatten)]
    pub metadata: PackManagerCapabilityMetadataDto,

    /// Set of functions provided by the plugin to handle pack operations.
    pub handlers: PackManagerHandlersDto,
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

/// Names of plugin functions that implement specific pack manager logic.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PackManagerHandlersDto {
    /// Function to install a pack into an already-created instance.
    pub install: String,

    /// Optional function to update an existing pack-managed instance.
    pub update: Option<String>,

    /// Optional function to check whether a newer version of the pack is available.
    pub check_updates: Option<String>,

    /// Optional function to resolve pack metadata (name, game version, loader)
    /// from a pack source before instance creation.
    pub resolve_metadata: Option<String>,
}

impl From<PluginCapabilities> for PluginCapabilitiesDto {
    fn from(m: PluginCapabilities) -> Self {
        Self {
            content_providers: m.content_providers.into_iter().map(Into::into).collect(),
            pack_managers: m.pack_managers.into_iter().map(Into::into).collect(),
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

impl From<PluginPackManagerCapability> for PluginPackManagerCapabilityDto {
    fn from(m: PluginPackManagerCapability) -> Self {
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

impl From<PackManagerHandlers> for PackManagerHandlersDto {
    fn from(m: PackManagerHandlers) -> Self {
        Self {
            install: m.install,
            update: m.update,
            check_updates: m.check_updates,
            resolve_metadata: m.resolve_metadata,
        }
    }
}
