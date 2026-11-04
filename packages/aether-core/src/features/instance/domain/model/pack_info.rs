use super::{ContentId, ContentVersionId};
use crate::features::instance::ProviderId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackInfo {
    pub provider_id: ProviderId,
    pub modpack_id: ContentId,
    pub version_id: ContentVersionId,
}
