use super::ContentId;
use crate::features::instance::ProviderId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackInfo {
    pub provider_id: ProviderId,
    pub modpack_id: ContentId,
    /// The pack's own version label, free-form (an author may leave it empty or write
    /// "2024/06"), not a provider id.
    pub version_id: String,
}
