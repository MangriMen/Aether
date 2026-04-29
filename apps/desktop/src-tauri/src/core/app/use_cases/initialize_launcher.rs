use std::path::{Path, PathBuf};

use aether_core::{core::LauncherState, features::events::SharedEventEmitter};

pub struct InitializeLauncherUseCase {
    event_emitter: SharedEventEmitter,
}

impl InitializeLauncherUseCase {
    pub fn new(event_emitter: SharedEventEmitter) -> Self {
        Self { event_emitter }
    }

    pub async fn execute(&self, launcher_dir: impl AsRef<Path>) -> crate::Result<()> {
        if LauncherState::initialized().await {
            return Ok(());
        }

        let launcher_dir = launcher_dir.as_ref().to_path_buf();

        let db_path = launcher_dir.join("app.db");
        let pool = Self::create_pool(db_path).await?;

        LauncherState::init(
            launcher_dir.clone(),
            launcher_dir,
            self.event_emitter.clone(),
            pool,
        )
        .await?;

        Ok(())
    }

    async fn create_pool(db_path: PathBuf) -> crate::Result<sqlx::SqlitePool> {
        let connection_options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = sqlx::SqlitePool::connect_with(connection_options)
            .await
            .map_err(|err| crate::Error::LaunchError(err.to_string()))?;

        Ok(pool)
    }
}
