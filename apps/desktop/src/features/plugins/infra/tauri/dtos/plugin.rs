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
    Incompatible,
    Failed,
}

/// Translated by its `code`; the payload fills in the message.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PluginStateReasonDto {
    IncompatibleApiVersion {
        required: semver::VersionReq,
        host: semver::Version,
    },
    InvalidManifest {
        details: String,
    },
    LoadFailed {
        details: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PluginDto {
    pub manifest: PluginManifestDto,
    pub capabilities: Option<PluginCapabilitiesDto>,
    pub state: PluginDtoState,
    pub state_reason: Option<PluginStateReasonDto>,
    /// The plugin runs with other folders than its settings give it now: they take effect once
    /// it restarts.
    pub restart_needed: bool,
    pub verified: bool,
}

impl From<aether_core::features::plugins::PluginDto> for PluginDto {
    fn from(value: aether_core::features::plugins::PluginDto) -> Self {
        Self {
            manifest: value.manifest.into(),
            capabilities: value.capabilities.map(Into::into),
            state: value.state.into(),
            state_reason: value.state_reason.map(Into::into),
            restart_needed: value.restart_needed,
            verified: value.verified,
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
        }
    }
}

impl From<aether_core::features::plugins::ApiConfigDto> for ApiConfigDto {
    fn from(dto: aether_core::features::plugins::ApiConfigDto) -> Self {
        Self {
            version: dto.version,
            features: dto.features,
            host_functions: dto.host_functions,
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
            check_compatibility: dto.check_compatibility,
            resolve_file: dto.resolve_file,
            resolve_install_plan: dto.resolve_install_plan,
            resolve_pack_metadata: dto.resolve_pack_metadata,
            get_contents_metadata: dto.get_contents_metadata,
            get_versions_metadata: dto.get_versions_metadata,
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
            aether_core::features::plugins::PluginDtoState::Incompatible => Self::Incompatible,
        }
    }
}

impl From<aether_core::features::plugins::PluginStateReasonDto> for PluginStateReasonDto {
    fn from(value: aether_core::features::plugins::PluginStateReasonDto) -> Self {
        use aether_core::features::plugins::PluginStateReasonDto as Core;
        match value {
            Core::IncompatibleApiVersion { required, host } => {
                Self::IncompatibleApiVersion { required, host }
            }
            Core::InvalidManifest { details } => Self::InvalidManifest { details },
            Core::LoadFailed { details } => Self::LoadFailed { details },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_as_a_code_with_a_payload_when_the_reason_reaches_the_frontend() {
        let reason = PluginStateReasonDto::IncompatibleApiVersion {
            required: semver::VersionReq::parse("^2.0.0").unwrap(),
            host: semver::Version::new(1, 4, 0),
        };

        let json = serde_json::to_value(&reason).unwrap();

        assert_eq!(
            json,
            serde_json::json!({
                "code": "INCOMPATIBLE_API_VERSION",
                "payload": { "required": "^2.0.0", "host": "1.4.0" },
            })
        );
    }
}
