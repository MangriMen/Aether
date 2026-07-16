use std::ops::Deref;

use crate::features::{
    instance::{
        CapabilityMetadata, ContentProviderCapabilityMetadata, PackManagerCapabilityMetadata,
    },
    plugins::AsCapabilityMetadata,
};

/// Describes the declarative capabilities of a plugin.
#[derive(Debug, Clone, Default)]
pub struct PluginCapabilities {
    pub content_providers: Vec<PluginContentProviderCapability>,
    pub pack_managers: Vec<PluginPackManagerCapability>,
}

#[derive(Debug, Clone)]
pub struct PluginContentProviderCapability {
    pub metadata: ContentProviderCapabilityMetadata,
    pub handlers: ProviderHandlers,
}

#[derive(Debug, Clone)]
pub struct PluginPackManagerCapability {
    pub metadata: PackManagerCapabilityMetadata,
    pub handlers: PackManagerHandlers,
}

#[derive(Debug, Clone)]
pub struct PackManagerHandlers {
    pub install: String,
    pub update: Option<String>,
    pub check_updates: Option<String>,
    pub resolve_metadata: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProviderHandlers {
    pub search: String,
    pub get_content: String,
    pub list_version: Option<String>,
    pub install_atomic: String,
    pub install_modpack: Option<String>,
    pub check_compatibility: Option<String>,
}

impl Deref for PluginContentProviderCapability {
    type Target = ContentProviderCapabilityMetadata;
    fn deref(&self) -> &Self::Target {
        &self.metadata
    }
}

impl Deref for PluginPackManagerCapability {
    type Target = PackManagerCapabilityMetadata;
    fn deref(&self) -> &Self::Target {
        &self.metadata
    }
}

impl AsCapabilityMetadata for PluginContentProviderCapability {
    fn as_metadata(&self) -> &CapabilityMetadata {
        &self.metadata.base
    }
}

impl AsCapabilityMetadata for PluginPackManagerCapability {
    fn as_metadata(&self) -> &CapabilityMetadata {
        &self.metadata.base
    }
}
