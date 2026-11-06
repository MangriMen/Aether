use serde::{Deserialize, Serialize};

use crate::v0::{ContentItemDto, ProviderIdDto};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentSearchResultDto {
    /// The page the items are of, counted from 1 (the host refuses `0`, also for an empty result).
    pub page: u32,
    /// How many items a page holds, 1 to 1000 (the host refuses anything else).
    pub page_size: u32,
    /// How many pages the search has; `0` for no results.
    pub page_count: u32,
    pub provider_id: ProviderIdDto,
    pub items: Vec<ContentItemDto>,
}
