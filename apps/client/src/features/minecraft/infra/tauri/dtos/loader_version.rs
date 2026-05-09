use aether_core::features::minecraft::LoaderVersionPreference;
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

impl From<LoaderVersionPreferenceDto> for LoaderVersionPreference {
    fn from(value: LoaderVersionPreferenceDto) -> Self {
        match value {
            LoaderVersionPreferenceDto::Latest => Self::Latest,
            LoaderVersionPreferenceDto::Stable => Self::Stable,
            LoaderVersionPreferenceDto::Exact(x) => Self::Exact(x),
        }
    }
}

impl From<LoaderVersionPreference> for LoaderVersionPreferenceDto {
    fn from(value: LoaderVersionPreference) -> Self {
        match value {
            LoaderVersionPreference::Latest => Self::Latest,
            LoaderVersionPreference::Stable => Self::Stable,
            LoaderVersionPreference::Exact(x) => Self::Exact(x),
        }
    }
}
