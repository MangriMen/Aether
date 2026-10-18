use async_trait::async_trait;

use crate::features::{
    events::ProgressBarId,
    instance::{InstanceError, InstanceId},
};

#[async_trait]
pub trait InstanceInstallService: Send + Sync {
    /// Install the game of the instance. Its bar goes under `parent_bar` (a pack being
    /// installed into it), or at the top without one; the shared downloads it waits on (its
    /// Java, its game version) are mirrored under that bar.
    async fn execute(
        &self,
        instance_id: InstanceId,
        force: bool,
        parent_bar: Option<&ProgressBarId>,
    ) -> Result<(), InstanceError>;
}
