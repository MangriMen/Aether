use async_trait::async_trait;

use crate::{
    core::{WindowError, WindowLabel},
    features::settings::WindowEffect,
};

#[async_trait]
pub trait WindowManager: Send + Sync {
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
