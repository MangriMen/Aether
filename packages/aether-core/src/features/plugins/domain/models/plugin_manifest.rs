use std::path::{Path, PathBuf};

use crate::features::instance::{PluginId, ProviderId};

use super::{ManifestError, PluginFilePath};

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
    /// Unique identifier for the plugin, never [`ProviderId::BUILTIN_PLUGIN_ID`] (see
    /// [`PluginMetadata::validate_id`]).
    pub id: PluginId,

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

    /// Ignored by the launcher, which logs a warning: folders are given to a plugin by the
    /// user, in its settings. Kept so that manifests declaring it still load.
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
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum LoadConfig {
    /// Use WebAssembly (Extism) for a secure, cross-platform sandbox.
    Extism {
        /// Path to the .wasm file relative to the plugin root.
        file: PluginFilePath,
        /// Maximum memory (in bytes) the plugin is allowed to allocate.
        memory_limit: Option<usize>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApiConfig {
    /// Required API version range (`SemVer` requirement).
    pub version: semver::VersionReq,

    /// List of optional host features requested by the plugin.
    pub features: Vec<String>,

    /// Host functions the plugin declares it calls (`api.hostFunctions`).
    ///
    /// The plugin is linked with exactly these. Required: `None` (a manifest older than the
    /// field) is representable only so the plugin is reported incompatible instead of unparsable.
    pub host_functions: Option<Vec<String>>,
}

impl PluginManifest {
    /// The checks that hold however the plugin is enabled: force-enabling skips the API
    /// version check, never these.
    ///
    /// `available_host_functions` is what the plugin's loader can link.
    pub fn validate_requirements(
        &self,
        available_host_functions: &[&str],
    ) -> Result<(), ManifestError> {
        self.metadata.validate_id()?;
        self.runtime.validate()?;
        self.api.validate_host_functions(available_host_functions)?;

        Ok(())
    }
}

impl PluginMetadata {
    /// The id must not be the one reserved for the providers built into Aether: a plugin taking
    /// it would pass its providers off as built-in ones.
    pub fn validate_id(&self) -> Result<(), ManifestError> {
        if self.id.as_str() == ProviderId::BUILTIN_PLUGIN_ID {
            return Err(ManifestError::ReservedPluginId);
        }

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
    Incompatible {
        required: semver::VersionReq,
        host: semver::Version,
    },
}

impl ApiConfig {
    pub fn validate(&self, api_version: &semver::Version) -> Result<(), ManifestError> {
        if !self.version.matches(api_version) {
            return Err(ManifestError::UnsupportedApi);
        }

        Ok(())
    }

    /// The declaration must be present, and every name in it one the loader can link. A missing
    /// declaration means the plugin was built against an API from before `api.hostFunctions`;
    /// unknown names usually mean it was built against a newer one than this host offers.
    pub fn validate_host_functions(&self, available: &[&str]) -> Result<(), ManifestError> {
        let Some(declared) = &self.host_functions else {
            return Err(ManifestError::HostFunctionsNotDeclared);
        };

        let mut unknown: Vec<String> = declared
            .iter()
            .filter(|name| !available.contains(&name.as_str()))
            .cloned()
            .collect();

        if unknown.is_empty() {
            return Ok(());
        }

        unknown.sort();
        unknown.dedup();

        Err(ManifestError::UnknownHostFunctions(unknown))
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

        Compatibility::Incompatible {
            required: self.version.clone(),
            host: api_version.clone(),
        }
    }
}

// Note: From<&LoadConfig> for LoadConfigType is implemented in mappers/plugin_manifest.rs

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn absolute_path() -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(r"C:\absolute\path")
        } else {
            PathBuf::from("/absolute/path")
        }
    }

    // ── PluginMetadata::validate_id ──

    fn metadata(id: &'static str) -> PluginMetadata {
        PluginMetadata {
            id: PluginId::from_static(id),
            name: "Test".to_owned(),
            version: semver::Version::new(0, 1, 0),
            description: None,
            authors: vec![],
            license: None,
        }
    }

    #[rstest]
    #[case::packwiz("packwiz")]
    #[case::underscore("curseforge_plugin")]
    #[case::dash("test-plugin")]
    fn should_accept_a_manifest_when_the_plugin_id_is_one_of_existing_plugins(
        #[case] id: &'static str,
    ) {
        assert!(metadata(id).validate_id().is_ok(), "{id:?}");
    }

    #[test]
    fn should_reject_a_manifest_when_the_plugin_id_is_reserved() {
        assert!(matches!(
            metadata("aether").validate_id(),
            Err(ManifestError::ReservedPluginId)
        ));
    }

    // ── RuntimeConfig::validate ──

    #[test]
    fn should_accept_a_manifest_when_a_path_mapping_is_relative() {
        let config = RuntimeConfig {
            allowed_hosts: vec![],
            allowed_paths: vec![PathMapping("relative/path".into(), PathBuf::new())],
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn should_reject_a_manifest_when_a_host_path_is_absolute() {
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
    fn should_accept_a_manifest_when_allowed_paths_are_empty() {
        let config = RuntimeConfig {
            allowed_hosts: vec![],
            allowed_paths: vec![],
        };
        assert!(config.validate().is_ok());
    }

    // ── ApiConfig::validate ──

    #[rstest]
    #[case::range_matches(">=0.1.0", semver::Version::new(0, 2, 0))]
    #[case::exact_version("=0.1.0", semver::Version::new(0, 1, 0))]
    fn should_accept_a_manifest_when_the_api_version_matches(
        #[case] requirement: &str,
        #[case] host: semver::Version,
    ) {
        assert!(api_requiring(requirement).validate(&host).is_ok());
    }
    #[test]
    fn should_reject_a_manifest_when_the_api_version_does_not_match() {
        let config = ApiConfig {
            version: semver::VersionReq::parse(">=1.0.0").unwrap(),
            features: vec![],
            host_functions: None,
        };
        assert!(matches!(
            config.validate(&semver::Version::new(0, 9, 0)),
            Err(ManifestError::UnsupportedApi)
        ));
    }

    // ── ApiConfig::check_compatibility ──

    fn api_requiring(version: &str) -> ApiConfig {
        ApiConfig {
            version: semver::VersionReq::parse(version).unwrap(),
            features: vec![],
            host_functions: None,
        }
    }

    #[test]
    fn should_report_the_required_and_host_versions_when_the_requirement_is_not_met() {
        let host = semver::Version::new(0, 3, 0);

        let compatibility = api_requiring("=0.2.0").check_compatibility(&host);

        assert_eq!(
            compatibility,
            Compatibility::Incompatible {
                required: semver::VersionReq::parse("=0.2.0").unwrap(),
                host,
            }
        );
    }

    #[test]
    fn should_report_a_major_mismatch_when_the_plugin_requires_the_next_major_version() {
        let host = semver::Version::new(1, 4, 0);

        let compatibility = api_requiring("^2.0.0").check_compatibility(&host);

        assert_eq!(
            compatibility,
            Compatibility::MajorMismatch {
                required: semver::VersionReq::parse("^2.0.0").unwrap(),
                host,
            }
        );
    }

    // ── ApiConfig::validate_host_functions ──

    const AVAILABLE: &[&str] = &["log", "get_id", "http_get"];

    fn api_declaring(host_functions: Option<Vec<&str>>) -> ApiConfig {
        ApiConfig {
            version: semver::VersionReq::STAR,
            features: vec![],
            host_functions: host_functions
                .map(|names| names.into_iter().map(str::to_owned).collect()),
        }
    }

    #[rstest]
    #[case::provided_by_the_host(&["log", "http_get"])]
    #[case::empty_declaration(&[])]
    fn should_accept_a_manifest_when_the_declared_host_functions_are_acceptable(
        #[case] declared: &[&str],
    ) {
        let config = api_declaring(Some(declared.to_vec()));
        assert!(config.validate_host_functions(AVAILABLE).is_ok());
    }

    /// A manifest without `api.hostFunctions` is refused rather than given every function.
    #[test]
    fn should_reject_a_manifest_when_it_has_no_host_function_declaration() {
        let config = api_declaring(None);
        assert!(matches!(
            config.validate_host_functions(AVAILABLE),
            Err(ManifestError::HostFunctionsNotDeclared)
        ));
    }

    #[test]
    fn should_reject_a_manifest_when_it_declares_unknown_host_functions_and_name_each_once() {
        let config = api_declaring(Some(vec![
            "log",
            "spawn_shell",
            "read_registry",
            "spawn_shell",
        ]));

        let error = config
            .validate_host_functions(AVAILABLE)
            .expect_err("functions the host does not provide must not pass");

        match &error {
            ManifestError::UnknownHostFunctions(names) => {
                assert_eq!(names, &["read_registry", "spawn_shell"]);
            }
            other => panic!("expected UnknownHostFunctions, got: {other:?}"),
        }
        assert!(
            error.to_string().contains("`spawn_shell`"),
            "the message should name the function, got: {error}"
        );
    }

    /// Matching is exact: the import namespace and case are part of the name.
    #[rstest]
    #[case::uppercase("HTTP_GET")]
    #[case::namespaced("extism:host/user::http_get")]
    #[case::leading_space(" http_get")]
    fn should_match_host_function_names_exactly_when_declared(#[case] name: &str) {
        let config = api_declaring(Some(vec![name]));
        assert!(
            config.validate_host_functions(AVAILABLE).is_err(),
            "`{name}` must not be taken for `http_get`"
        );
    }
}
