use std::sync::Arc;

use async_trait::async_trait;

use crate::features::{
    instance::{InstanceError, PlatformProviderSelection, ProviderId},
    settings::SettingsStorage,
};

/// The provider of each platform as the user chose it in the launcher settings.
pub struct SettingsPlatformProviderSelection {
    settings_storage: Arc<dyn SettingsStorage>,
}

impl SettingsPlatformProviderSelection {
    pub fn new(settings_storage: Arc<dyn SettingsStorage>) -> Self {
        Self { settings_storage }
    }
}

#[async_trait]
impl PlatformProviderSelection for SettingsPlatformProviderSelection {
    async fn selected_provider(&self, platform: &str) -> Result<Option<ProviderId>, InstanceError> {
        let settings = self.settings_storage.get().await.map_err(|err| {
            tracing::error!("Can not read the platform provider choice: {err}");
            InstanceError::CapabilityOperationError
        })?;

        // A stored id that no longer parses is treated as a choice that cannot be satisfied, not
        // as no choice: falling through would quietly hand the platform to another provider.
        settings
            .platform_providers()
            .get(platform)
            .map(|id| {
                id.parse()
                    .map_err(|_| InstanceError::PlatformProviderNotChosen {
                        platform: platform.to_owned(),
                    })
            })
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::Mutex;

    use super::*;
    use crate::features::settings::{Settings, SettingsError};

    struct FixedSettings(Mutex<Settings>);

    #[async_trait]
    impl SettingsStorage for FixedSettings {
        async fn get(&self) -> Result<Settings, SettingsError> {
            Ok(self.0.lock().await.clone())
        }

        async fn upsert(&self, settings: Settings) -> Result<Settings, SettingsError> {
            Ok(settings)
        }

        async fn update_mut(
            &self,
            f: Box<dyn FnOnce(Settings) -> (Settings, bool) + Send>,
        ) -> Result<Settings, SettingsError> {
            Ok(f(self.get().await?).0)
        }
    }

    fn selection(choices: &[(&str, &str)]) -> SettingsPlatformProviderSelection {
        let mut settings = Settings::default();
        for (platform, provider) in choices {
            settings.set_platform_provider(platform, Some(provider));
        }
        SettingsPlatformProviderSelection::new(Arc::new(FixedSettings(Mutex::new(settings))))
    }

    #[tokio::test]
    async fn should_return_the_chosen_provider_when_the_platform_has_a_choice() {
        let selection = selection(&[("curseforge", "cf-plugin#curseforge")]);

        let chosen = selection.selected_provider("curseforge").await.unwrap();

        assert_eq!(chosen, Some("cf-plugin#curseforge".parse().unwrap()));
    }

    #[tokio::test]
    async fn should_return_nothing_when_the_platform_has_no_choice() {
        let selection = selection(&[("curseforge", "cf-plugin#curseforge")]);

        assert_eq!(selection.selected_provider("modrinth").await.unwrap(), None);
    }

    #[tokio::test]
    async fn should_refuse_a_stored_choice_when_it_is_not_a_provider_id() {
        let selection = selection(&[("curseforge", "not a provider id")]);

        let result = selection.selected_provider("curseforge").await;

        assert!(matches!(
            result,
            Err(InstanceError::PlatformProviderNotChosen { .. })
        ));
    }
}
