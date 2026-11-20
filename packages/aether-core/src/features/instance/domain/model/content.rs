use std::ops::Deref;

use crate::{
    features::minecraft::{GameVersion, ModLoader},
    shared::web_url::{HttpUrl, HttpsUrl},
};

use super::{
    ContentId, ContentType, ContentVersionId, InstanceId, PageIndex, PageSize, ProviderId,
};

#[derive(Clone, Debug)]
pub struct ContentSearchParams {
    pub content_type: ContentType,
    pub provider_id: ProviderId,
    pub page: PageIndex,
    pub page_size: PageSize,
    pub query: Option<String>,
    pub game_versions: Option<Vec<GameVersion>>,
    pub loader: Option<ModLoader>,
}

#[derive(Clone, Debug)]
pub struct ContentSearchResult {
    pub page: PageIndex,
    pub page_size: PageSize,
    pub page_count: u32,
    pub provider_id: ProviderId,
    pub items: Vec<ContentItem>,
}

#[derive(Clone, Debug)]
pub struct BaseContentParams {
    pub content_id: ContentId,
    pub content_version: Option<ContentVersionId>,
    pub provider_id: ProviderId,
}

#[derive(Clone, Debug)]
pub struct AtomicInstallParams {
    pub base: BaseContentParams,
    pub instance_id: InstanceId,
    pub game_version: GameVersion,
    pub loader: Option<ModLoader>,
    pub content_type: ContentType,
}

impl Deref for AtomicInstallParams {
    type Target = BaseContentParams;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

#[derive(Clone, Debug)]
pub struct ModpackInstallParams {
    pub base: BaseContentParams,
}

impl Deref for ModpackInstallParams {
    type Target = BaseContentParams;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

#[derive(Clone, Debug)]
pub enum ContentInstallParams {
    Atomic(AtomicInstallParams),
    Modpack(ModpackInstallParams),
}

impl ContentInstallParams {
    pub fn provider(&self) -> &ProviderId {
        match self {
            Self::Atomic(p) => &p.base.provider_id,
            Self::Modpack(p) => &p.base.provider_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ContentItem {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub long_description: Option<String>,
    pub author: String,
    /// The content's page, to link to.
    pub url: Option<HttpUrl>,
    pub icon_url: Option<HttpsUrl>,
    pub versions: Vec<String>,
    pub content_type: ContentType,
}
