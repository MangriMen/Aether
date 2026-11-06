use serde::{Deserialize, Serialize};

use crate::v0::{ContentTypeDto, ModLoaderDto, ProviderIdDto};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentSearchParamsDto {
    pub content_type: ContentTypeDto,
    pub provider_id: ProviderIdDto,
    pub page: u32,
    pub page_size: u32,
    pub query: Option<String>,
    pub game_versions: Option<Vec<String>>,
    pub loader: Option<ModLoaderDto>,
}
