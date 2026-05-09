use aether_core::features::instance::app::ImportInstance;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct ImportInstanceDto {
    pub plugin_id: String,
    pub importer_id: String,
    pub path: String,
}

impl From<ImportInstanceDto> for ImportInstance {
    fn from(value: ImportInstanceDto) -> Self {
        Self {
            plugin_id: value.plugin_id,
            importer_id: value.importer_id,
            path: value.path,
        }
    }
}
