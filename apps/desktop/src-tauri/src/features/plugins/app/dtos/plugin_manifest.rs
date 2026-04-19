use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use specta::Type;

/// Root configuration for an Aether plugin.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifestDto {
    /// Information about the plugin identity.
    pub metadata: PluginMetadataDto,
    /// Sandbox and security restrictions for the plugin.
    pub runtime: RuntimeConfigDto,
    /// Defines how the plugin code should be loaded and executed.
    pub load: LoadConfigDto,
    /// Integration with the host application API.
    pub api: ApiConfigDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[serde(rename_all = "camelCase")]
pub struct PluginMetadataDto {
    /// Unique identifier for the plugin (lowercase, kebab-case).
    pub id: String,

    /// Readable name of the plugin.
    pub name: String,

    /// Semantic version of the plugin.
    pub version: semver::Version,

    /// Short summary of the plugin's purpose.
    pub description: Option<String>,

    /// List of plugin authors.
    #[serde(default)]
    pub authors: Vec<String>,

    /// License under which the plugin is distributed.
    pub license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeConfigDto {
    /// List of domains or IP addresses the plugin is allowed to connect to.
    #[serde(default)]
    pub allowed_hosts: Vec<String>,

    /// Filesystem access restrictions. Maps host paths to plugin-internal paths.
    #[serde(default)]
    pub allowed_paths: Vec<PathMappingDto>,
}

/// A mapping between a path on the host and a virtual path in the plugin.
/// Format: [`host_path`, `virtual_path`]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub struct PathMappingDto(
    /// Path on the host disk.
    pub String,
    /// Virtual path inside the plugin sandbox.
    pub PathBuf,
);

impl From<PathMappingDto> for (String, PathBuf) {
    fn from(m: PathMappingDto) -> Self {
        (m.0, m.1)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq, Type)]
#[serde(rename_all = "snake_case")]
pub enum LoadConfigTypeDto {
    Extism,
    Native,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq, Type)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum LoadConfigDto {
    /// Use WebAssembly (Extism) for a secure, cross-platform sandbox.
    #[serde(rename_all = "camelCase")]
    Extism {
        /// Path to the .wasm file relative to the plugin root.
        file: PathBuf,
        /// Maximum memory (in bytes) the plugin is allowed to allocate.
        #[serde(default)]
        memory_limit: Option<usize>,
    },
    /// Load a native shared library (.dll, .so, .dylib). Less secure, but faster.
    #[serde(rename_all = "camelCase")]
    Native {
        /// Path to the dynamic library file relative to the plugin root.
        lib_path: PathBuf,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[serde(rename_all = "camelCase")]
pub struct ApiConfigDto {
    /// Required API version range (`SemVer` requirement).
    #[specta(type = String)]
    pub version: semver::VersionReq,

    /// List of optional host features requested by the plugin.
    #[serde(default)]
    pub features: Vec<String>,
}

impl From<aether_core::features::plugins::PluginManifest> for PluginManifestDto {
    fn from(m: aether_core::features::plugins::PluginManifest) -> Self {
        Self {
            metadata: m.metadata.into(),
            runtime: m.runtime.into(),
            load: m.load.into(),
            api: m.api.into(),
        }
    }
}

impl From<aether_core::features::plugins::PluginMetadata> for PluginMetadataDto {
    fn from(m: aether_core::features::plugins::PluginMetadata) -> Self {
        Self {
            id: m.id,
            name: m.name,
            version: m.version,
            description: m.description,
            authors: m.authors,
            license: m.license,
        }
    }
}

impl From<aether_core::features::plugins::RuntimeConfig> for RuntimeConfigDto {
    fn from(m: aether_core::features::plugins::RuntimeConfig) -> Self {
        Self {
            allowed_hosts: m.allowed_hosts,
            allowed_paths: m.allowed_paths.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<aether_core::features::plugins::PathMapping> for PathMappingDto {
    fn from(m: aether_core::features::plugins::PathMapping) -> Self {
        Self(m.0, m.1)
    }
}

impl From<PathMappingDto> for aether_core::features::plugins::PathMapping {
    fn from(value: PathMappingDto) -> Self {
        Self(value.0, value.1)
    }
}

impl From<aether_core::features::plugins::LoadConfig> for LoadConfigDto {
    fn from(m: aether_core::features::plugins::LoadConfig) -> Self {
        match m {
            aether_core::features::plugins::LoadConfig::Extism { file, memory_limit } => {
                Self::Extism { file, memory_limit }
            }
            aether_core::features::plugins::LoadConfig::Native { lib_path } => {
                Self::Native { lib_path }
            }
        }
    }
}

impl From<aether_core::features::plugins::ApiConfig> for ApiConfigDto {
    fn from(m: aether_core::features::plugins::ApiConfig) -> Self {
        Self {
            version: m.version,
            features: m.features,
        }
    }
}
