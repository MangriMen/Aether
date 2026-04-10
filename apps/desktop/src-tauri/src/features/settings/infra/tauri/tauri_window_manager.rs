use async_trait::async_trait;
use tauri::Runtime;

use crate::features::settings::{WindowEffect, WindowManager};

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
}
