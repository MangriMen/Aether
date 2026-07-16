use aether_core_plugin_api::v0::{
    CapabilityMetadataDto, ContentProviderCapabilityMetadataDto, PackManagerCapabilityMetadataDto,
    PackManagerHandlersDto, PluginCapabilitiesDto, PluginContentProviderCapabilityDto,
    PluginPackManagerCapabilityDto, ProviderHandlersDto,
};

use crate::features::{
    instance::{
        CapabilityMetadata, ContentProviderCapabilityMetadata, PackManagerCapabilityMetadata,
    },
    plugins::{
        PackManagerHandlers, PluginCapabilities, PluginContentProviderCapability,
        PluginPackManagerCapability, ProviderHandlers,
    },
};

impl From<PluginCapabilities> for PluginCapabilitiesDto {
    fn from(v: PluginCapabilities) -> Self {
        Self {
            dollar_schema: None,
            content_providers: v.content_providers.into_iter().map(Into::into).collect(),
            pack_managers: v.pack_managers.into_iter().map(Into::into).collect(),
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

impl From<PluginPackManagerCapability> for PluginPackManagerCapabilityDto {
    fn from(v: PluginPackManagerCapability) -> Self {
        Self {
            metadata: v.metadata.into(),
            handlers: v.handlers.into(),
        }
    }
}

impl From<PackManagerHandlers> for PackManagerHandlersDto {
    fn from(v: PackManagerHandlers) -> Self {
        Self {
            install: v.install,
            update: v.update,
            check_updates: v.check_updates,
            resolve_metadata: v.resolve_metadata,
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

impl From<ContentProviderCapabilityMetadata> for ContentProviderCapabilityMetadataDto {
    fn from(v: ContentProviderCapabilityMetadata) -> Self {
        Self {
            base: v.base.into(),
            supports_install_atomic: v.supports_install_atomic,
            supports_install_modpacks: v.supports_install_modpacks,
        }
    }
}

impl From<PackManagerCapabilityMetadata> for PackManagerCapabilityMetadataDto {
    fn from(v: PackManagerCapabilityMetadata) -> Self {
        Self {
            base: v.base.into(),
            supports_install: v.supports_install,
            supports_update: v.supports_update,
            supports_check_updates: v.supports_check_updates,
            field_label: v.field_label,
            supported_extensions: v.supported_extensions,
        }
    }
}
