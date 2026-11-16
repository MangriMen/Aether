use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::v0::{ContentCompatibilityCheckParamsDto, InstanceSummaryDto};

#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginImportInstanceDto {
    pub importer_id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize)]
pub struct PluginCheckCompatibilityParamsDto {
    pub instances: Vec<InstanceSummaryDto>,
    pub check_params: ContentCompatibilityCheckParamsDto,
}
