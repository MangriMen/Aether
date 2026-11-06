use aether_core_plugin_api::v0::LoaderVersionPreferenceDto;

use crate::features::minecraft::{InvalidLoaderVersionId, LoaderVersionPreference};

impl TryFrom<LoaderVersionPreferenceDto> for LoaderVersionPreference {
    type Error = InvalidLoaderVersionId;

    fn try_from(value: LoaderVersionPreferenceDto) -> Result<Self, Self::Error> {
        Ok(match value {
            LoaderVersionPreferenceDto::Latest => Self::Latest,
            LoaderVersionPreferenceDto::Stable => Self::Stable,
            LoaderVersionPreferenceDto::Exact(version) => Self::Exact(version.try_into()?),
        })
    }
}

impl From<LoaderVersionPreference> for LoaderVersionPreferenceDto {
    fn from(value: LoaderVersionPreference) -> Self {
        match value {
            LoaderVersionPreference::Latest => Self::Latest,
            LoaderVersionPreference::Stable => Self::Stable,
            LoaderVersionPreference::Exact(version) => Self::Exact(version.into()),
        }
    }
}
