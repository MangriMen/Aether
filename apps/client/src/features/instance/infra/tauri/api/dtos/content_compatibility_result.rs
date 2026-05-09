use aether_core::features::instance::app::ContentCompatibilityResult;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentCompatibilityResultDto {
    pub is_compatible: bool,
}

impl From<ContentCompatibilityResult> for ContentCompatibilityResultDto {
    fn from(value: ContentCompatibilityResult) -> Self {
        Self {
            is_compatible: value.is_compatible,
        }
    }
}
