use std::str::FromStr;

use crate::features::instance::{PackInfo, ProviderId};

pub struct SqlPackInfo {
    pub provider_id: String,
    pub modpack_id: String,
    pub version_id: String,
}

impl From<SqlPackInfo> for PackInfo {
    fn from(row: SqlPackInfo) -> Self {
        Self {
            provider_id: ProviderId::from_str(&row.provider_id).unwrap_or_else(|_| ProviderId {
                plugin_id: "unknown".into(),
                capability_id: "unknown".into(),
            }),
            modpack_id: row.modpack_id,
            version_id: row.version_id,
        }
    }
}

impl From<PackInfo> for SqlPackInfo {
    fn from(p: PackInfo) -> Self {
        Self {
            provider_id: p.provider_id.to_string(),
            modpack_id: p.modpack_id,
            version_id: p.version_id,
        }
    }
}
