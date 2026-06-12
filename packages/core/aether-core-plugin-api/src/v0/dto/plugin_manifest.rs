use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Root configuration for an Aether plugin.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct PluginMetadataDto {
    /// Unique identifier for the plugin (lowercase, kebab-case).
    #[schemars(regex(pattern = r"^[a-z0-9_\-]+$"))]
    pub id: String,

    /// Readable name of the plugin.
    pub name: String,

    /// Semantic version of the plugin.
    #[schemars(with = "String")]
    pub version: String,

    /// Short summary of the plugin's purpose.
    pub description: Option<String>,

    /// List of plugin authors.
    #[serde(default)]
    pub authors: Vec<String>,

    /// License under which the plugin is distributed.
    pub license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct RuntimeConfigDto {
    /// List of domains or IP addresses the plugin is allowed to connect to.
    #[serde(default)]
    pub allowed_hosts: Vec<String>,

    /// Filesystem access restrictions. Maps host paths to plugin-internal paths.
    #[serde(default)]
    pub allowed_paths: Vec<PathMappingDto>,
}

/// A mapping between a path on the host and a virtual path in the plugin.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct PathMappingDto(
    /// Path on the host disk.
    pub String,
    /// Virtual path inside the plugin sandbox.
    pub String,
);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LoadConfigTypeDto {
    Extism,
    Native,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq, JsonSchema)]
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
    /// Load a native shared library (.dll, .so, .dylib).
    #[serde(rename_all = "camelCase")]
    Native {
        /// Path to the dynamic library file relative to the plugin root.
        lib_path: PathBuf,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ApiConfigDto {
    /// Required API version range (`SemVer` requirement).
    #[schemars(with = "String")]
    pub version: String,

    /// List of optional host features requested by the plugin.
    #[serde(default)]
    pub features: Vec<String>,
}
