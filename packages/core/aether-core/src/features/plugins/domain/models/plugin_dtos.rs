use serde::{Deserialize, Serialize};

use crate::features::instance::{ContentCompatibilityCheckParams, Instance};

/// DTO for passing import params between host and plugin.
/// No serde rename — this serializes via `MessagePack` in the proxy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginImportInstance {
    pub importer_id: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct PluginCheckCompatibilityParams {
    pub instances: Vec<Instance>,
    pub check_params: ContentCompatibilityCheckParams,
}
