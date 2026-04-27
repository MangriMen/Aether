use async_trait::async_trait;
use tauri::Runtime;

use crate::{
    core::{close_all_windows, recreate_windows},
    features::settings::{AppSettings, WindowEffect, WindowManager},
};

use super::window::set_window_effect;

pub struct TauriWindowManager<R: Runtime> {
    app_handle: tauri::AppHandle<R>,
}

impl<R: Runtime> TauriWindowManager<R> {
    pub fn new(app_handle: tauri::AppHandle<R>) -> Self {
        Self { app_handle }
    }
}

#[async_trait]
impl<R: Runtime> WindowManager for TauriWindowManager<R> {
    async fn apply_visual_effects(&self, effect: WindowEffect) -> Result<(), String> {
        set_window_effect(self.app_handle.clone(), effect).map_err(|e| e.to_string())
    }

    async fn close_windows(&self) -> Result<(), String> {
        close_all_windows(&self.app_handle);

        Ok(())
    }

    async fn create_windows(&self, app_settings: &AppSettings) -> Result<(), String> {
        recreate_windows(&self.app_handle, app_settings);

        self.apply_visual_effects(app_settings.window_effect)
            .await?;

        Ok(())
    }
}
