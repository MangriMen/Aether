use serde::{Deserialize, Serialize};

use crate::features::instance::{ContentItem, ProviderId};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContentCompatibilityCheckParams {
    pub provider_id: ProviderId,
    pub content_item: ContentItem,
}
