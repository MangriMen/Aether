use crate::features::instance::{ContentId, ProviderId};

#[derive(Debug, Clone)]
pub struct ContentListVersionsParams {
    pub content_id: ContentId,
    pub provider_id: ProviderId,
}
