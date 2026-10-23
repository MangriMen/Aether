use std::sync::Arc;

use async_trait::async_trait;

use crate::features::settings::{
    Settings, SettingsError, SettingsStorage,
    app::{EditSettings, ports::EditSettingsUseCasePort},
};

pub struct EditSettingsUseCase {
    settings_storage: Arc<dyn SettingsStorage>,
}

impl EditSettingsUseCase {
    pub fn new(settings_storage: Arc<dyn SettingsStorage>) -> Self {
        Self { settings_storage }
    }
}

#[async_trait]
impl EditSettingsUseCasePort for EditSettingsUseCase {
    async fn execute(&self, edit_settings: EditSettings) -> Result<Settings, SettingsError> {
        edit_settings.validate()?;

        self.settings_storage
            .update_mut(Box::new(move |mut settings| {
                let changed = edit_settings.apply_to(&mut settings);
                (settings, changed)
            }))
            .await
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::Mutex;

    use super::*;
    use crate::features::settings::app::EditPlatformProvider;

    struct FixedSettings(Mutex<Settings>);

    #[async_trait]
    impl SettingsStorage for FixedSettings {
        async fn get(&self) -> Result<Settings, SettingsError> {
            Ok(self.0.lock().await.clone())
        }

        async fn upsert(&self, settings: Settings) -> Result<Settings, SettingsError> {
            *self.0.lock().await = settings.clone();
            Ok(settings)
        }

        async fn update_mut(
            &self,
            f: Box<dyn FnOnce(Settings) -> (Settings, bool) + Send>,
        ) -> Result<Settings, SettingsError> {
            let mut guard = self.0.lock().await;
            let (updated, _) = f(guard.clone());
            *guard = updated.clone();
            Ok(updated)
        }
    }

    fn edit(platform: &str, provider_id: &str) -> EditSettings {
        EditSettings {
            max_concurrent_downloads: 10,
            platform_provider: Some(EditPlatformProvider {
                platform: platform.to_owned(),
                provider_id: Some(provider_id.to_owned()),
            }),
        }
    }

    async fn run(edit_settings: EditSettings) -> (Result<Settings, SettingsError>, Settings) {
        let storage = Arc::new(FixedSettings(Mutex::new(Settings::default())));
        let result = EditSettingsUseCase::new(storage.clone())
            .execute(edit_settings)
            .await;
        let stored = storage.get().await.unwrap();
        (result, stored)
    }

    #[tokio::test]
    async fn should_refuse_zero_concurrent_downloads() {
        let mut edit_settings = edit("curseforge", "cf-plugin#curseforge");
        edit_settings.max_concurrent_downloads = 0;

        let (result, stored) = run(edit_settings).await;

        assert!(matches!(
            result,
            Err(SettingsError::InvalidMaxConcurrentDownloads)
        ));
        assert_eq!(
            stored.max_concurrent_downloads(),
            Settings::default().max_concurrent_downloads()
        );
    }

    #[tokio::test]
    async fn should_refuse_a_provider_choice_when_the_platform_is_empty() {
        let (result, stored) = run(edit("", "cf-plugin#curseforge")).await;

        assert!(matches!(
            result,
            Err(SettingsError::InvalidPlatformProvider { ref platform, .. }) if platform.is_empty()
        ));
        assert!(stored.platform_providers().is_empty());
    }

    #[tokio::test]
    async fn should_refuse_a_provider_choice_when_the_provider_id_is_malformed() {
        let (result, stored) = run(edit("curseforge", "not a provider id")).await;

        assert!(matches!(
            result,
            Err(SettingsError::InvalidPlatformProvider { ref platform, ref reason })
                if platform == "curseforge" && reason.contains("not a provider id")
        ));
        assert!(stored.platform_providers().is_empty());
    }

    #[tokio::test]
    async fn should_save_a_provider_choice_when_it_is_valid() {
        let (result, stored) = run(edit("curseforge", "cf-plugin#curseforge")).await;

        assert!(result.is_ok());
        assert_eq!(
            stored
                .platform_providers()
                .get("curseforge")
                .map(String::as_str),
            Some("cf-plugin#curseforge")
        );
    }
}
