use std::ops::Deref;

use crate::features::{
    instance::{
        CapabilityMetadata, ContentProviderCapabilityMetadata, ImporterCapabilityMetadata,
        UpdaterCapabilityMetadata, is_valid_id,
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
    /// Every capability id, and every platform a content provider serves, must fit the id
    /// scheme: they become parts of `plugin_id#capability_id` and of `ProviderRef`.
    pub fn validate(&self) -> Result<(), ManifestError> {
        let ids = self
            .importers
            .iter()
            .map(|c| ("importers[].id", &c.metadata.base.id))
            .chain(
                self.updaters
                    .iter()
                    .map(|c| ("updaters[].id", &c.metadata.base.id)),
            )
            .chain(
                self.content_providers
                    .iter()
                    .map(|c| ("contentProviders[].id", &c.metadata.base.id)),
            )
            .chain(self.content_providers.iter().filter_map(|c| {
                c.metadata
                    .platform
                    .as_ref()
                    .map(|platform| ("contentProviders[].platform", platform))
            }));

        for (field, value) in ids {
            if !is_valid_id(value) {
                return Err(ManifestError::InvalidId {
                    field: field.to_owned(),
                    value: value.clone(),
                });
            }
        }

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

    fn base(id: &str) -> CapabilityMetadata {
        CapabilityMetadata {
            id: id.to_owned(),
            name: "Test".to_owned(),
            description: None,
            icon: None,
        }
    }

    fn importer(id: &str) -> PluginImporterCapability {
        PluginImporterCapability {
            metadata: ImporterCapabilityMetadata {
                base: base(id),
                field_label: None,
                supported_extensions: vec![],
            },
            handler: "import".to_owned(),
        }
    }

    fn updater(id: &str) -> PluginUpdaterCapability {
        PluginUpdaterCapability {
            metadata: UpdaterCapabilityMetadata { base: base(id) },
            handler: "update".to_owned(),
        }
    }

    fn content_provider(id: &str, platform: Option<&str>) -> PluginContentProviderCapability {
        PluginContentProviderCapability {
            metadata: ContentProviderCapabilityMetadata {
                base: base(id),
                platform: platform.map(ToOwned::to_owned),
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
            },
        }
    }

    fn rejected_field(capabilities: &PluginCapabilities) -> Option<String> {
        match capabilities.validate() {
            Err(ManifestError::InvalidId { field, .. }) => Some(field),
            _ => None,
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
    fn should_reject_capabilities_when_an_id_is_invalid() {
        for id in ["", "a#b", "x:y", "Packwiz"] {
            let importers = PluginCapabilities {
                importers: vec![importer(id)],
                ..Default::default()
            };
            let updaters = PluginCapabilities {
                updaters: vec![updater(id)],
                ..Default::default()
            };
            let providers = PluginCapabilities {
                content_providers: vec![content_provider(id, None)],
                ..Default::default()
            };

            assert_eq!(
                rejected_field(&importers).as_deref(),
                Some("importers[].id")
            );
            assert_eq!(rejected_field(&updaters).as_deref(), Some("updaters[].id"));
            assert_eq!(
                rejected_field(&providers).as_deref(),
                Some("contentProviders[].id")
            );
        }
    }

    #[test]
    fn should_reject_a_capability_when_the_platform_is_invalid() {
        let capabilities = PluginCapabilities {
            content_providers: vec![content_provider("provider", Some("Mod Rinth"))],
            ..Default::default()
        };

        assert_eq!(
            rejected_field(&capabilities).as_deref(),
            Some("contentProviders[].platform")
        );
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
                if capability_id == "curseforge"
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
