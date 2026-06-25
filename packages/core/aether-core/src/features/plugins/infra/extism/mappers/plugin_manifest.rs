use std::str::FromStr;

use aether_core_plugin_api::v0::{
    ApiConfigDto, LoadConfigDto, PathMappingDto, PluginManifestDto, PluginMetadataDto,
    RuntimeConfigDto,
};

use crate::features::plugins::domain::{
    ApiConfig, LoadConfig, LoadConfigType, ManifestError, PathMapping, PluginManifest,
    PluginMetadata, RuntimeConfig,
};

// ── DTO → Domain (fallible — semver parsing) ──

impl TryFrom<PluginManifestDto> for PluginManifest {
    type Error = ManifestError;

    fn try_from(dto: PluginManifestDto) -> Result<Self, Self::Error> {
        Ok(Self {
            metadata: dto.metadata.try_into()?,
            runtime: dto.runtime.into(),
            load: dto.load.into(),
            api: dto.api.try_into()?,
        })
    }
}

impl TryFrom<PluginMetadataDto> for PluginMetadata {
    type Error = ManifestError;

    fn try_from(dto: PluginMetadataDto) -> Result<Self, Self::Error> {
        let version =
            semver::Version::parse(&dto.version).map_err(|e| ManifestError::InvalidSemver {
                field: "metadata.version".to_owned(),
                value: dto.version.clone(),
                error: e.to_string(),
            })?;

        Ok(Self {
            id: dto.id,
            name: dto.name,
            version,
            description: dto.description,
            authors: dto.authors,
            license: dto.license,
        })
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

impl TryFrom<ApiConfigDto> for ApiConfig {
    type Error = ManifestError;

    fn try_from(dto: ApiConfigDto) -> Result<Self, Self::Error> {
        let version = semver::VersionReq::from_str(&dto.version).map_err(|e| {
            ManifestError::InvalidSemver {
                field: "api.version".to_owned(),
                value: dto.version.clone(),
                error: e.to_string(),
            }
        })?;

        Ok(Self {
            version,
            features: dto.features,
        })
    }
}

// ── Domain → DTO (infallible) ──

impl From<PluginManifest> for PluginManifestDto {
    fn from(v: PluginManifest) -> Self {
        Self {
            dollar_schema: None,
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
