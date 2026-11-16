use serde::{Deserialize, Serialize};
use specta::Type;

use super::HostFunctionRiskDto;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PluginManifestPreviewDto {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub api_version: Option<String>,
    pub allowed_hosts: Vec<String>,
    /// The entries of `allowed_hosts` that admit far more than one site.
    pub wide_allowed_hosts: Vec<String>,
    pub host_functions: Option<Vec<String>>,
    pub host_function_risks: Vec<HostFunctionRiskDto>,
}

impl From<aether_core::features::plugins::PluginManifestPreview> for PluginManifestPreviewDto {
    fn from(value: aether_core::features::plugins::PluginManifestPreview) -> Self {
        Self {
            id: value.id,
            name: value.name,
            version: value.version,
            description: value.description,
            authors: value.authors,
            license: value.license,
            api_version: value.api_version,
            allowed_hosts: value.allowed_hosts,
            wide_allowed_hosts: value.wide_allowed_hosts,
            host_functions: value.host_functions,
            host_function_risks: value
                .host_function_risks
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}
