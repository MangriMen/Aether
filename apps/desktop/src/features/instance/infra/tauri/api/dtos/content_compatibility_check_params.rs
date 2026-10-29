use aether_core::features::instance::{ContentCompatibilityCheckParams, InstanceError};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::instance::infra::{ContentItemDto, ProviderIdDto};

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentCompatibilityCheckParamsDto {
    pub provider_id: ProviderIdDto,
    pub content_item: ContentItemDto,
}

impl TryFrom<ContentCompatibilityCheckParamsDto> for ContentCompatibilityCheckParams {
    type Error = InstanceError;

    fn try_from(value: ContentCompatibilityCheckParamsDto) -> Result<Self, Self::Error> {
        Ok(Self {
            provider_id: value.provider_id.try_into()?,
            content_item: value.content_item.into(),
        })
    }
}
