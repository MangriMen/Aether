use serde::{Deserialize, Serialize};

use super::PluginSourceType;

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
}
