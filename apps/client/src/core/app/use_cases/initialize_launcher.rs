use std::sync::Arc;

use aether_core::{
    core::LauncherState,
    features::{events::SharedEventEmitter, settings::LocationInfo},
};

pub struct InitializeLauncherUseCase {
    event_emitter: SharedEventEmitter,
}

impl InitializeLauncherUseCase {
    pub fn new(event_emitter: SharedEventEmitter) -> Self {
        Self { event_emitter }
    }

    pub async fn execute(
        &self,
        location_info: Arc<LocationInfo>,
        pool: sqlx::SqlitePool,
    ) -> crate::Result<()> {
        if LauncherState::initialized().await {
            return Ok(());
        }

        LauncherState::init(location_info, self.event_emitter.clone(), pool).await?;

        Ok(())
    }
}
