use aether_core::features::minecraft::{InvalidLoaderVersionId, LoaderVersionPreference};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Serialize, Deserialize, Clone, Default, Type)]
#[serde(rename_all = "snake_case")]
pub enum LoaderVersionPreferenceDto {
    Latest,
    #[default]
    Stable,
    #[serde(untagged)]
    Exact(String),
}

impl TryFrom<LoaderVersionPreferenceDto> for LoaderVersionPreference {
    type Error = InvalidLoaderVersionId;

    fn try_from(value: LoaderVersionPreferenceDto) -> Result<Self, Self::Error> {
        Ok(match value {
            LoaderVersionPreferenceDto::Latest => Self::Latest,
            LoaderVersionPreferenceDto::Stable => Self::Stable,
            LoaderVersionPreferenceDto::Exact(id) => Self::Exact(id.try_into()?),
        })
    }
}

impl From<LoaderVersionPreference> for LoaderVersionPreferenceDto {
    fn from(value: LoaderVersionPreference) -> Self {
        match value {
            LoaderVersionPreference::Latest => Self::Latest,
            LoaderVersionPreference::Stable => Self::Stable,
            LoaderVersionPreference::Exact(id) => Self::Exact(id.into()),
        }
    }
}
