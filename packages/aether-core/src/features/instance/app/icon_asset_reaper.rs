use std::{path::Path, sync::Arc};

use log::warn;
use tokio::sync::{Mutex, MutexGuard};

use crate::{
    features::instance::InstanceStorage,
    shared::cache::domain::{AssetError, AssetsStorage},
};

/// Held while an icon is imported and stored on an instance, and by [`IconAssetReaper::release`]
/// while it checks and deletes: a release can never delete the file a concurrent import just
/// reused.
pub type IconAssetGuard<'a> = MutexGuard<'a, ()>;

/// Deletes an icon asset once no instance uses it.
///
/// Assets are named by content, so two instances with the same picture share one file: it can
/// only go with its last user. Clones share one lock, so every user of icon assets must be given
/// clones of a single reaper.
#[derive(Clone)]
pub struct IconAssetReaper {
    instances: Arc<dyn InstanceStorage>,
    assets: Arc<dyn AssetsStorage>,
    lock: Arc<Mutex<()>>,
}

impl IconAssetReaper {
    pub fn new(instances: Arc<dyn InstanceStorage>, assets: Arc<dyn AssetsStorage>) -> Self {
        Self {
            instances,
            assets,
            lock: Arc::new(Mutex::new(())),
        }
    }

    /// Hold the guard from the import of an icon until an instance stores it. Do not call
    /// [`Self::release`] meanwhile.
    pub async fn hold(&self) -> IconAssetGuard<'_> {
        self.lock.lock().await
    }

    /// Imports `source` as an icon asset, under the guard from [`Self::hold`] that the caller
    /// keeps until an instance stores the icon.
    pub async fn import_file(
        &self,
        _guard: &IconAssetGuard<'_>,
        source: &Path,
    ) -> Result<String, AssetError> {
        self.assets.import_file(source).await
    }

    /// Call after the instance stopped using `icon`. Never fails: a leftover file is only
    /// wasted space, and must not undo the removal or edit that is already done. It is logged.
    pub async fn release(&self, icon: &str) {
        let _guard = self.hold().await;

        match self.instances.list().await {
            Ok(instances) => {
                if instances
                    .iter()
                    .any(|instance| instance.icon_path() == Some(icon))
                {
                    return;
                }
            }
            Err(err) => {
                warn!("Keeping icon asset {icon}: cannot tell whether it is in use: {err}");
                return;
            }
        }
        if let Err(err) = self.assets.remove(icon).await {
            warn!("Failed to delete the unused icon asset {icon}: {err}");
        }
    }
}
