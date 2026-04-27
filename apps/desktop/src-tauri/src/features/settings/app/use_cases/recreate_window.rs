use std::{sync::Arc, time::Duration};

use tokio::time::sleep;

use crate::features::settings::{AppSettings, AppSettingsError, AppSettingsStorage, WindowManager};

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

    pub async fn execute(&self) -> Result<(), AppSettingsError> {
        let app_settings = self.app_settings_storage.get().await?;

        self.window_manager
            .close_windows()
            .await
            .map_err(|err| AppSettingsError::CanNotSetEffect(err.clone()))?;

        sleep(Duration::from_secs(1)).await;

        self.window_manager
            .create_windows(&app_settings)
            .await
            .map_err(|err| AppSettingsError::CanNotSetEffect(err.clone()))?;

        let new_settings = AppSettings {
            is_actual_transparent: app_settings.transparent,
            ..app_settings
        };

        self.app_settings_storage.upsert(new_settings).await?;

        Ok(())
    }
}
