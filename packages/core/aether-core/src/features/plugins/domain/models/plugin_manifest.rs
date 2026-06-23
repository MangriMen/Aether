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
    pub fn validate(&self, api_version: &semver::Version) -> Result<(), ManifestError> {
        self.runtime.validate()?;
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
    pub fn validate(&self) -> Result<(), ManifestError> {
        // File existence is validated by the infra layer when loading the manifest.
        Ok(())
    }
}

/// Result of checking a plugin's API version compatibility against the host.
#[derive(Debug, Clone, PartialEq)]
pub enum Compatibility {
    /// The plugin API version is fully compatible with the host.
    Compatible,
    /// The plugin requires a higher major version of the host API.
    /// It can still be force-loaded at the user's own risk.
    MajorMismatch {
        required: semver::VersionReq,
        host: semver::Version,
    },
    /// The plugin version requirement cannot be satisfied (same-major mismatch
    /// or other incompatibility).
    Incompatible(String),
}

impl ApiConfig {
    pub fn validate(&self, api_version: &semver::Version) -> Result<(), ManifestError> {
        if !self.version.matches(api_version) {
            return Err(ManifestError::UnsupportedApi);
        }

        Ok(())
    }

    /// Check compatibility returning a detailed [`Compatibility`] enum instead
    /// of a simple error. This allows the caller to distinguish between a
    /// major-version mismatch (which can be force-enabled) and other failures.
    pub fn check_compatibility(&self, api_version: &semver::Version) -> Compatibility {
        if self.version.matches(api_version) {
            return Compatibility::Compatible;
        }

        // Detect major mismatch: does the requirement match the *next* major version?
        let next_major = api_version.major + 1;
        if self
            .version
            .matches(&semver::Version::new(next_major, 0, 0))
        {
            return Compatibility::MajorMismatch {
                required: self.version.clone(),
                host: api_version.clone(),
            };
        }

        Compatibility::Incompatible(format!(
            "Plugin requires {}, host API version is {}",
            self.version, api_version
        ))
    }
}

// Note: From<&LoadConfig> for LoadConfigType is implemented in mappers/plugin_manifest.rs

#[cfg(test)]
mod tests {
    use super::*;

    fn absolute_path() -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(r"C:\absolute\path")
        } else {
            PathBuf::from("/absolute/path")
        }
    }

    // ── RuntimeConfig::validate ──

    #[test]
    fn should_accept_relative_path_mapping() {
        let config = RuntimeConfig {
            allowed_hosts: vec![],
            allowed_paths: vec![PathMapping("relative/path".into(), PathBuf::new())],
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn should_reject_absolute_host_path() {
        let config = RuntimeConfig {
            allowed_hosts: vec![],
            allowed_paths: vec![PathMapping(
                absolute_path().to_string_lossy().into_owned(),
                PathBuf::new(),
            )],
        };
        assert!(matches!(
            config.validate(),
            Err(ManifestError::InvalidPathMapping)
        ));
    }

    #[test]
    fn should_accept_empty_allowed_paths() {
        let config = RuntimeConfig {
            allowed_hosts: vec![],
            allowed_paths: vec![],
        };
        assert!(config.validate().is_ok());
    }

    // ── ApiConfig::validate ──

    #[test]
    fn should_accept_matching_api_version() {
        let config = ApiConfig {
            version: semver::VersionReq::parse(">=0.1.0").unwrap(),
            features: vec![],
        };
        assert!(config.validate(&semver::Version::new(0, 2, 0)).is_ok());
    }

    #[test]
    fn should_reject_non_matching_api_version() {
        let config = ApiConfig {
            version: semver::VersionReq::parse(">=1.0.0").unwrap(),
            features: vec![],
        };
        assert!(matches!(
            config.validate(&semver::Version::new(0, 9, 0)),
            Err(ManifestError::UnsupportedApi)
        ));
    }

    #[test]
    fn should_accept_exact_api_version() {
        let config = ApiConfig {
            version: semver::VersionReq::parse("=0.1.0").unwrap(),
            features: vec![],
        };
        assert!(config.validate(&semver::Version::new(0, 1, 0)).is_ok());
    }

    // ── PluginManifest::validate integration ──

    #[test]
    fn should_fail_when_invalid_path_mapping() {
        let manifest = PluginManifest {
            metadata: PluginMetadata {
                id: "test".into(),
                name: "Test".into(),
                version: semver::Version::new(0, 1, 0),
                description: None,
                authors: vec![],
                license: None,
            },
            runtime: RuntimeConfig {
                allowed_hosts: vec![],
                allowed_paths: vec![PathMapping(
                    absolute_path().to_string_lossy().into_owned(),
                    PathBuf::new(),
                )],
            },
            load: LoadConfig::Extism {
                file: PathBuf::from("plugin.wasm"),
                memory_limit: None,
            },
            api: ApiConfig {
                version: semver::VersionReq::STAR,
                features: vec![],
            },
        };
        assert!(matches!(
            manifest.validate(&semver::Version::new(0, 1, 0)),
            Err(ManifestError::InvalidPathMapping)
        ));
    }

    #[test]
    fn should_fail_when_unsupported_api() {
        let manifest = PluginManifest {
            metadata: PluginMetadata {
                id: "test".into(),
                name: "Test".into(),
                version: semver::Version::new(0, 1, 0),
                description: None,
                authors: vec![],
                license: None,
            },
            runtime: RuntimeConfig {
                allowed_hosts: vec![],
                allowed_paths: vec![],
            },
            load: LoadConfig::Extism {
                file: PathBuf::from("plugin.wasm"),
                memory_limit: None,
            },
            api: ApiConfig {
                version: semver::VersionReq::parse(">=2.0.0").unwrap(),
                features: vec![],
            },
        };
        assert!(matches!(
            manifest.validate(&semver::Version::new(1, 0, 0)),
            Err(ManifestError::UnsupportedApi)
        ));
    }

    #[test]
    fn should_pass_when_all_valid() {
        let manifest = PluginManifest {
            metadata: PluginMetadata {
                id: "test".into(),
                name: "Test".into(),
                version: semver::Version::new(0, 1, 0),
                description: None,
                authors: vec![],
                license: None,
            },
            runtime: RuntimeConfig {
                allowed_hosts: vec![],
                allowed_paths: vec![],
            },
            load: LoadConfig::Extism {
                file: PathBuf::from("plugin.wasm"),
                memory_limit: None,
            },
            api: ApiConfig {
                version: semver::VersionReq::STAR,
                features: vec![],
            },
        };
        assert!(manifest.validate(&semver::Version::new(0, 1, 0)).is_ok());
    }
}
