use std::sync::Arc;

use log::warn;

use crate::{features::instance::InstanceStorage, shared::cache::domain::AssetsStorage};

/// Deletes an icon asset once no instance uses it.
///
/// Assets are named by content, so two instances with the same picture share one file: it can
/// only go with its last user.
pub struct IconAssetReaper {
    instances: Arc<dyn InstanceStorage>,
    assets: Arc<dyn AssetsStorage>,
}

impl IconAssetReaper {
    pub fn new(instances: Arc<dyn InstanceStorage>, assets: Arc<dyn AssetsStorage>) -> Self {
        Self { instances, assets }
    }

    /// Call after the instance stopped using `icon`. Never fails: a leftover file is only
    /// wasted space, and must not undo the removal or edit that is already done.
    pub async fn release(&self, icon: &str) {
        let still_used = match self.instances.list().await {
            Ok(instances) => instances
                .iter()
                .any(|instance| instance.icon_path() == Some(icon)),
            Err(err) => {
                warn!("Keeping icon asset {icon}: cannot tell whether it is in use: {err}");
                return;
            }
        };
        if !still_used {
            self.assets.remove(icon).await;
        }
    }
}
