use std::sync::Arc;

use crate::{
    core::{WindowLabel, WindowManager},
    features::settings::AppSettingsStorage,
};

pub struct RecreateWindowUseCase<ASS: AppSettingsStorage, WM: WindowManager> {
    app_settings_storage: Arc<ASS>,
    window_manager: Arc<WM>,
}

impl<ASS: AppSettingsStorage, WM: WindowManager> RecreateWindowUseCase<ASS, WM> {
    pub fn new(app_settings_storage: Arc<ASS>, window_manager: Arc<WM>) -> Self {
        Self {
            app_settings_storage,
            window_manager,
        }
    }

    pub async fn execute(&self) -> crate::Result<()> {
        self.window_manager
            .close_window_and_wait(WindowLabel::Main)
            .await?;

        let mut app_settings = self.app_settings_storage.get().await?;

        self.window_manager
            .create_window(
                WindowLabel::Main,
                app_settings.transparent,
                app_settings.window_effect,
            )
            .await?;

        app_settings.is_actual_transparent = app_settings.transparent;

        self.app_settings_storage.upsert(app_settings).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::WindowError;
    use crate::features::settings::{
        ActionOnInstanceLaunch, AppSettings, AppSettingsError, AppSettingsStorage, WindowEffect,
    };
    use async_trait::async_trait;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};

    // ── Mock Storage ──

    #[derive(Clone)]
    struct MockStorage {
        stored: Arc<Mutex<AppSettings>>,
    }

    impl MockStorage {
        fn new(settings: AppSettings) -> Self {
            Self {
                stored: Arc::new(Mutex::new(settings)),
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
            Ok(())
        }
    }

    // ── Mock Window Manager ──

    #[derive(Clone)]
    struct MockWindowManager {
        close_and_wait_called: Arc<AtomicBool>,
        create_window_called: Arc<AtomicBool>,
        last_transparent: Arc<Mutex<Option<bool>>>,
        last_effect: Arc<Mutex<Option<WindowEffect>>>,
        close_should_fail: bool,
        create_should_fail: bool,
    }

    impl MockWindowManager {
        fn new() -> Self {
            Self {
                close_and_wait_called: Arc::new(AtomicBool::new(false)),
                create_window_called: Arc::new(AtomicBool::new(false)),
                last_transparent: Arc::new(Mutex::new(None)),
                last_effect: Arc::new(Mutex::new(None)),
                close_should_fail: false,
                create_should_fail: false,
            }
        }
    }

    #[async_trait]
    impl WindowManager for MockWindowManager {
        async fn apply_visual_effect(
            &self,
            _label: WindowLabel,
            _effect: WindowEffect,
        ) -> Result<(), WindowError> {
            Ok(())
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

        async fn close_window_and_wait(&self, label: WindowLabel) -> Result<(), WindowError> {
            self.close_and_wait_called.store(true, Ordering::SeqCst);
            assert_eq!(label, WindowLabel::Main, "Should close Main window");
            if self.close_should_fail {
                Err(WindowError::Other("close failed".to_string()))
            } else {
                Ok(())
            }
        }

        async fn create_window(
            &self,
            label: WindowLabel,
            transparent: bool,
            window_effect: WindowEffect,
        ) -> Result<(), WindowError> {
            self.create_window_called.store(true, Ordering::SeqCst);
            assert_eq!(label, WindowLabel::Main);
            *self.last_transparent.lock().unwrap() = Some(transparent);
            *self.last_effect.lock().unwrap() = Some(window_effect);
            if self.create_should_fail {
                Err(WindowError::Other("create failed".to_string()))
            } else {
                Ok(())
            }
        }
    }

    // ── Tests ──

    #[tokio::test]
    async fn recreate_window_happy_path() {
        let storage = Arc::new(MockStorage::new(AppSettings {
            action_on_instance_launch: ActionOnInstanceLaunch::Nothing,
            is_actual_transparent: false,
            transparent: true,
            window_effect: WindowEffect::Mica,
        }));
        let window = Arc::new(MockWindowManager::new());
        let use_case = RecreateWindowUseCase::new(storage.clone(), window.clone());

        use_case.execute().await.unwrap();

        assert!(window.close_and_wait_called.load(Ordering::SeqCst));
        assert!(window.create_window_called.load(Ordering::SeqCst));
        assert_eq!(*window.last_transparent.lock().unwrap(), Some(true));
        assert_eq!(
            *window.last_effect.lock().unwrap(),
            Some(WindowEffect::Mica)
        );
    }

    #[tokio::test]
    async fn recreate_window_syncs_is_actual_transparent() {
        let storage = Arc::new(MockStorage::new(AppSettings {
            action_on_instance_launch: ActionOnInstanceLaunch::Nothing,
            is_actual_transparent: false,
            transparent: true,
            window_effect: WindowEffect::Off,
        }));
        let window = Arc::new(MockWindowManager::new());
        let use_case = RecreateWindowUseCase::new(storage.clone(), window.clone());

        use_case.execute().await.unwrap();

        let stored = storage.stored.lock().unwrap();
        assert!(
            stored.is_actual_transparent,
            "After recreate, is_actual_transparent should be true"
        );
        assert!(stored.transparent);
    }

    #[tokio::test]
    async fn recreate_window_creates_with_correct_transparency() {
        let storage = Arc::new(MockStorage::new(AppSettings {
            action_on_instance_launch: ActionOnInstanceLaunch::Hide,
            is_actual_transparent: true,
            transparent: false,
            window_effect: WindowEffect::Acrylic,
        }));
        let window = Arc::new(MockWindowManager::new());
        let use_case = RecreateWindowUseCase::new(storage.clone(), window.clone());

        use_case.execute().await.unwrap();

        assert_eq!(*window.last_transparent.lock().unwrap(), Some(false));
        assert_eq!(
            *window.last_effect.lock().unwrap(),
            Some(WindowEffect::Acrylic)
        );
    }
}
