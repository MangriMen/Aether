use serde::{Deserialize, Serialize};

use crate::features::instance::ProviderId;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentListVersionsParams {
    pub content_id: String,
    pub provider_id: ProviderId,
}
