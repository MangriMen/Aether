use aether_core::features::instance::app::ContentGetParams;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::instance::ProviderIdDto;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentGetParamsDto {
    pub content_id: String,
    pub provider_id: ProviderIdDto,
}

impl From<ContentGetParamsDto> for ContentGetParams {
    fn from(value: ContentGetParamsDto) -> Self {
        Self {
            content_id: value.content_id,
            provider_id: value.provider_id.into(),
        }
    }
}
