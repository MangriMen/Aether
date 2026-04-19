use extism::ToBytes;
use extism_convert::Msgpack;
use serde::{Deserialize, Serialize};

use crate::features::instance::{Instance, app::ContentCompatibilityCheckParams};

#[derive(Debug, Clone, Serialize, Deserialize, ToBytes)]
#[encoding(Msgpack)]
#[serde(rename_all = "camelCase")]
pub struct PluginImportInstance {
    pub importer_id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCheckCompatibilityParams {
    pub instances: Vec<Instance>,
    pub check_params: ContentCompatibilityCheckParams,
}
