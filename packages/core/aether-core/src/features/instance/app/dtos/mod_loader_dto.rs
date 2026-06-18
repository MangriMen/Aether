use serde::{Deserialize, Serialize};

use crate::features::minecraft::{LoaderVersionPreference, ModLoader};

/// App-layer serializable DTO for ModLoader.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModLoaderDto {
    Vanilla,
    Forge,
    Fabric,
    Quilt,
    NeoForge,
}

impl From<ModLoaderDto> for ModLoader {
    fn from(value: ModLoaderDto) -> Self {
        match value {
            ModLoaderDto::Vanilla => Self::Vanilla,
            ModLoaderDto::Forge => Self::Forge,
            ModLoaderDto::Fabric => Self::Fabric,
            ModLoaderDto::Quilt => Self::Quilt,
            ModLoaderDto::NeoForge => Self::NeoForge,
        }
    }
}

impl From<ModLoader> for ModLoaderDto {
    fn from(value: ModLoader) -> Self {
        match value {
            ModLoader::Vanilla => Self::Vanilla,
            ModLoader::Forge => Self::Forge,
            ModLoader::Fabric => Self::Fabric,
            ModLoader::Quilt => Self::Quilt,
            ModLoader::NeoForge => Self::NeoForge,
        }
    }
}

/// App-layer serializable DTO for LoaderVersionPreference.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum LoaderVersionPreferenceDto {
    Latest,
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
