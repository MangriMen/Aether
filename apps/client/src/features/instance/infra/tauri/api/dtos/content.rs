use std::str::FromStr;

use aether_core::features::instance::{
    AtomicInstallParams, BaseContentParams, ContentInstallParams, ContentItem, ContentSearchParams,
    ContentSearchResult, ModpackInstallParams, ProviderId,
};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::{instance::ContentTypeDto, minecraft::ModLoaderDto};

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentSearchParamsDto {
    pub content_type: ContentTypeDto,
    pub provider_id: ProviderIdDto,
    pub page: i64,
    pub page_size: i64,
    #[specta(optional)]
    pub query: Option<String>,
    #[specta(optional)]
    pub game_versions: Option<Vec<String>>,
    #[specta(optional)]
    pub loader: Option<ModLoaderDto>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentSearchResultDto {
    pub page: i64,
    pub page_size: i64,
    pub page_count: i64,
    pub provider_id: ProviderIdDto,
    pub items: Vec<ContentItemDto>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProviderIdDto {
    pub plugin_id: String,
    pub capability_id: String,
}

impl ProviderIdDto {
    pub const SEPARATOR: char = '#';
}

impl std::fmt::Display for ProviderIdDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.plugin_id,
            Self::SEPARATOR,
            self.capability_id
        )
    }
}

impl FromStr for ProviderIdDto {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (plugin, cap) = s.split_once(Self::SEPARATOR).ok_or_else(|| {
            format!(
                "Invalid format '{}'. Expected plugin{}capability",
                s,
                Self::SEPARATOR
            )
        })?;

        Ok(Self {
            plugin_id: plugin.to_string(),
            capability_id: cap.to_string(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct BaseContentParamsDto {
    pub content_id: String,
    #[specta(optional)]
    pub content_version: Option<String>,
    pub provider_id: ProviderIdDto,
}

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct AtomicInstallParamsDto {
    #[serde(flatten)]
    pub base: BaseContentParamsDto,
    pub instance_id: String,
    pub game_version: String,
    #[specta(optional)]
    pub loader: Option<String>,
    pub content_type: ContentTypeDto,
}

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct ModpackInstallParamsDto {
    #[serde(flatten)]
    pub base: BaseContentParamsDto,
}

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum ContentInstallParamsDto {
    Atomic(AtomicInstallParamsDto),
    Modpack(ModpackInstallParamsDto),
}

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentItemDto {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub long_description: Option<String>,
    pub author: String,
    pub url: String,
    pub icon_url: String,
    pub versions: Vec<String>,
    pub content_type: ContentTypeDto,
}

impl From<ContentSearchParamsDto> for ContentSearchParams {
    fn from(value: ContentSearchParamsDto) -> Self {
        Self {
            content_type: value.content_type.into(),
            provider_id: value.provider_id.into(),
            page: value.page,
            page_size: value.page_size,
            query: value.query,
            game_versions: value.game_versions,
            loader: value.loader.map(Into::into),
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

impl From<ProviderIdDto> for ProviderId {
    fn from(value: ProviderIdDto) -> Self {
        Self {
            plugin_id: value.plugin_id,
            capability_id: value.capability_id,
        }
    }
}

impl From<ProviderId> for ProviderIdDto {
    fn from(value: ProviderId) -> Self {
        Self {
            plugin_id: value.plugin_id,
            capability_id: value.capability_id,
        }
    }
}

impl From<BaseContentParamsDto> for BaseContentParams {
    fn from(value: BaseContentParamsDto) -> Self {
        Self {
            content_id: value.content_id,
            content_version: value.content_version,
            provider_id: value.provider_id.into(),
        }
    }
}

impl From<AtomicInstallParamsDto> for AtomicInstallParams {
    fn from(value: AtomicInstallParamsDto) -> Self {
        Self {
            base: value.base.into(),
            instance_id: value.instance_id,
            game_version: value.game_version,
            loader: value.loader,
            content_type: value.content_type.into(),
        }
    }
}

impl From<ModpackInstallParamsDto> for ModpackInstallParams {
    fn from(value: ModpackInstallParamsDto) -> Self {
        Self {
            base: value.base.into(),
        }
    }
}

impl From<ContentInstallParamsDto> for ContentInstallParams {
    fn from(value: ContentInstallParamsDto) -> Self {
        match value {
            ContentInstallParamsDto::Atomic(x) => Self::Atomic(x.into()),
            ContentInstallParamsDto::Modpack(x) => Self::Modpack(x.into()),
        }
    }
}

impl From<ContentItem> for ContentItemDto {
    fn from(value: ContentItem) -> Self {
        Self {
            id: value.id,
            slug: value.slug,
            name: value.name,
            description: value.description,
            long_description: value.long_description,
            author: value.author,
            url: value.url,
            icon_url: value.icon_url,
            versions: value.versions,
            content_type: value.content_type.into(),
        }
    }
}

impl From<ContentItemDto> for ContentItem {
    fn from(value: ContentItemDto) -> Self {
        Self {
            id: value.id,
            slug: value.slug,
            name: value.name,
            description: value.description,
            long_description: value.long_description,
            author: value.author,
            url: value.url,
            icon_url: value.icon_url,
            versions: value.versions,
            content_type: value.content_type.into(),
        }
    }
}
