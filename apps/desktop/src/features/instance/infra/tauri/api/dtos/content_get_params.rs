use aether_core::features::instance::{ContentGetParams, InstanceError};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::instance::infra::ProviderIdDto;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentGetParamsDto {
    pub content_id: String,
    pub provider_id: ProviderIdDto,
}

impl TryFrom<ContentGetParamsDto> for ContentGetParams {
    type Error = InstanceError;

    fn try_from(value: ContentGetParamsDto) -> Result<Self, Self::Error> {
        Ok(Self {
            content_id: value.content_id,
            provider_id: value.provider_id.try_into()?,
        })
    }
}
