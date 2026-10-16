use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{events::ProgressBarStorage, instance::InstanceLocks},
    shared::housekeeping::{ActivityPause, StorageActivity},
};

/// The app is busy while content is being changed in some instance or anything is in progress
/// (a download, an install).
pub(super) struct AppStorageActivity {
    pub instance_locks: Arc<InstanceLocks>,
    pub progress_bars: Arc<dyn ProgressBarStorage>,
}

#[async_trait]
impl StorageActivity for AppStorageActivity {
    async fn try_pause(&self) -> Option<ActivityPause> {
        let pause = self.instance_locks.try_pause()?;
        if self.progress_bars.list().await.is_empty() {
            Some(Box::new(pause))
        } else {
            None
        }
    }
}
