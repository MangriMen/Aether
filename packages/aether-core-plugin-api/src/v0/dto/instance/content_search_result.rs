use serde::{Deserialize, Serialize};

use crate::v0::{ContentItemDto, ProviderIdDto};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentSearchResultDto {
    pub page: u32,
    pub page_size: u32,
    pub page_count: u32,
    pub provider_id: ProviderIdDto,
    pub items: Vec<ContentItemDto>,
}
