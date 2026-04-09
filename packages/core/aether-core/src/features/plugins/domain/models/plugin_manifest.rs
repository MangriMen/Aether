use std::path::{Path, PathBuf};

use register_schema::RegisterSchema;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::ManifestError;

/// Root configuration for an Aether plugin.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema, RegisterSchema)]
#[schema_category("plugin_api")]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct PluginManifest {
    /// Information about the plugin identity.
    pub metadata: PluginMetadata,
    /// Sandbox and security restrictions for the plugin.
    pub runtime: RuntimeConfig,
    /// Defines how the plugin code should be loaded and executed.
    pub load: LoadConfig,
    /// Integration with the host application API.
    pub api: ApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct PluginMetadata {
    /// Unique identifier for the plugin (lowercase, kebab-case).
    #[schemars(regex(pattern = r"^[a-z0-9_\-]+$"))]
    pub id: String,

    /// Readable name of the plugin.
    pub name: String,

    /// Semantic version of the plugin.
    #[serde(with = "crate::shared::serde_semver")]
    #[schemars(with = "String")]
    pub version: semver::Version,

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
pub struct RuntimeConfig {
    /// List of domains or IP addresses the plugin is allowed to connect to.
    #[serde(default)]
    pub allowed_hosts: Vec<String>,

    /// Filesystem access restrictions. Maps host paths to plugin-internal paths.
    #[serde(default)]
    pub allowed_paths: Vec<PathMapping>,
}

/// A mapping between a path on the host and a virtual path in the plugin.
/// Format: [host_path, virtual_path]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct PathMapping(
    /// Path on the host disk.
    pub String,
    /// Virtual path inside the plugin sandbox.
    pub PathBuf,
);

impl From<PathMapping> for (String, PathBuf) {
    fn from(m: PathMapping) -> Self {
        (m.0, m.1)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LoadConfigType {
    Extism,
    Native,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum LoadConfig {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct ApiConfig {
    /// Required API version range (SemVer requirement).
    #[serde(with = "crate::shared::serde_semver")]
    #[schemars(with = "String")]
    pub version: semver::VersionReq,

    /// List of optional host features requested by the plugin.
    #[serde(default)]
    pub features: Vec<String>,
}

impl PluginManifest {
    pub fn validate(
        &self,
        api_version: semver::Version,
        base_dir: &Path,
    ) -> Result<(), ManifestError> {
        self.runtime.validate()?;
        self.load.validate(base_dir)?;
        self.api.validate(api_version)?;

        Ok(())
    }
}

impl RuntimeConfig {
    pub fn validate(&self) -> Result<(), ManifestError> {
        for mapping in &self.allowed_paths {
            if Path::new(&mapping.0).is_absolute() {
                return Err(ManifestError::InvalidPathMapping);
            }
        }

        Ok(())
    }
}

impl LoadConfig {
    pub fn validate(&self, base_dir: &Path) -> Result<(), ManifestError> {
        match &self {
            Self::Extism { file, .. } => {
                let full_path = base_dir.join(file);
                if !full_path.exists() {
                    return Err(ManifestError::InvalidFilePath {
                        path: file.to_path_buf(),
                    });
                }
            }
            Self::Native { lib_path } => {
                let full_path = base_dir.join(lib_path);
                if !full_path.exists() {
                    return Err(ManifestError::InvalidFilePath {
                        path: lib_path.to_path_buf(),
                    });
                }
            }
        }

        Ok(())
    }
}

impl ApiConfig {
    pub fn validate(&self, api_version: semver::Version) -> Result<(), ManifestError> {
        if !self.version.matches(&api_version) {
            return Err(ManifestError::UnsupportedApi);
        }

        Ok(())
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
