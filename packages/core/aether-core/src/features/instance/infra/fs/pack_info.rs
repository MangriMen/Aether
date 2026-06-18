use std::str::FromStr;

use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ProviderIdV1 {
    pub plugin_id: String,
    pub capability_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_field_names)]
pub struct PackInfoV2 {
    pub provider_id: ProviderIdV1,
    pub modpack_id: String,
    pub version_id: String,
}

impl From<ProviderIdV1> for ProviderId {
    fn from(value: ProviderIdV1) -> Self {
        Self {
            plugin_id: value.plugin_id,
            capability_id: value.capability_id,
        }
    }
}

impl From<ProviderId> for ProviderIdV1 {
    fn from(value: ProviderId) -> Self {
        Self {
            plugin_id: value.plugin_id,
            capability_id: value.capability_id,
        }
    }
}

impl From<PackInfoV2> for PackInfo {
    fn from(value: PackInfoV2) -> Self {
        Self {
            provider_id: value.provider_id.into(),
            modpack_id: value.modpack_id,
            version_id: value.version_id,
        }
    }
}

impl From<PackInfo> for PackInfoV2 {
    fn from(value: PackInfo) -> Self {
        Self {
            provider_id: value.provider_id.into(),
            modpack_id: value.modpack_id,
            version_id: value.version_id,
        }
    }
}
