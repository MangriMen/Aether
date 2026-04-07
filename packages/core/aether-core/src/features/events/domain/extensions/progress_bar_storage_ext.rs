use async_trait::async_trait;

use crate::features::events::{
    ProgressBar, ProgressBarId, ProgressBarStorage, ProgressBarStorageError,
};

#[async_trait]
pub trait ProgressBarStorageExt: ProgressBarStorage {
    async fn upsert_with<F>(
        &self,
        progress_bar_id: &ProgressBarId,
        update_fn: F,
    ) -> Result<(), ProgressBarStorageError>
    where
        F: FnOnce(&mut ProgressBar) -> Result<(), ProgressBarStorageError> + Send;
}

#[async_trait]
impl<PS: ProgressBarStorage> ProgressBarStorageExt for PS {
    async fn upsert_with<F>(
        &self,
        progress_bar_id: &ProgressBarId,
        update_fn: F,
    ) -> Result<(), ProgressBarStorageError>
    where
        F: FnOnce(&mut ProgressBar) -> Result<(), ProgressBarStorageError> + Send,
    {
        let mut progress_bar = self.get(progress_bar_id.0).await?.clone();
        update_fn(&mut progress_bar)?;
        self.upsert(progress_bar_id.0, progress_bar).await?;
        Ok(())
    }
}
