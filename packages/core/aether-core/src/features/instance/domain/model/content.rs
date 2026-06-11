use std::{ops::Deref, str::FromStr};

use crate::features::minecraft::ModLoader;

use super::ContentType;

#[derive(Clone, Debug)]
pub struct ContentSearchParams {
    pub content_type: ContentType,
    pub provider_id: ProviderId,
    pub page: i64,
    pub page_size: i64,
    pub query: Option<String>,
    pub game_versions: Option<Vec<String>>,
    pub loader: Option<ModLoader>,
}

#[derive(Clone, Debug)]
pub struct ContentSearchResult {
    pub page: i64,
    pub page_size: i64,
    pub page_count: i64,
    pub provider_id: ProviderId,
    pub items: Vec<ContentItem>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProviderId {
    pub plugin_id: String,
    pub capability_id: String,
}

impl ProviderId {
    pub const SEPARATOR: char = '#';
}

impl std::fmt::Display for ProviderId {
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

impl FromStr for ProviderId {
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

#[derive(Clone, Debug)]
pub struct BaseContentParams {
    pub content_id: String,
    pub content_version: Option<String>,
    pub provider_id: ProviderId,
}

#[derive(Clone, Debug)]
pub struct AtomicInstallParams {
    pub base: BaseContentParams,
    pub instance_id: String,
    pub game_version: String,
    pub loader: Option<String>,
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
    pub url: String,
    pub icon_url: String,
    pub versions: Vec<String>,
    pub content_type: ContentType,
}
