use crate::features::instance::{ContentItem, ProviderId};

#[derive(Clone, Debug)]
pub struct ContentCompatibilityCheckParams {
    pub provider_id: ProviderId,
    pub content_item: ContentItem,
}
