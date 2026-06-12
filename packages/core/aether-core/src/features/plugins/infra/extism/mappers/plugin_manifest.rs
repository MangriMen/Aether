use std::str::FromStr;

use aether_core_plugin_api::v0::{
    ApiConfigDto, LoadConfigDto, PathMappingDto, PluginManifestDto, PluginMetadataDto,
    RuntimeConfigDto,
};

use crate::features::plugins::domain::{
    ApiConfig, LoadConfig, LoadConfigType, PathMapping, PluginManifest, PluginMetadata,
    RuntimeConfig,
};

// ── DTO → Domain ──

impl From<PluginManifestDto> for PluginManifest {
    fn from(dto: PluginManifestDto) -> Self {
        Self {
            metadata: dto.metadata.into(),
            runtime: dto.runtime.into(),
            load: dto.load.into(),
            api: dto.api.into(),
        }
    }
}

impl From<PluginMetadataDto> for PluginMetadata {
    fn from(dto: PluginMetadataDto) -> Self {
        Self {
            id: dto.id,
            name: dto.name,
            version: semver::Version::parse(&dto.version).unwrap_or(semver::Version::new(0, 0, 0)),
            description: dto.description,
            authors: dto.authors,
            license: dto.license,
        }
    }
}

impl From<RuntimeConfigDto> for RuntimeConfig {
    fn from(dto: RuntimeConfigDto) -> Self {
        Self {
            allowed_hosts: dto.allowed_hosts,
            allowed_paths: dto.allowed_paths.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PathMappingDto> for PathMapping {
    fn from(dto: PathMappingDto) -> Self {
        Self(dto.0, dto.1.into())
    }
}

impl From<LoadConfigDto> for LoadConfig {
    fn from(dto: LoadConfigDto) -> Self {
        match dto {
            LoadConfigDto::Extism { file, memory_limit } => Self::Extism { file, memory_limit },
            LoadConfigDto::Native { lib_path } => Self::Native { lib_path },
        }
    }
}

impl From<ApiConfigDto> for ApiConfig {
    fn from(dto: ApiConfigDto) -> Self {
        Self {
            version: semver::VersionReq::from_str(&dto.version).unwrap_or_default(),
            features: dto.features,
        }
    }
}

// ── Domain → DTO ──

impl From<PluginManifest> for PluginManifestDto {
    fn from(v: PluginManifest) -> Self {
        Self {
            metadata: v.metadata.into(),
            runtime: v.runtime.into(),
            load: v.load.into(),
            api: v.api.into(),
        }
    }
}

impl From<PluginMetadata> for PluginMetadataDto {
    fn from(v: PluginMetadata) -> Self {
        Self {
            id: v.id,
            name: v.name,
            version: v.version.to_string(),
            description: v.description,
            authors: v.authors,
            license: v.license,
        }
    }
}

impl From<RuntimeConfig> for RuntimeConfigDto {
    fn from(v: RuntimeConfig) -> Self {
        Self {
            allowed_hosts: v.allowed_hosts,
            allowed_paths: v.allowed_paths.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PathMapping> for PathMappingDto {
    fn from(v: PathMapping) -> Self {
        Self(v.0, v.1.to_string_lossy().to_string())
    }
}

impl From<LoadConfig> for LoadConfigDto {
    fn from(v: LoadConfig) -> Self {
        match v {
            LoadConfig::Extism { file, memory_limit } => Self::Extism { file, memory_limit },
            LoadConfig::Native { lib_path } => Self::Native { lib_path },
        }
    }
}

impl From<ApiConfig> for ApiConfigDto {
    fn from(v: ApiConfig) -> Self {
        Self {
            version: v.version.to_string(),
            features: v.features,
        }
    }
}

impl From<&LoadConfig> for LoadConfigType {
    fn from(config: &LoadConfig) -> Self {
        match config {
            LoadConfig::Extism { .. } => LoadConfigType::Extism,
            LoadConfig::Native { .. } => LoadConfigType::Native,
        }
    }
}
