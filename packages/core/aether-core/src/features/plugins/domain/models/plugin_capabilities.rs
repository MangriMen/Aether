use std::ops::Deref;

use crate::features::{
    instance::{
        CapabilityMetadata, ContentProviderCapabilityMetadata, ContentSourceCapabilityMetadata,
        ImporterCapabilityMetadata, UpdaterCapabilityMetadata,
    },
    plugins::AsCapabilityMetadata,
};

/// Describes the declarative capabilities of a plugin.
#[derive(Debug, Clone, Default)]
pub struct PluginCapabilities {
    pub importers: Vec<PluginImporterCapability>,
    pub updaters: Vec<PluginUpdaterCapability>,
    pub content_providers: Vec<PluginContentProviderCapability>,
    pub content_sources: Vec<PluginContentSourceCapability>,
}

#[derive(Debug, Clone)]
pub struct PluginImporterCapability {
    pub metadata: ImporterCapabilityMetadata,
    pub handler: String,
}

#[derive(Debug, Clone)]
pub struct PluginUpdaterCapability {
    pub metadata: UpdaterCapabilityMetadata,
    pub handler: String,
}

#[derive(Debug, Clone)]
pub struct PluginContentProviderCapability {
    pub metadata: ContentProviderCapabilityMetadata,
    pub handlers: ProviderHandlers,
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

/// New unified capability — replaces `ContentProvider`, `Importer`, and `Updater`.
#[derive(Debug, Clone)]
pub struct PluginContentSourceCapability {
    pub metadata: ContentSourceCapabilityMetadata,
    pub handlers: ContentSourceHandlers,
}

#[derive(Debug, Clone)]
pub struct ContentSourceHandlers {
    pub search: String,
    pub get_content: String,
    pub list_version: Option<String>,
    pub get_version_info: String,
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

impl Deref for PluginContentSourceCapability {
    type Target = ContentSourceCapabilityMetadata;
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

impl AsCapabilityMetadata for PluginContentSourceCapability {
    fn as_metadata(&self) -> &CapabilityMetadata {
        &self.metadata.base
    }
}
