use std::sync::Arc;

use crate::{
    core::{WindowLabel, WindowManager},
    features::settings::{
        AppSettings, AppSettingsError, AppSettingsStorage, EditAppSettingsRequest,
    },
};

pub struct EditAppSettingsUseCase<ASS: AppSettingsStorage, WM: WindowManager> {
    app_settings_storage: Arc<ASS>,
    window_manager: Arc<WM>,
}

impl<ASS: AppSettingsStorage, WM: WindowManager> EditAppSettingsUseCase<ASS, WM> {
    pub fn new(app_settings_storage: Arc<ASS>, window_manager: Arc<WM>) -> Self {
        Self {
            app_settings_storage,
            window_manager,
        }
    }

    pub async fn execute(
        &self,
        edit_app_settings: EditAppSettingsRequest,
    ) -> Result<AppSettings, AppSettingsError> {
        let old_settings = self.app_settings_storage.get().await?;
        let mut new_settings = old_settings;

        if let Some(action_on_instance_launch) = edit_app_settings.action_on_instance_launch {
            new_settings.action_on_instance_launch = action_on_instance_launch;
        }

        if let Some(transparent) = edit_app_settings.transparent {
            new_settings.transparent = transparent;
        }

        if let Some(window_effect) = edit_app_settings.window_effect {
            new_settings.window_effect = window_effect;
        }

        self.apply_effect(&new_settings, &old_settings).await?;

        self.app_settings_storage.upsert(new_settings).await?;

        Ok(new_settings)
    }

