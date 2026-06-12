use std::path::PathBuf;

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

impl From<aether_core::plugin_api::v0::PluginManifestDto> for PluginManifestDto {
    fn from(dto: aether_core::plugin_api::v0::PluginManifestDto) -> Self {
        Self {
            metadata: PluginMetadataDto {
                id: dto.metadata.id,
                name: dto.metadata.name,
                version: semver::Version::parse(&dto.metadata.version)
                    .unwrap_or(semver::Version::new(0, 0, 0)),
                description: dto.metadata.description,
                authors: dto.metadata.authors,
                license: dto.metadata.license,
            },
            runtime: RuntimeConfigDto {
                allowed_hosts: dto.runtime.allowed_hosts,
                allowed_paths: dto
                    .runtime
                    .allowed_paths
                    .into_iter()
                    .map(|pm| PathMappingDto(pm.0, PathBuf::from(pm.1)))
                    .collect(),
            },
            load: dto.load.into(),
            api: ApiConfigDto {
                version: semver::VersionReq::parse(&dto.api.version)
                    .unwrap_or(semver::VersionReq::STAR),
                features: dto.api.features,
            },
        }
    }
}

impl From<aether_core::plugin_api::v0::PluginCapabilitiesDto> for PluginCapabilitiesDto {
    fn from(dto: aether_core::plugin_api::v0::PluginCapabilitiesDto) -> Self {
        use aether_core::plugin_api::v0::{
            PluginContentProviderCapabilityDto as SrcProvider,
            PluginImporterCapabilityDto as SrcImporter, PluginUpdaterCapabilityDto as SrcUpdater,
        };

        Self {
            importers: dto
                .importers
                .into_iter()
                .map(|i: SrcImporter| PluginImporterCapabilityDto {
                    metadata: i.metadata.into(),
                    handler: i.handler,
                })
                .collect(),
            updaters: dto
                .updaters
                .into_iter()
                .map(|u: SrcUpdater| PluginUpdaterCapabilityDto {
                    metadata: u.metadata.into(),
                    handler: u.handler,
                })
                .collect(),
            content_providers: dto
                .content_providers
                .into_iter()
                .map(|cp: SrcProvider| PluginContentProviderCapabilityDto {
                    metadata: cp.metadata.into(),
                    handlers: ProviderHandlersDto {
                        search: cp.handlers.search,
                        get_content: cp.handlers.get_content,
                        list_version: cp.handlers.list_version,
                        install_atomic: cp.handlers.install_atomic,
                        install_modpack: cp.handlers.install_modpack,
                        check_compatibility: cp.handlers.check_compatibility,
                    },
                })
                .collect(),
        }
    }
}

impl From<aether_core::plugin_api::v0::LoadConfigDto> for LoadConfigDto {
    fn from(dto: aether_core::plugin_api::v0::LoadConfigDto) -> Self {
        match dto {
            aether_core::plugin_api::v0::LoadConfigDto::Extism { file, memory_limit } => {
                Self::Extism { file, memory_limit }
            }
            aether_core::plugin_api::v0::LoadConfigDto::Native { lib_path } => {
                Self::Native { lib_path }
            }
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
