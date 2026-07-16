use serde::{Deserialize, Serialize};

use crate::v0::PackSourceDto;

/// Parameters passed to the plugin's `install` handler.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackInstallParamsDto {
    /// The instance ID to install the pack into.
    pub instance_id: String,
    /// The pack source (URL or local path) used to create the instance.
    pub pack_source: PackSourceDto,
}
