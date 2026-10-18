use async_trait::async_trait;
use dashmap::DashMap;
use uuid::Uuid;

use dashmap::mapref::one::Ref as DashMapRef;

use crate::features::events::{
    ProgressBar, ProgressBarChange, ProgressBarStorage, ProgressBarStorageError,
};

#[derive(Debug, Default)]
pub struct InMemoryProgressBarStorage {
    progress_bars: DashMap<Uuid, ProgressBar>,
}

#[async_trait]
impl ProgressBarStorage for InMemoryProgressBarStorage {
    async fn insert(
        &self,
        progress_bar_id: Uuid,
        progress_bar: ProgressBar,
    ) -> Result<(), ProgressBarStorageError> {
        if self.progress_bars.contains_key(&progress_bar_id) {
            return Err(ProgressBarStorageError::AlreadyExists { progress_bar_id });
        }
        self.progress_bars.insert(progress_bar_id, progress_bar);

        Ok(())
    }

    async fn list(&self) -> Vec<ProgressBar> {
        self.progress_bars
            .iter()
            .map(|record| record.value().clone())
            .collect()
    }

    async fn get(
        &self,
        progress_bar_id: Uuid,
    ) -> Result<DashMapRef<'_, Uuid, ProgressBar>, ProgressBarStorageError> {
        self.progress_bars
            .get(&progress_bar_id)
            .ok_or(ProgressBarStorageError::NotFound { progress_bar_id })
    }

    async fn update(
        &self,
        progress_bar_id: Uuid,
        change: ProgressBarChange<'_>,
    ) -> Result<(), ProgressBarStorageError> {
        let mut progress_bar = self
            .progress_bars
            .get_mut(&progress_bar_id)
            .ok_or(ProgressBarStorageError::NotFound { progress_bar_id })?;
        change(progress_bar.value_mut());

        Ok(())
    }

    async fn remove(
        &self,
        progress_bar_id: Uuid,
    ) -> Result<Option<(Uuid, ProgressBar)>, ProgressBarStorageError> {
        Ok(self.progress_bars.remove(&progress_bar_id))
    }
}
