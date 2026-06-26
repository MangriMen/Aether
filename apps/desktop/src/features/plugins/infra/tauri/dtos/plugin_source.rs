use serde::{Deserialize, Serialize};
use specta::Type;

use super::ProviderReleaseInfoDto;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum PluginSourceDto {
    Remote {
        source_type: PluginSourceTypeDto,
        identifier: String,
        current_tag: String,
        current_version: String,
    },
    Local,
}

impl From<aether_core::features::plugins::PluginSource> for PluginSourceDto {
    fn from(value: aether_core::features::plugins::PluginSource) -> Self {
        match value {
            aether_core::features::plugins::PluginSource::Remote {
                source_type,
                identifier,
                current_tag,
                current_version,
            } => Self::Remote {
                source_type: source_type.into(),
                identifier,
                current_tag,
                current_version,
            },
            aether_core::features::plugins::PluginSource::Local => Self::Local,
        }
    }
}

/// Provider source type for plugin installation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum PluginSourceTypeDto {
    GitHub,
    Local,
}

impl From<aether_core::features::plugins::PluginSourceType> for PluginSourceTypeDto {
    fn from(value: aether_core::features::plugins::PluginSourceType) -> Self {
        match value {
            aether_core::features::plugins::PluginSourceType::GitHub => Self::GitHub,
            aether_core::features::plugins::PluginSourceType::Local => Self::Local,
        }
    }
}

impl From<PluginSourceTypeDto> for aether_core::features::plugins::PluginSourceType {
    fn from(value: PluginSourceTypeDto) -> Self {
        match value {
            PluginSourceTypeDto::GitHub => Self::GitHub,
            PluginSourceTypeDto::Local => Self::Local,
        }
    }
}

/// Provider-agnostic plugin update info.
/// Uses `ProviderReleaseInfoDto` for releases so it works with any source type.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PluginUpdateInfoDto {
    pub current_version: String,
    pub current_tag: String,
    pub latest_version: Option<String>,
    pub latest_tag: Option<String>,
    pub has_update: bool,
    pub all_releases: Vec<ProviderReleaseInfoDto>,
}

impl From<aether_core::features::plugins::ProviderUpdateInfo> for PluginUpdateInfoDto {
    fn from(value: aether_core::features::plugins::ProviderUpdateInfo) -> Self {
        Self {
            current_version: value.current_version,
            current_tag: value.current_tag,
            latest_version: value.latest_version,
            latest_tag: value.latest_tag,
            has_update: value.has_update,
            all_releases: value
                .all_releases
                .into_iter()
                .map(|r| ProviderReleaseInfoDto {
                    tag_name: r.tag_name,
                    version: r.version,
                    is_prerelease: r.is_prerelease,
                    published_at: r.published_at,
                    html_url: r.html_url,
                    download_url: r.download_url,
                })
                .collect(),
        }
    }
}
