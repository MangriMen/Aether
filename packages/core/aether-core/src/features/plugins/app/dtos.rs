use std::path::PathBuf;

use dashmap::mapref::{multiple::RefMulti as DashMapRefMulti, one::Ref as DashMapRef};
use serde::{Deserialize, Serialize};

use crate::features::{
    instance::{
        CapabilityMetadata, ContentProviderCapabilityMetadata, ImporterCapabilityMetadata,
        UpdaterCapabilityMetadata,
    },
    plugins::domain::{
        ApiConfig, LoadConfig, LoadConfigType, PathMapping, Plugin, PluginCapabilities,
        PluginContentProviderCapability, PluginImporterCapability, PluginManifest, PluginMetadata,
        PluginState, PluginUpdaterCapability, ProviderHandlers, RuntimeConfig,
    },
};

// ── PluginManifest DTO ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifestDto {
    pub metadata: PluginMetadataDto,
    pub runtime: RuntimeConfigDto,
    pub load: LoadConfigDto,
    pub api: ApiConfigDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadataDto {
    pub id: String,
    pub name: String,
    pub version: semver::Version,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfigDto {
    pub allowed_hosts: Vec<String>,
    pub allowed_paths: Vec<PathMappingDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathMappingDto(pub String, pub PathBuf);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum LoadConfigTypeDto {
    Extism,
    Native,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum LoadConfigDto {
    Extism {
        file: PathBuf,
        #[serde(default)]
        memory_limit: Option<usize>,
    },
    Native {
        lib_path: PathBuf,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfigDto {
    pub version: semver::VersionReq,
    pub features: Vec<String>,
}

// ── PluginCapabilities DTO ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMetadataDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImporterCapabilityMetadataDto {
    pub base: CapabilityMetadataDto,
    pub field_label: Option<String>,
    pub supported_extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdaterCapabilityMetadataDto {
    pub base: CapabilityMetadataDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentProviderCapabilityMetadataDto {
    pub base: CapabilityMetadataDto,
    pub supports_install_atomic: bool,
    pub supports_install_modpacks: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginCapabilitiesDto {
    pub importers: Vec<PluginImporterCapabilityDto>,
    pub updaters: Vec<PluginUpdaterCapabilityDto>,
    pub content_providers: Vec<PluginContentProviderCapabilityDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginImporterCapabilityDto {
    pub metadata: ImporterCapabilityMetadataDto,
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUpdaterCapabilityDto {
    pub metadata: UpdaterCapabilityMetadataDto,
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginContentProviderCapabilityDto {
    pub metadata: ContentProviderCapabilityMetadataDto,
    pub handlers: ProviderHandlersDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHandlersDto {
    pub search: String,
    pub get_content: String,
    pub list_version: Option<String>,
    pub install_atomic: String,
    pub install_modpack: Option<String>,
    pub check_compatibility: Option<String>,
}

// ── PluginDto ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginDtoState {
    NotLoaded,
    Loading,
    Loaded,
    Unloading,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDto {
    pub manifest: PluginManifestDto,
    pub capabilities: Option<PluginCapabilitiesDto>,
    pub state: PluginDtoState,
}

// ── Converters: Domain → App DTO ──

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
            version: v.version,
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
        Self(v.0, v.1)
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

impl From<LoadConfigType> for LoadConfigTypeDto {
    fn from(v: LoadConfigType) -> Self {
        match v {
            LoadConfigType::Extism => Self::Extism,
            LoadConfigType::Native => Self::Native,
        }
    }
}

impl From<ApiConfig> for ApiConfigDto {
    fn from(v: ApiConfig) -> Self {
        Self {
            version: v.version,
            features: v.features,
        }
    }
}

impl From<PluginCapabilities> for PluginCapabilitiesDto {
    fn from(v: PluginCapabilities) -> Self {
        Self {
            importers: v.importers.into_iter().map(Into::into).collect(),
            updaters: v.updaters.into_iter().map(Into::into).collect(),
            content_providers: v.content_providers.into_iter().map(Into::into).collect(),
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

// ── Capability metadata converters ──

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

impl From<&PluginState> for PluginDtoState {
    fn from(value: &PluginState) -> Self {
        match value {
            PluginState::NotLoaded => PluginDtoState::NotLoaded,
            PluginState::Loading => PluginDtoState::Loading,
            PluginState::Loaded(_) => PluginDtoState::Loaded,
            PluginState::Unloading => PluginDtoState::Unloading,
            PluginState::Failed(_) => PluginDtoState::Failed,
        }
    }
}

impl From<Plugin> for PluginDto {
    fn from(value: Plugin) -> Self {
        Self {
            manifest: value.manifest.into(),
            capabilities: value.capabilities.map(Into::into),
            state: (&value.state).into(),
        }
    }
}

impl From<DashMapRef<'_, String, Plugin>> for PluginDto {
    fn from(value: DashMapRef<'_, String, Plugin>) -> Self {
        Self {
            manifest: value.manifest.clone().into(),
            capabilities: value.capabilities.clone().map(Into::into),
            state: (&value.state).into(),
        }
    }
}

impl From<DashMapRefMulti<'_, String, Plugin>> for PluginDto {
    fn from(value: DashMapRefMulti<'_, String, Plugin>) -> Self {
        Self {
            manifest: value.manifest.clone().into(),
            capabilities: value.capabilities.clone().map(Into::into),
            state: (&value.state).into(),
        }
    }
}
