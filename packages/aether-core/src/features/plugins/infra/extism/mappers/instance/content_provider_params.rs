use aether_core_plugin_api::v0::{ContentSearchParamsDto, ContentSearchResultDto, ModLoaderDto};

use crate::features::instance::{ContentSearchParams, ContentSearchResult};

impl From<ContentSearchParams> for ContentSearchParamsDto {
    fn from(value: ContentSearchParams) -> Self {
        Self {
            content_type: value.content_type.into(),
            provider_id: value.provider_id.into(),
            page: value.page,
            page_size: value.page_size,
            query: value.query,
            game_versions: value.game_versions,
            loader: value.loader.map(ModLoaderDto::from),
        }
    }
}

impl From<ContentSearchResult> for ContentSearchResultDto {
    fn from(value: ContentSearchResult) -> Self {
        Self {
            page: value.page,
            page_size: value.page_size,
            page_count: value.page_count,
            provider_id: value.provider_id.into(),
            items: value.items.into_iter().map(Into::into).collect(),
        }
    }
}
