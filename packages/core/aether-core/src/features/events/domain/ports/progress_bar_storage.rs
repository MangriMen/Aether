use async_trait::async_trait;
use dashmap::DashMap;
use uuid::Uuid;

use dashmap::mapref::one::Ref as DashMapRef;

use crate::features::events::{progress_bar_error::ProgressBarStorageError, ProgressBar};

#[async_trait]
pub trait ProgressBarStorage: Send + Sync {
    async fn insert(
        &self,
        progress_bar_id: Uuid,
        progress_bar: ProgressBar,
    ) -> Result<(), ProgressBarStorageError>;
    async fn list(&self) -> DashMap<Uuid, ProgressBar>;
    async fn get(
        &self,
        progress_bar_id: Uuid,
    ) -> Result<DashMapRef<'_, Uuid, ProgressBar>, ProgressBarStorageError>;
    async fn upsert(
        &self,
        progress_bar_id: Uuid,
        progress_bar: ProgressBar,
    ) -> Result<(), ProgressBarStorageError>;
    async fn remove(
        &self,
        progress_bar_id: Uuid,
    ) -> Result<Option<(Uuid, ProgressBar)>, ProgressBarStorageError>;
}
