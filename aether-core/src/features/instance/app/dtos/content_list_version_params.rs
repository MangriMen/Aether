use serde::{Deserialize, Serialize};

use crate::features::instance::ProviderId;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentListVersionParams {
    pub content_id: String,
    pub provider_id: ProviderId,
}
