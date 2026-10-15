use serde::{Deserialize, Serialize};

use crate::v0::{ContentItemDto, ProviderIdDto};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContentCompatibilityCheckParamsDto {
    pub provider_id: ProviderIdDto,
    pub content_item: ContentItemDto,
}
