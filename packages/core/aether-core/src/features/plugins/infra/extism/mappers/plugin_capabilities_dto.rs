use aether_core_plugin_api::v0::{
    CapabilityMetadataDto, ContentProviderCapabilityMetadataDto,
    ContentSourceCapabilityMetadataDto, ContentSourceHandlersDto, ImporterCapabilityMetadataDto,
    PluginCapabilitiesDto, PluginContentProviderCapabilityDto, PluginContentSourceCapabilityDto,
    PluginImporterCapabilityDto, PluginUpdaterCapabilityDto, ProviderHandlersDto,
    UpdaterCapabilityMetadataDto,
};

use crate::features::{
    instance::{
        CapabilityMetadata, ContentProviderCapabilityMetadata, ContentSourceCapabilityMetadata,
        ImporterCapabilityMetadata, UpdaterCapabilityMetadata,
    },
    plugins::{
        ContentSourceHandlers, PluginCapabilities, PluginContentProviderCapability,
        PluginContentSourceCapability, PluginImporterCapability, PluginUpdaterCapability,
        ProviderHandlers,
    },
};

impl From<PluginCapabilities> for PluginCapabilitiesDto {
    fn from(v: PluginCapabilities) -> Self {
        Self {
            dollar_schema: None,
            importers: v.importers.into_iter().map(Into::into).collect(),
            updaters: v.updaters.into_iter().map(Into::into).collect(),
            content_providers: v.content_providers.into_iter().map(Into::into).collect(),
            content_sources: v.content_sources.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PluginImporterCapability> for PluginImporterCapabilityDto {
    fn from(v: PluginImporterCapability) -> Self {
        Self {
            metadata: v.metadata.into(),
            handler: v.handler,
        }
    }
}

impl From<PluginUpdaterCapability> for PluginUpdaterCapabilityDto {
    fn from(v: PluginUpdaterCapability) -> Self {
        Self {
            metadata: v.metadata.into(),
            handler: v.handler,
        }
    }
}

impl From<PluginContentProviderCapability> for PluginContentProviderCapabilityDto {
    fn from(v: PluginContentProviderCapability) -> Self {
        Self {
            metadata: v.metadata.into(),
            handlers: v.handlers.into(),
        }
    }
}

impl From<ProviderHandlers> for ProviderHandlersDto {
    fn from(v: ProviderHandlers) -> Self {
        Self {
            search: v.search,
            get_content: v.get_content,
            list_version: v.list_version,
            install_atomic: v.install_atomic,
            install_modpack: v.install_modpack,
            check_compatibility: v.check_compatibility,
        }
    }
}

impl From<PluginContentSourceCapability> for PluginContentSourceCapabilityDto {
    fn from(v: PluginContentSourceCapability) -> Self {
        Self {
            metadata: v.metadata.into(),
            handlers: v.handlers.into(),
        }
    }
}

impl From<ContentSourceHandlers> for ContentSourceHandlersDto {
    fn from(v: ContentSourceHandlers) -> Self {
        Self {
            search: v.search,
            get_content: v.get_content,
            list_version: v.list_version,
            get_version_info: v.get_version_info,
            check_compatibility: v.check_compatibility,
        }
    }
}

// Capability metadata converters (instance domain → plugin-api DTO)

impl From<CapabilityMetadata> for CapabilityMetadataDto {
    fn from(v: CapabilityMetadata) -> Self {
        Self {
            id: v.id,
            name: v.name,
            description: v.description,
            icon: v.icon,
        }
    }
}

impl From<ImporterCapabilityMetadata> for ImporterCapabilityMetadataDto {
    fn from(v: ImporterCapabilityMetadata) -> Self {
        Self {
            base: v.base.into(),
            field_label: v.field_label,
            supported_extensions: v.supported_extensions,
        }
    }
}

impl From<UpdaterCapabilityMetadata> for UpdaterCapabilityMetadataDto {
    fn from(v: UpdaterCapabilityMetadata) -> Self {
        Self {
            base: v.base.into(),
        }
    }
}

impl From<ContentProviderCapabilityMetadata> for ContentProviderCapabilityMetadataDto {
    fn from(v: ContentProviderCapabilityMetadata) -> Self {
        Self {
            base: v.base.into(),
            supports_install_atomic: v.supports_install_atomic,
            supports_install_modpacks: v.supports_install_modpacks,
        }
    }
}

impl From<ContentSourceCapabilityMetadata> for ContentSourceCapabilityMetadataDto {
    fn from(v: ContentSourceCapabilityMetadata) -> Self {
        Self {
            base: v.base.into(),
        }
    }
}
