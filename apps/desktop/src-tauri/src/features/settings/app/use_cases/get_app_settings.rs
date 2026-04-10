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