    async fn apply_effect(
        &self,
        new_settings: &AppSettings,
        old_settings: &AppSettings,
    ) -> Result<(), AppSettingsError> {
        let is_transparent_or_was_it = new_settings.transparent || old_settings.transparent;
        let is_effect_equals = new_settings.window_effect == old_settings.window_effect;

        if is_transparent_or_was_it && !is_effect_equals {
            self.window_manager
                .apply_visual_effect(WindowLabel::Main, new_settings.window_effect)
                .await
                .map_err(|err| AppSettingsError::CanNotSetEffect(err.to_string()))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::WindowError;
    use crate::features::settings::{ActionOnInstanceLaunch, WindowEffect};
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    // ── Mock Storage ──

    #[derive(Clone)]
    struct MockStorage {
        stored: Arc<std::sync::Mutex<AppSettings>>,
        upsert_call_count: Arc<AtomicUsize>,
    }

    impl MockStorage {
        fn new(settings: AppSettings) -> Self {
            Self {
                stored: Arc::new(std::sync::Mutex::new(settings)),
                upsert_call_count: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    #[async_trait]
    impl AppSettingsStorage for MockStorage {
        async fn get(&self) -> Result<AppSettings, AppSettingsError> {
            Ok(*self.stored.lock().unwrap())
        }

        async fn upsert(&self, settings: AppSettings) -> Result<(), AppSettingsError> {
            *self.stored.lock().unwrap() = settings;
            self.upsert_call_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    // ── Mock Window Manager ──

    #[derive(Clone)]
    struct MockWindowManager {
        apply_effect_called: Arc<AtomicBool>,
        apply_effect_label: Arc<std::sync::Mutex<Option<WindowLabel>>>,
        apply_effect_effect: Arc<std::sync::Mutex<Option<WindowEffect>>>,
        should_fail: bool,
    }

    impl MockWindowManager {
        fn new(should_fail: bool) -> Self {
            Self {
                apply_effect_called: Arc::new(AtomicBool::new(false)),
                apply_effect_label: Arc::new(std::sync::Mutex::new(None)),
                apply_effect_effect: Arc::new(std::sync::Mutex::new(None)),
                should_fail,
            }
        }
    }

    #[async_trait]
    impl WindowManager for MockWindowManager {
        async fn apply_visual_effect(
            &self,
            label: WindowLabel,
            effect: WindowEffect,
        ) -> Result<(), WindowError> {
            self.apply_effect_called.store(true, Ordering::SeqCst);
            *self.apply_effect_label.lock().unwrap() = Some(label);
            *self.apply_effect_effect.lock().unwrap() = Some(effect);
            if self.should_fail {
                Err(WindowError::Other("mock failure".to_string()))
            } else {
                Ok(())
            }
        }

        async fn hide_window(&self, _label: WindowLabel) -> Result<(), WindowError> {
            Ok(())
        }

        async fn show_window(&self, _label: WindowLabel) -> Result<(), WindowError> {
            Ok(())
        }

        async fn close_window(&self, _label: WindowLabel) -> Result<(), WindowError> {
            Ok(())
        }

        async fn close_window_and_wait(&self, _label: WindowLabel) -> Result<(), WindowError> {
            Ok(())
        }

        async fn create_window(
            &self,
            _label: WindowLabel,
            _transparent: bool,
            _window_effect: WindowEffect,
        ) -> Result<(), WindowError> {
            Ok(())
        }
    }

    // ── Tests ──

    #[tokio::test]
    async fn edit_no_changes_returns_same_settings() {
        let storage = Arc::new(MockStorage::new(AppSettings::default()));
        let window = Arc::new(MockWindowManager::new(false));
        let use_case = EditAppSettingsUseCase::new(storage.clone(), window.clone());

        let request = EditAppSettingsRequest {
            action_on_instance_launch: None,
            transparent: None,
            window_effect: None,
        };

        let result = use_case.execute(request).await.unwrap();
        assert_eq!(
            result.action_on_instance_launch,
            ActionOnInstanceLaunch::Nothing
        );
        assert!(!window.apply_effect_called.load(Ordering::SeqCst));
        assert_eq!(storage.upsert_call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn edit_changes_action_on_instance_launch() {
        let storage = Arc::new(MockStorage::new(AppSettings::default()));
        let window = Arc::new(MockWindowManager::new(false));
        let use_case = EditAppSettingsUseCase::new(storage.clone(), window.clone());

        let request = EditAppSettingsRequest {
            action_on_instance_launch: Some(ActionOnInstanceLaunch::Hide),
            transparent: None,
            window_effect: None,
        };

        let result = use_case.execute(request).await.unwrap();
        assert_eq!(
            result.action_on_instance_launch,
            ActionOnInstanceLaunch::Hide
        );
    }

    #[tokio::test]
    async fn edit_changes_transparent() {
        let storage = Arc::new(MockStorage::new(AppSettings::default()));
        let window = Arc::new(MockWindowManager::new(false));
        let use_case = EditAppSettingsUseCase::new(storage.clone(), window.clone());

        let request = EditAppSettingsRequest {
            action_on_instance_launch: None,
            transparent: Some(true),
            window_effect: None,
        };

        let result = use_case.execute(request).await.unwrap();
        assert!(result.transparent);
        assert!(!result.is_actual_transparent);
    }

    #[tokio::test]
    async fn edit_transparent_and_effect_applies_visual_effect() {
        let storage = Arc::new(MockStorage::new(AppSettings::default()));
        let window = Arc::new(MockWindowManager::new(false));
        let use_case = EditAppSettingsUseCase::new(storage.clone(), window.clone());

        let request = EditAppSettingsRequest {
            action_on_instance_launch: None,
            transparent: Some(true),
            window_effect: Some(WindowEffect::Acrylic),
        };

        let result = use_case.execute(request).await.unwrap();
        assert!(result.transparent);
        assert_eq!(result.window_effect, WindowEffect::Acrylic);
        assert!(
            window.apply_effect_called.load(Ordering::SeqCst),
            "apply_visual_effect should have been called"
        );
    }

    #[tokio::test]
    async fn edit_effect_changes_when_transparent_calls_apply() {
        let initial = AppSettings {
            action_on_instance_launch: ActionOnInstanceLaunch::Nothing,
            is_actual_transparent: true,
            transparent: true,
            window_effect: WindowEffect::Off,
        };
        let storage = Arc::new(MockStorage::new(initial));
        let window = Arc::new(MockWindowManager::new(false));
        let use_case = EditAppSettingsUseCase::new(storage.clone(), window.clone());

        let request = EditAppSettingsRequest {
            action_on_instance_launch: None,
            transparent: None,
            window_effect: Some(WindowEffect::Mica),
        };

        let result = use_case.execute(request).await.unwrap();
        assert_eq!(result.window_effect, WindowEffect::Mica);
        assert!(window.apply_effect_called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn edit_same_effect_not_applied_when_transparent() {
        let initial = AppSettings {
            action_on_instance_launch: ActionOnInstanceLaunch::Nothing,
            is_actual_transparent: false,
            transparent: true,
            window_effect: WindowEffect::Mica,
        };
        let storage = Arc::new(MockStorage::new(initial));
        let window = Arc::new(MockWindowManager::new(false));
        let use_case = EditAppSettingsUseCase::new(storage.clone(), window.clone());

        let request = EditAppSettingsRequest {
            action_on_instance_launch: None,
            transparent: None,
            window_effect: Some(WindowEffect::Mica),
        };

        let _ = use_case.execute(request).await.unwrap();
        assert!(
            !window.apply_effect_called.load(Ordering::SeqCst),
            "apply should NOT be called when effect is the same"
        );
    }

    #[tokio::test]
    async fn edit_effect_not_applied_when_not_transparent() {
        let storage = Arc::new(MockStorage::new(AppSettings::default()));
        let window = Arc::new(MockWindowManager::new(false));
        let use_case = EditAppSettingsUseCase::new(storage.clone(), window.clone());

        let request = EditAppSettingsRequest {
            action_on_instance_launch: None,
            transparent: None,
            window_effect: Some(WindowEffect::Acrylic),
        };

        let _ = use_case.execute(request).await.unwrap();
        assert!(
            !window.apply_effect_called.load(Ordering::SeqCst),
            "apply should NOT be called when neither new nor old is transparent"
        );
    }

    #[tokio::test]
    async fn edit_apply_effect_failure_returns_error() {
        let initial = AppSettings {
            action_on_instance_launch: ActionOnInstanceLaunch::Nothing,
            is_actual_transparent: false,
            transparent: true,
            window_effect: WindowEffect::Off,
        };
        let storage = Arc::new(MockStorage::new(initial));
        let window = Arc::new(MockWindowManager::new(true)); // will fail
        let use_case = EditAppSettingsUseCase::new(storage.clone(), window.clone());

        let request = EditAppSettingsRequest {
            action_on_instance_launch: None,
            transparent: None,
            window_effect: Some(WindowEffect::Mica),
        };

        let result = use_case.execute(request).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppSettingsError::CanNotSetEffect(msg) => {
                assert!(msg.contains("mock failure"), "Got: {msg}");
            }
            other => panic!("Expected CanNotSetEffect, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn edit_preserves_unchanged_fields() {
        let storage = Arc::new(MockStorage::new(AppSettings {
            action_on_instance_launch: ActionOnInstanceLaunch::Close,
            is_actual_transparent: false,
            transparent: true,
            window_effect: WindowEffect::Acrylic,
        }));
        let window = Arc::new(MockWindowManager::new(false));
        let use_case = EditAppSettingsUseCase::new(storage.clone(), window.clone());

        let request = EditAppSettingsRequest {
            action_on_instance_launch: Some(ActionOnInstanceLaunch::Hide),
            transparent: None,
            window_effect: None,
        };

        let result = use_case.execute(request).await.unwrap();
        // Changed field
        assert_eq!(
            result.action_on_instance_launch,
            ActionOnInstanceLaunch::Hide
        );
        // Unchanged fields should be preserved
        assert!(result.transparent);
        assert_eq!(result.window_effect, WindowEffect::Acrylic);
    }
}
