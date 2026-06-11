use crate::features::instance::ProviderId;

#[derive(Clone, Debug)]
pub struct PackInfo {
    pub provider_id: ProviderId,
    pub modpack_id: String,
    pub version_id: String,
}
