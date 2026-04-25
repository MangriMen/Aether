use serde::{Deserialize, Serialize};

use crate::features::instance::ProviderId;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackInfo {
    pub provider_id: ProviderId,
    pub modpack_id: String,
    pub version_id: String,
}
