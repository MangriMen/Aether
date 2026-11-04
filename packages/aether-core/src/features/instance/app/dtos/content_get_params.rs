use crate::features::instance::{ContentId, ProviderId};

#[derive(Debug, Clone)]
pub struct ContentGetParams {
    pub content_id: ContentId,
    pub provider_id: ProviderId,
}
