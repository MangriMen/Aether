use serde::{Deserialize, Serialize};

use crate::v0::{ContentItemDto, ProviderIdDto};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentSearchResultDto {
    pub page: i64,
    pub page_size: i64,
    pub page_count: i64,
    pub provider_id: ProviderIdDto,
    pub items: Vec<ContentItemDto>,
}
