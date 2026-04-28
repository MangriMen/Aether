use std::{sync::Arc, time::Duration};

use tokio::time::sleep;

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
        self.window_manager.close_window(WindowLabel::Main).await?;

        sleep(Duration::from_secs(1)).await;

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
