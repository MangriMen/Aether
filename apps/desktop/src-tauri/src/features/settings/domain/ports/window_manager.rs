use async_trait::async_trait;

use crate::features::settings::{AppSettings, WindowEffect};

#[async_trait]
pub trait WindowManager: Send + Sync {
    async fn apply_visual_effects(&self, effect: WindowEffect) -> Result<(), String>;

    async fn close_windows(&self) -> Result<(), String>;

    async fn create_windows(&self, app_settings: &AppSettings) -> Result<(), String>;
}
