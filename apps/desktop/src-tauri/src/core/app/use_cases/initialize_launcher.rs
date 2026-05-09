use std::path::Path;

use aether_core::{core::LauncherState, features::events::SharedEventEmitter};

pub struct InitializeLauncherUseCase {
    event_emitter: SharedEventEmitter,
}

impl InitializeLauncherUseCase {
    pub fn new(event_emitter: SharedEventEmitter) -> Self {
        Self { event_emitter }
    }

    pub async fn execute(
        &self,
        launcher_dir: impl AsRef<Path>,
        pool: sqlx::SqlitePool,
    ) -> crate::Result<()> {
        if LauncherState::initialized().await {
            return Ok(());
        }

        let launcher_dir = launcher_dir.as_ref().to_path_buf();

        LauncherState::init(
            launcher_dir.clone(),
            launcher_dir,
            self.event_emitter.clone(),
            pool,
        )
        .await?;

        Ok(())
    }
}
