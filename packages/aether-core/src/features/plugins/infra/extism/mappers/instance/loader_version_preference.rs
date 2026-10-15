use aether_core_plugin_api::v0::LoaderVersionPreferenceDto;

use crate::features::minecraft::LoaderVersionPreference;

impl From<LoaderVersionPreferenceDto> for LoaderVersionPreference {
    fn from(value: LoaderVersionPreferenceDto) -> Self {
        match value {
            LoaderVersionPreferenceDto::Latest => Self::Latest,
            LoaderVersionPreferenceDto::Stable => Self::Stable,
            LoaderVersionPreferenceDto::Exact(version) => Self::Exact(version),
        }
    }
}

impl From<LoaderVersionPreference> for LoaderVersionPreferenceDto {
    fn from(value: LoaderVersionPreference) -> Self {
        match value {
            LoaderVersionPreference::Latest => Self::Latest,
            LoaderVersionPreference::Stable => Self::Stable,
            LoaderVersionPreference::Exact(version) => Self::Exact(version),
        }
    }
}
