use async_trait::async_trait;

use crate::{
    core::{WindowError, WindowLabel},
    features::settings::WindowEffect,
};

#[async_trait]
pub trait WindowManager: Send + Sync {
    /// Whether this system can show `effect`. The system drops the error of one it cannot, so
    /// asking first is the only way to tell the user.
    fn supports_window_effect(&self, _effect: WindowEffect) -> bool {
        true
    }

    async fn apply_visual_effect(
        &self,
        label: WindowLabel,
        effect: WindowEffect,
    ) -> Result<(), WindowError>;

    async fn hide_window(&self, label: WindowLabel) -> Result<(), WindowError>;

    async fn show_window(&self, label: WindowLabel) -> Result<(), WindowError>;

    async fn close_window(&self, label: WindowLabel) -> Result<(), WindowError>;

    async fn close_window_and_wait(&self, label: WindowLabel) -> Result<(), WindowError>;

    async fn create_window(
        &self,
        label: WindowLabel,
        transparent: bool,
        window_effect: WindowEffect,
    ) -> Result<(), WindowError>;
}
