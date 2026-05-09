use aether_core::features::instance::app::ContentCompatibilityCheckParams;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::instance::{ContentItemDto, ProviderIdDto};

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentCompatibilityCheckParamsDto {
    pub provider_id: ProviderIdDto,
    pub content_item: ContentItemDto,
}

impl From<ContentCompatibilityCheckParamsDto> for ContentCompatibilityCheckParams {
    fn from(value: ContentCompatibilityCheckParamsDto) -> Self {
        Self {
            provider_id: value.provider_id.into(),
            content_item: value.content_item.into(),
        }
    }
}
