use serde::{Deserialize, Serialize};

use super::{HostFunctionRisk, PluginManifest, PluginSourceType, is_wide_allowed_host};

/// Describes where a plugin was originally installed from,
/// enabling update checks and version switching.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum PluginSource {
    /// Plugin was installed from a remote provider (GitHub, Modrinth, etc.).
    Remote {
        /// The provider source type (e.g., "github", "modrinth").
        source_type: PluginSourceType,
        /// Provider-specific identifier (e.g., "owner/repo" for GitHub).
        identifier: String,
        /// The tag/release name currently installed.
        current_tag: String,
        /// The semver version string currently installed.
        current_version: String,
    },
    /// Plugin was installed from a local file/archive.
    /// No update support available.
    Local,
}

impl PluginSource {
    /// Returns the provider identifier for this source, if remote.
    pub fn identifier(&self) -> Option<&str> {
        match self {
            PluginSource::Remote { identifier, .. } => Some(identifier),
            PluginSource::Local => None,
        }
    }

    pub fn current_tag(&self) -> Option<&str> {
        match self {
            PluginSource::Remote { current_tag, .. } => Some(current_tag),
            PluginSource::Local => None,
        }
    }

    /// Returns the provider source type for this plugin source.
    pub fn to_source_type(&self) -> PluginSourceType {
        match self {
            PluginSource::Remote { source_type, .. } => source_type.clone(),
            PluginSource::Local => PluginSourceType::Local,
        }
    }
}

/// Lightweight manifest info for preview (no runtime/load details).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifestPreview {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub api_version: Option<String>,
    pub allowed_hosts: Vec<String>,
    /// The `allowed_hosts` entries that admit far more than one site, as the manifest spells them.
    pub wide_allowed_hosts: Vec<String>,
    /// The host functions the plugin declares; none when its manifest does not say.
    pub host_functions: Option<Vec<String>>,
    /// What the declared host functions let the plugin do that the user should be warned about.
    pub host_function_risks: Vec<HostFunctionRisk>,
}

impl From<&PluginManifest> for PluginManifestPreview {
    fn from(manifest: &PluginManifest) -> Self {
        Self {
            id: manifest.metadata.id.to_string(),
            name: manifest.metadata.name.clone(),
            version: manifest.metadata.version.to_string(),
            description: manifest.metadata.description.clone(),
            authors: manifest.metadata.authors.clone(),
            license: manifest.metadata.license.clone(),
            api_version: Some(manifest.api.version.to_string()),
            allowed_hosts: manifest.runtime.allowed_hosts.clone(),
            wide_allowed_hosts: manifest
                .runtime
                .allowed_hosts
                .iter()
                .filter(|entry| is_wide_allowed_host(entry))
                .cloned()
                .collect(),
            host_functions: manifest.api.host_functions.clone(),
            host_function_risks: manifest.api.host_function_risks(),
        }
    }
}
