use crate::features::instance::ProviderId;

#[derive(Debug, Clone)]
pub struct ContentListVersionsParams {
    pub content_id: String,
    pub provider_id: ProviderId,
}
