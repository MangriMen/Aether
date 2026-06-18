use aether_core_plugin_api::v0::{
    AtomicInstallParamsDto, BaseContentParamsDto, ContentSearchParamsDto, ContentSearchResultDto,
    DownloadedContentDto, ModLoaderDto, ModpackInstallParamsDto,
};

use crate::features::instance::{
    AtomicInstallParams, BaseContentParams, ContentSearchParams, ContentSearchResult,
    DownloadedContent, ModpackInstallParams,
};

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

impl From<BaseContentParams> for BaseContentParamsDto {
    fn from(value: BaseContentParams) -> Self {
        Self {
            content_id: value.content_id,
            content_version: value.content_version,
            provider_id: value.provider_id.into(),
        }
    }
}

impl From<AtomicInstallParams> for AtomicInstallParamsDto {
    fn from(value: AtomicInstallParams) -> Self {
        Self {
            base: value.base.into(),
            instance_id: value.instance_id,
            game_version: value.game_version,
            loader: value.loader,
            content_type: value.content_type.into(),
        }
    }
}

impl From<ModpackInstallParams> for ModpackInstallParamsDto {
    fn from(value: ModpackInstallParams) -> Self {
        Self {
            base: value.base.into(),
        }
    }
}

impl From<DownloadedContent> for DownloadedContentDto {
    fn from(value: DownloadedContent) -> Self {
        Self {
            metadata: value.metadata.into(),
            temp_path: value.temp_path,
        }
    }
}
