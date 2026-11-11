use crate::features::instance::{ContentVersionId, ProviderId};

#[derive(Debug, Clone)]
pub struct ContentGetVersionParams {
    pub version_id: ContentVersionId,
    pub provider_id: ProviderId,
}
