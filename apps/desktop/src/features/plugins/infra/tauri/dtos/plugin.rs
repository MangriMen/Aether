use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::plugins::{
    ApiConfigDto, LoadConfigDto, PathMappingDto, PluginCapabilitiesDto,
    PluginContentProviderCapabilityDto, PluginImporterCapabilityDto, PluginManifestDto,
    PluginMetadataDto, PluginUpdaterCapabilityDto, ProviderHandlersDto, RuntimeConfigDto,
};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum PluginDtoState {
    NotLoaded,
    Loading,
    Loaded,
    Unloading,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PluginDto {
    pub manifest: PluginManifestDto,
    pub capabilities: Option<PluginCapabilitiesDto>,
    pub state: PluginDtoState,
}

impl From<aether_core::features::plugins::PluginDto> for PluginDto {
    fn from(value: aether_core::features::plugins::PluginDto) -> Self {
        Self {
            manifest: value.manifest.into(),
            capabilities: value.capabilities.map(Into::into),
            state: value.state.into(),
        }
    }
}

// ── App DTO → Desktop DTO converters ──

impl From<aether_core::features::plugins::PluginManifestDto> for PluginManifestDto {
    fn from(dto: aether_core::features::plugins::PluginManifestDto) -> Self {
        Self {
            metadata: dto.metadata.into(),
            runtime: dto.runtime.into(),
            load: dto.load.into(),
            api: dto.api.into(),
        }
    }
}

impl From<aether_core::features::plugins::PluginMetadataDto> for PluginMetadataDto {
    fn from(dto: aether_core::features::plugins::PluginMetadataDto) -> Self {
        Self {
            id: dto.id,
            name: dto.name,
            version: dto.version,
            description: dto.description,
            authors: dto.authors,
            license: dto.license,
        }
    }
}

impl From<aether_core::features::plugins::RuntimeConfigDto> for RuntimeConfigDto {
    fn from(dto: aether_core::features::plugins::RuntimeConfigDto) -> Self {
        Self {
            allowed_hosts: dto.allowed_hosts,
            allowed_paths: dto.allowed_paths.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<aether_core::features::plugins::PathMappingDto> for PathMappingDto {
    fn from(dto: aether_core::features::plugins::PathMappingDto) -> Self {
        Self(dto.0, dto.1)
    }
}

impl From<aether_core::features::plugins::LoadConfigDto> for LoadConfigDto {
    fn from(dto: aether_core::features::plugins::LoadConfigDto) -> Self {
        match dto {
            aether_core::features::plugins::LoadConfigDto::Extism { file, memory_limit } => {
                Self::Extism { file, memory_limit }
            }
            aether_core::features::plugins::LoadConfigDto::Native { lib_path } => {
                Self::Native { lib_path }
            }
        }
    }
}

impl From<aether_core::features::plugins::ApiConfigDto> for ApiConfigDto {
    fn from(dto: aether_core::features::plugins::ApiConfigDto) -> Self {
        Self {
            version: dto.version,
            features: dto.features,
        }
    }
}

impl From<aether_core::features::plugins::PluginCapabilitiesDto> for PluginCapabilitiesDto {
    fn from(dto: aether_core::features::plugins::PluginCapabilitiesDto) -> Self {
        Self {
            importers: dto.importers.into_iter().map(Into::into).collect(),
            updaters: dto.updaters.into_iter().map(Into::into).collect(),
            content_providers: dto.content_providers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<aether_core::features::plugins::PluginImporterCapabilityDto>
    for PluginImporterCapabilityDto
{
    fn from(dto: aether_core::features::plugins::PluginImporterCapabilityDto) -> Self {
        Self {
            metadata: dto.metadata.into(),
            handler: dto.handler,
        }
    }
}

impl From<aether_core::features::plugins::PluginUpdaterCapabilityDto>
    for PluginUpdaterCapabilityDto
{
    fn from(dto: aether_core::features::plugins::PluginUpdaterCapabilityDto) -> Self {
        Self {
            metadata: dto.metadata.into(),
            handler: dto.handler,
        }
    }
}

impl From<aether_core::features::plugins::PluginContentProviderCapabilityDto>
    for PluginContentProviderCapabilityDto
{
    fn from(dto: aether_core::features::plugins::PluginContentProviderCapabilityDto) -> Self {
        Self {
            metadata: dto.metadata.into(),
            handlers: dto.handlers.into(),
        }
    }
}

impl From<aether_core::features::plugins::ProviderHandlersDto> for ProviderHandlersDto {
    fn from(dto: aether_core::features::plugins::ProviderHandlersDto) -> Self {
        Self {
            search: dto.search,
            get_content: dto.get_content,
            list_version: dto.list_version,
            install_atomic: dto.install_atomic,
            install_modpack: dto.install_modpack,
            check_compatibility: dto.check_compatibility,
        }
    }
}

impl From<aether_core::features::plugins::PluginDtoState> for PluginDtoState {
    fn from(value: aether_core::features::plugins::PluginDtoState) -> Self {
        match value {
            aether_core::features::plugins::PluginDtoState::NotLoaded => Self::NotLoaded,
            aether_core::features::plugins::PluginDtoState::Loading => Self::Loading,
            aether_core::features::plugins::PluginDtoState::Loaded => Self::Loaded,
            aether_core::features::plugins::PluginDtoState::Unloading => Self::Unloading,
            aether_core::features::plugins::PluginDtoState::Failed => Self::Failed,
        }
    }
}
