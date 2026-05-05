use std::str::FromStr;

use serde::Deserialize;

use crate::features::instance::{PackInfo, ProviderId};

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackInfoV1 {
    pub plugin_id: String,
    pub modpack_id: String,
    pub version: String,
}

impl From<PackInfoV1> for PackInfo {
    fn from(old: PackInfoV1) -> Self {
        let provider_id = ProviderId::from_str(&old.plugin_id).unwrap_or_else(|_| ProviderId {
            plugin_id: old.plugin_id.clone(),
            capability_id: old.plugin_id,
        });

        Self {
            provider_id,
            modpack_id: old.modpack_id,
            version_id: old.version,
        }
    }
}
