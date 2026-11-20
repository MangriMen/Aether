use std::sync::Arc;

use crate::features::instance::InstanceId;
use crate::shared::cache::AssetError;

/// Something the user should know of that did not fail the operation it happened in.
#[derive(Debug, Clone)]
pub enum WarningEvent {
    /// The icon of a new instance could not be imported; the instance was created without it.
    InstanceIconNotImported { cause: Arc<AssetError> },
    /// The post-exit hook of a game was not run: the game ended while no launcher followed it,
    /// or while a launcher that did not start it followed it and could not tell how it ended.
    PostExitHookSkipped {
        instance_id: InstanceId,
        instance_name: String,
    },
}
