use std::path::{Path, PathBuf};

use super::ManifestError;

/// Root configuration for an Aether plugin.
#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct PluginMetadata {
    /// Unique identifier for the plugin (lowercase, kebab-case).
    pub id: String,

    /// Readable name of the plugin.
    pub name: String,

    /// Semantic version of the plugin.
    pub version: semver::Version,

    /// Short summary of the plugin's purpose.
    pub description: Option<String>,

    /// List of plugin authors.
    pub authors: Vec<String>,

    /// License under which the plugin is distributed.
    pub license: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeConfig {
    /// List of domains or IP addresses the plugin is allowed to connect to.
    pub allowed_hosts: Vec<String>,

    /// Filesystem access restrictions. Maps host paths to plugin-internal paths.
    pub allowed_paths: Vec<PathMapping>,
}

/// A mapping between a path on the host and a virtual path in the plugin.
/// Format: [`host_path`, `virtual_path`]
#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum LoadConfigType {
    Extism,
    Native,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum LoadConfig {
    /// Use WebAssembly (Extism) for a secure, cross-platform sandbox.
    Extism {
        /// Path to the .wasm file relative to the plugin root.
        file: PathBuf,
        /// Maximum memory (in bytes) the plugin is allowed to allocate.
        memory_limit: Option<usize>,
    },
    /// Load a native shared library (.dll, .so, .dylib).
    Native {
        /// Path to the dynamic library file relative to the plugin root.
        lib_path: PathBuf,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApiConfig {
    /// Required API version range (`SemVer` requirement).
    pub version: semver::VersionReq,

    /// List of optional host features requested by the plugin.
    pub features: Vec<String>,
}

impl PluginManifest {
    pub fn validate(
        &self,
        api_version: &semver::Version,
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
                    return Err(ManifestError::InvalidFilePath { path: file.clone() });
                }
            }
            Self::Native { lib_path } => {
                let full_path = base_dir.join(lib_path);
                if !full_path.exists() {
                    return Err(ManifestError::InvalidFilePath {
                        path: lib_path.clone(),
                    });
                }
            }
        }

        Ok(())
    }
}

impl ApiConfig {
    pub fn validate(&self, api_version: &semver::Version) -> Result<(), ManifestError> {
        if !self.version.matches(api_version) {
            return Err(ManifestError::UnsupportedApi);
        }

        Ok(())
    }
}

// Note: From<&LoadConfig> for LoadConfigType is implemented in mappers/plugin_manifest.rs
