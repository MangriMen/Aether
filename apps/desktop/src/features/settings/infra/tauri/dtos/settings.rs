use std::collections::HashMap;

use aether_core::features::{
    instance::PlatformId,
    settings::{EditPlatformProvider, EditSettings, Settings, SettingsError},
};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use specta::Type;

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    #[specta(type = String)]
    #[serde_as(as = "DisplayFromStr")]
    max_concurrent_downloads: usize,

    /// Platform → id of the provider chosen to serve it.
    platform_providers: HashMap<String, String>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditSettingsDto {
    #[specta(type = String)]
    #[serde_as(as = "DisplayFromStr")]
    pub max_concurrent_downloads: usize,

    /// Choose the provider serving a platform; absent leaves the choices as they are.
    #[serde(default)]
    pub platform_provider: Option<EditPlatformProviderDto>,
}

#[derive(Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditPlatformProviderDto {
    pub platform: String,

    /// `None` drops the choice, so the platform falls back to the default rule.
    pub provider_id: Option<String>,
}

impl From<Settings> for SettingsDto {
    fn from(value: Settings) -> Self {
        Self {
            max_concurrent_downloads: value.max_concurrent_downloads(),
            platform_providers: value
                .platform_providers()
                .iter()
                .map(|(platform, provider)| (platform.to_string(), provider.clone()))
                .collect(),
        }
    }
}
impl TryFrom<EditSettingsDto> for EditSettings {
    type Error = SettingsError;

    fn try_from(value: EditSettingsDto) -> Result<Self, Self::Error> {
        let platform_provider = value
            .platform_provider
            .map(|choice| {
                let platform = PlatformId::try_from(choice.platform.as_str()).map_err(|err| {
                    SettingsError::InvalidPlatformProvider {
                        platform: choice.platform.clone(),
                        reason: err.to_string(),
                    }
                })?;
                Ok::<_, SettingsError>(EditPlatformProvider {
                    platform,
                    provider_id: choice.provider_id,
                })
            })
            .transpose()?;

        Ok(Self {
            max_concurrent_downloads: value.max_concurrent_downloads,
            platform_provider,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edit(platform: &str) -> EditSettingsDto {
        EditSettingsDto {
            max_concurrent_downloads: 10,
            platform_provider: Some(EditPlatformProviderDto {
                platform: platform.to_owned(),
                provider_id: None,
            }),
        }
    }

    #[test]
    fn should_refuse_a_platform_when_it_is_not_a_platform_id() {
        for platform in ["", "Modrinth"] {
            let result = EditSettings::try_from(edit(platform));

            assert!(matches!(
                result,
                Err(SettingsError::InvalidPlatformProvider { platform: ref got, .. })
                    if got == platform
            ));
        }
    }

    #[test]
    fn should_accept_a_platform_when_it_is_a_platform_id() {
        let result = EditSettings::try_from(edit("modrinth")).unwrap();

        assert_eq!(
            result.platform_provider.unwrap().platform.as_str(),
            "modrinth"
        );
    }
}
