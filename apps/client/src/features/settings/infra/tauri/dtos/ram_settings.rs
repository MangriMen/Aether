use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use specta::Type;

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct RamSettingsDto {
    #[specta(type = String)]
    #[serde_as(as = "DisplayFromStr")]
    pub total_memory: u64,
}
