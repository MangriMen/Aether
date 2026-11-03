use std::ops::Deref;

use crate::features::{
    instance::{
        CapabilityMetadata, ContentProviderCapabilityMetadata, ImporterCapabilityMetadata,
        UpdaterCapabilityMetadata,
    },
    plugins::ManifestError,
};

pub(crate) trait AsCapabilityMetadata {
    fn as_metadata(&self) -> &CapabilityMetadata;
}

/// Describes the declarative capabilities of a plugin.
#[derive(Debug, Clone, Default)]
pub struct PluginCapabilities {
    pub importers: Vec<PluginImporterCapability>,
    pub updaters: Vec<PluginUpdaterCapability>,
    pub content_providers: Vec<PluginContentProviderCapability>,
}

impl PluginCapabilities {
    /// Refuse capabilities that cannot work together.
    pub fn validate(&self) -> Result<(), ManifestError> {
        // A platform makes the provider a candidate for every `ProviderRef` to it, resolved
        // through `resolve_file`: without it every ref would fail instead of the plugin's load.
        if let Some(provider) = self
            .content_providers
            .iter()
            .find(|c| c.metadata.platform.is_some() && c.handlers.resolve_file.is_none())
        {
            return Err(ManifestError::PlatformWithoutResolveFile {
                capability_id: provider.metadata.base.id.clone(),
            });
        }

        Ok(())
    }
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

/// Names of the plugin functions behind a content provider. An operation without a handler is
/// unsupported: the proxy answers it with `InstanceError::UnsupportedOperation`.
#[derive(Debug, Clone)]
pub struct ProviderHandlers {
    pub search: String,
    pub get_content: String,
    pub list_version: Option<String>,
    pub check_compatibility: Option<String>,
    pub resolve_file: Option<String>,
    pub resolve_install_plan: Option<String>,
    pub resolve_pack_metadata: Option<String>,
    pub get_contents_metadata: Option<String>,
    pub get_versions_metadata: Option<String>,
    pub get_authors: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::instance::{CapabilityId, PlatformId};

    fn base(id: &'static str) -> CapabilityMetadata {
        CapabilityMetadata {
            id: CapabilityId::from_static(id),
            name: "Test".to_owned(),
            description: None,
            icon: None,
        }
    }

    fn importer(id: &'static str) -> PluginImporterCapability {
        PluginImporterCapability {
            metadata: ImporterCapabilityMetadata {
                base: base(id),
                field_label: None,
                supported_extensions: vec![],
            },
            handler: "import".to_owned(),
        }
    }

    fn updater(id: &'static str) -> PluginUpdaterCapability {
        PluginUpdaterCapability {
            metadata: UpdaterCapabilityMetadata { base: base(id) },
            handler: "update".to_owned(),
        }
    }

    fn content_provider(
        id: &'static str,
        platform: Option<&'static str>,
    ) -> PluginContentProviderCapability {
        PluginContentProviderCapability {
            metadata: ContentProviderCapabilityMetadata {
                base: base(id),
                platform: platform.map(PlatformId::from_static),
            },
            handlers: ProviderHandlers {
                search: "search".to_owned(),
                get_content: "get_content".to_owned(),
                list_version: None,
                check_compatibility: None,
                resolve_file: Some("resolve_file".to_owned()),
                resolve_install_plan: None,
                resolve_pack_metadata: None,
                get_contents_metadata: None,
                get_versions_metadata: None,
                get_authors: None,
            },
        }
    }

    /// The ids `packwiz` and `curseforge` plugins declare in their `capabilities.json`.
    #[test]
    fn should_accept_capabilities_when_they_are_those_of_existing_plugins() {
        let capabilities = PluginCapabilities {
            importers: vec![importer("packwiz")],
            updaters: vec![updater("packwiz")],
            content_providers: vec![content_provider("curseforge", Some("curseforge"))],
        };

        assert!(capabilities.validate().is_ok());
    }

    #[test]
    fn should_require_resolve_file_when_a_provider_has_a_platform() {
        let mut provider = content_provider("curseforge", Some("curseforge"));
        provider.handlers.resolve_file = None;
        let capabilities = PluginCapabilities {
            content_providers: vec![provider],
            ..Default::default()
        };

        assert!(matches!(
            capabilities.validate(),
            Err(ManifestError::PlatformWithoutResolveFile { capability_id })
                if capability_id.as_str() == "curseforge"
        ));
    }

    #[test]
    fn should_accept_a_provider_when_it_has_no_platform_and_no_resolve_file() {
        let mut provider = content_provider("search-only", None);
        provider.handlers.resolve_file = None;
        let capabilities = PluginCapabilities {
            content_providers: vec![provider],
            ..Default::default()
        };

        assert!(capabilities.validate().is_ok());
    }
}
