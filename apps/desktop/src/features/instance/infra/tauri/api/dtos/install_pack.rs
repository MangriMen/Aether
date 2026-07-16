use aether_core::features::instance::{InstallPackRequest, PackInstallParams, PackSource};
use serde::{Deserialize, Serialize};
use specta::Type;

use super::{NewInstanceDto, ProviderIdDto};

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct InstallPackRequestDto {
    #[specta(optional)]
    pub new_instance: Option<NewInstanceDto>,
    pub pack_source: PackSourceDto,
    #[specta(optional)]
    pub pack_version: Option<String>,
    pub provider_id: ProviderIdDto,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct PackSourceDto {
    #[specta(optional)]
    pub provider_id: Option<ProviderIdDto>,
    #[specta(optional)]
    pub pack_id: Option<String>,
    #[specta(optional)]
    pub version_id: Option<String>,
    #[specta(optional)]
    pub local_file: Option<String>,
    #[specta(optional)]
    pub remote_url: Option<String>,
    /// Universal source string (path or URL).
    ///
    /// When set, the converter auto-detects the type:
    /// - contains `://` → `PackSource::RemoteUrl`
    /// - otherwise      → `PackSource::LocalFile`
    #[specta(optional)]
    pub source: Option<String>,
}

impl From<PackSourceDto> for PackSource {
    fn from(value: PackSourceDto) -> Self {
        // 1. Registry — explicit three-field combo
        if let (Some(provider_id), Some(pack_id), Some(version_id)) =
            (value.provider_id, value.pack_id, value.version_id)
        {
            return Self::Registry {
                provider_id: provider_id.into(),
                pack_id,
                version_id,
            };
        }

        // 2. Universal source — auto-detect URL vs file
        if let Some(source) = value.source {
            return if source.contains("://") {
                Self::RemoteUrl(source)
            } else {
                Self::LocalFile(source)
            };
        }

        // 3. Legacy fields — backward compatibility
        if let Some(remote_url) = value.remote_url {
            return Self::RemoteUrl(remote_url);
        }
        if let Some(local_file) = value.local_file {
            return Self::LocalFile(local_file);
        }

        // 4. Fallback — empty remote URL (shouldn't happen in practice)
        Self::RemoteUrl(String::new())
    }
}

impl From<InstallPackRequestDto> for InstallPackRequest {
    fn from(value: InstallPackRequestDto) -> Self {
        Self {
            new_instance: value.new_instance.map(Into::into),
            pack_params: PackInstallParams {
                source: value.pack_source.into(),
                version: value.pack_version,
            },
            provider_id: value.provider_id.into(),
        }
    }
}
