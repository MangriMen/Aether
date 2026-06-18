use serde::{Deserialize, Serialize};

use crate::v0::{ContentCompatibilityCheckParamsDto, InstanceDto};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginImportInstanceDto {
    pub importer_id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCheckCompatibilityParamsDto {
    pub instances: Vec<InstanceDto>,
    pub check_params: ContentCompatibilityCheckParamsDto,
}
