use std::sync::Arc;

use crate::features::settings::{AppSettings, AppSettingsError, AppSettingsStorage};

pub struct GetAppSettingsUseCase<ASS: AppSettingsStorage> {
    app_settings_storage: Arc<ASS>,
}

impl<ASS: AppSettingsStorage> GetAppSettingsUseCase<ASS> {
    pub fn new(app_settings_storage: Arc<ASS>) -> Self {
        Self {
            app_settings_storage,
        }
    }

    pub async fn execute(&self) -> Result<AppSettings, AppSettingsError> {
        self.app_settings_storage.get().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::settings::{ActionOnInstanceLaunch, WindowEffect};
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockStorage {
        settings: AppSettings,
        get_call_count: AtomicUsize,
    }

    #[async_trait]
    impl AppSettingsStorage for MockStorage {
        async fn get(&self) -> Result<AppSettings, AppSettingsError> {
            self.get_call_count.fetch_add(1, Ordering::SeqCst);
            Ok(self.settings)
        }

        async fn upsert(&self, _settings: AppSettings) -> Result<(), AppSettingsError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn get_app_settings_returns_storage_value() {
        let storage = Arc::new(MockStorage {
            settings: AppSettings {
                action_on_instance_launch: ActionOnInstanceLaunch::Close,
                is_actual_transparent: false,
                transparent: true,
                window_effect: WindowEffect::Mica,
            },
            get_call_count: AtomicUsize::new(0),
        });

        let use_case = GetAppSettingsUseCase::new(storage.clone());
        let result = use_case.execute().await.unwrap();

        assert_eq!(
            result.action_on_instance_launch,
            ActionOnInstanceLaunch::Close
        );
        assert!(result.transparent);
        assert_eq!(result.window_effect, WindowEffect::Mica);
        assert_eq!(storage.get_call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn get_app_settings_default_values() {
        let storage = Arc::new(MockStorage {
            settings: AppSettings::default(),
            get_call_count: AtomicUsize::new(0),
        });

        let use_case = GetAppSettingsUseCase::new(storage);
        let result = use_case.execute().await.unwrap();

        assert_eq!(
            result.action_on_instance_launch,
            ActionOnInstanceLaunch::Nothing
        );
        assert!(!result.transparent);
        assert_eq!(result.window_effect, WindowEffect::Off);
    }
}
