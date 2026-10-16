use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{events::ProgressBarStorage, instance::InstanceLocks},
    shared::housekeeping::StorageActivity,
};

/// The app is busy while content is being changed in some instance or anything is in progress
/// (a download, an install).
pub(super) struct AppStorageActivity {
    pub instance_locks: Arc<InstanceLocks>,
    pub progress_bars: Arc<dyn ProgressBarStorage>,
}

#[async_trait]
impl StorageActivity for AppStorageActivity {
    async fn is_busy(&self) -> bool {
        self.instance_locks.is_any_locked() || !self.progress_bars.list().await.is_empty()
    }
}
