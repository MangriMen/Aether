use std::sync::Arc;

use crate::shared::cache::AssetError;

/// Something the user should know of that did not fail the operation it happened in.
#[derive(Debug, Clone)]
pub enum WarningEvent {
    /// The icon of a new instance could not be imported; the instance was created without it.
    InstanceIconNotImported { cause: Arc<AssetError> },
}
