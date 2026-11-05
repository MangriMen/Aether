use std::str::FromStr;

use aether_core::features::{
    instance::{
        AtomicInstallParams, BaseContentParams, ContentInstallParams, ContentItem,
        ContentSearchParams, ContentSearchResult, InstanceError, InstanceId, ModpackInstallParams,
        ProviderId,
    },
    minecraft::GameVersion,
};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use specta::Type;

use crate::features::{instance::infra::ContentTypeDto, minecraft::infra::ModLoaderDto};

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentSearchParamsDto {
    pub content_type: ContentTypeDto,

    pub provider_id: ProviderIdDto,

    #[specta(type = String)]
    #[serde_as(as = "DisplayFromStr")]
    pub page: i64,

    #[specta(type = String)]
    #[serde_as(as = "DisplayFromStr")]
    pub page_size: i64,

    #[specta(optional)]
    pub query: Option<String>,

    #[specta(optional)]
    pub game_versions: Option<Vec<String>>,

    #[specta(optional)]
    pub loader: Option<ModLoaderDto>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentSearchResultDto {
    #[specta(type = String)]
    #[serde_as(as = "DisplayFromStr")]
    pub page: i64,

    #[specta(type = String)]
    #[serde_as(as = "DisplayFromStr")]
    pub page_size: i64,

    #[specta(type = String)]
    #[serde_as(as = "DisplayFromStr")]
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
    pub loader: Option<ModLoaderDto>,
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

impl TryFrom<ContentSearchParamsDto> for ContentSearchParams {
    type Error = InstanceError;

    fn try_from(value: ContentSearchParamsDto) -> Result<Self, Self::Error> {
        Ok(Self {
            content_type: value.content_type.into(),
            provider_id: value.provider_id.try_into()?,
            page: value.page,
            page_size: value.page_size,
            query: value.query,
            game_versions: value
                .game_versions
                .map(|versions| {
                    versions
                        .into_iter()
                        .map(GameVersion::try_from)
                        .collect::<Result<_, _>>()
                })
                .transpose()?,
            loader: value.loader.map(Into::into),
        })
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

/// An id that does not parse names no provider, hence [`InstanceError::ContentProviderNotFound`].
impl TryFrom<ProviderIdDto> for ProviderId {
    type Error = InstanceError;

    fn try_from(value: ProviderIdDto) -> Result<Self, Self::Error> {
        Self::from_parts(&value.plugin_id, &value.capability_id).map_err(|_| {
            InstanceError::ContentProviderNotFound {
                plugin_id: value.plugin_id,
                capability_id: value.capability_id,
            }
        })
    }
}

impl From<ProviderId> for ProviderIdDto {
    fn from(value: ProviderId) -> Self {
        Self {
            plugin_id: value.plugin_id.into(),
            capability_id: value.capability_id.into(),
        }
    }
}

impl TryFrom<BaseContentParamsDto> for BaseContentParams {
    type Error = InstanceError;

    fn try_from(value: BaseContentParamsDto) -> Result<Self, Self::Error> {
        Ok(Self {
            content_id: value.content_id.try_into()?,
            content_version: value.content_version.map(TryInto::try_into).transpose()?,
            provider_id: value.provider_id.try_into()?,
        })
    }
}

impl TryFrom<AtomicInstallParamsDto> for AtomicInstallParams {
    type Error = InstanceError;

    fn try_from(value: AtomicInstallParamsDto) -> Result<Self, Self::Error> {
        Ok(Self {
            base: value.base.try_into()?,
            instance_id: InstanceId::try_from(value.instance_id)?,
            game_version: GameVersion::try_from(value.game_version)?,
            loader: value.loader.map(Into::into),
            content_type: value.content_type.into(),
        })
    }
}

impl TryFrom<ModpackInstallParamsDto> for ModpackInstallParams {
    type Error = InstanceError;

    fn try_from(value: ModpackInstallParamsDto) -> Result<Self, Self::Error> {
        Ok(Self {
            base: value.base.try_into()?,
        })
    }
}

impl TryFrom<ContentInstallParamsDto> for ContentInstallParams {
    type Error = InstanceError;

    fn try_from(value: ContentInstallParamsDto) -> Result<Self, Self::Error> {
        Ok(match value {
            ContentInstallParamsDto::Atomic(x) => Self::Atomic(x.try_into()?),
            ContentInstallParamsDto::Modpack(x) => Self::Modpack(x.try_into()?),
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    fn base(content_id: &str, content_version: Option<&str>) -> BaseContentParamsDto {
        BaseContentParamsDto {
            content_id: content_id.into(),
            content_version: content_version.map(Into::into),
            provider_id: ProviderIdDto {
                plugin_id: "aether".into(),
                capability_id: "modrinth".into(),
            },
        }
    }

    #[test]
    fn should_refuse_a_content_id_when_it_is_not_a_valid_id() {
        for bad in ["", " a", "a/b", "a%2e"] {
            let result = BaseContentParams::try_from(base(bad, None));

            assert!(
                matches!(result, Err(InstanceError::InvalidContentId { .. })),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn should_refuse_a_content_version_when_it_is_not_a_valid_id() {
        let result = BaseContentParams::try_from(base("AANobbMI", Some("v?1")));

        assert!(matches!(
            result,
            Err(InstanceError::InvalidContentId { .. })
        ));
    }

    #[test]
    fn should_accept_the_ids_of_a_provider_when_they_are_valid() {
        let params = BaseContentParams::try_from(base("AANobbMI", Some("IZskON6d"))).unwrap();

        assert_eq!(params.content_id.as_str(), "AANobbMI");
        assert_eq!(params.content_version.unwrap().as_str(), "IZskON6d");
    }

    fn search(game_versions: &[&str]) -> ContentSearchParamsDto {
        ContentSearchParamsDto {
            content_type: ContentTypeDto::Mod,
            provider_id: base("AANobbMI", None).provider_id,
            page: 1,
            page_size: 20,
            query: None,
            game_versions: Some(game_versions.iter().map(|v| (*v).to_owned()).collect()),
            loader: None,
        }
    }

    #[test]
    fn should_refuse_the_search_when_a_game_version_is_not_a_valid_id() {
        let result = ContentSearchParams::try_from(search(&["1.21.1", "../x"]));

        assert!(matches!(
            result,
            Err(InstanceError::InvalidGameVersion { ref value, .. }) if value == "../x"
        ));
    }

    #[test]
    fn should_keep_the_game_versions_of_the_search_when_they_are_valid() {
        let params = ContentSearchParams::try_from(search(&["1.21.1", "24w14a"])).unwrap();

        assert_eq!(params.game_versions.unwrap(), ["1.21.1", "24w14a"]);
    }

    #[test]
    fn should_refuse_an_install_when_its_game_version_is_not_a_valid_id() {
        let result = AtomicInstallParams::try_from(AtomicInstallParamsDto {
            base: base("AANobbMI", None),
            instance_id: "instance".into(),
            game_version: String::new(),
            loader: None,
            content_type: ContentTypeDto::Mod,
        });

        assert!(matches!(
            result,
            Err(InstanceError::InvalidGameVersion { .. })
        ));
    }
}
