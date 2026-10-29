use aether_core::features::instance::{ImportInstance, InstanceError};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct ImportInstanceDto {
    pub plugin_id: String,
    pub importer_id: String,
    pub path: String,
}

/// An id that does not parse names no importer, hence [`InstanceError::ImporterNotFound`].
impl TryFrom<ImportInstanceDto> for ImportInstance {
    type Error = InstanceError;

    fn try_from(value: ImportInstanceDto) -> Result<Self, Self::Error> {
        let not_found = || InstanceError::ImporterNotFound {
            importer_id: value.importer_id.clone(),
        };

        Ok(Self {
            plugin_id: value.plugin_id.parse().map_err(|_| not_found())?,
            importer_id: value.importer_id.parse().map_err(|_| not_found())?,
            path: value.path,
        })
    }
}
