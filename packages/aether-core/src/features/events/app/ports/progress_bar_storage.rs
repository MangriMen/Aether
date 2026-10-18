use async_trait::async_trait;
use uuid::Uuid;

use dashmap::mapref::one::Ref as DashMapRef;

use crate::features::events::{ProgressBar, ProgressBarStorageError};

/// A change made to a bar in place by [`ProgressBarStorage::update`].
pub type ProgressBarChange<'a> = Box<dyn FnOnce(&mut ProgressBar) + Send + 'a>;

/// Bars are changed in place only ([`update`](Self::update)), never written back whole: a bar is
/// ended (removed) by whoever drops its handle, at any moment, and a copy written back after
/// that would bring it back for good.
#[async_trait]
pub trait ProgressBarStorage: Send + Sync {
    async fn insert(
        &self,
        progress_bar_id: Uuid,
        progress_bar: ProgressBar,
    ) -> Result<(), ProgressBarStorageError>;
    async fn list(&self) -> Vec<ProgressBar>;
    async fn get(
        &self,
        progress_bar_id: Uuid,
    ) -> Result<DashMapRef<'_, Uuid, ProgressBar>, ProgressBarStorageError>;
    /// Apply `change` to the bar, atomically with every other call on it; `NotFound` when the
    /// bar is gone, which `change` then never sees.
    async fn update(
        &self,
        progress_bar_id: Uuid,
        change: ProgressBarChange<'_>,
    ) -> Result<(), ProgressBarStorageError>;
    async fn remove(
        &self,
        progress_bar_id: Uuid,
    ) -> Result<Option<(Uuid, ProgressBar)>, ProgressBarStorageError>;
}
