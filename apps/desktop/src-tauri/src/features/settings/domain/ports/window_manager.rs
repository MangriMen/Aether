use async_trait::async_trait;

use crate::features::settings::WindowEffect;

#[async_trait]
pub trait WindowManager: Send + Sync {
    async fn apply_visual_effects(&self, effect: WindowEffect) -> Result<(), String>;
}
