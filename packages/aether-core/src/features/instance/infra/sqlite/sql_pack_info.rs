use crate::features::instance::{
    PackInfo, ProviderId, infra::legacy_provider_id::parse_stored_provider_id,
};

#[allow(clippy::struct_field_names)]
pub struct SqlPackInfo {
    pub provider_id: String,
    pub modpack_id: String,
    pub version_id: String,
}

impl From<SqlPackInfo> for PackInfo {
    fn from(row: SqlPackInfo) -> Self {
        Self {
            provider_id: parse_stored_provider_id(&row.provider_id).unwrap_or_else(|_| {
                // The stored value is lost on the next save of this instance, so leave a trace.
                log::warn!(
                    "Unparsable stored provider id \"{}\" of pack \"{}\"",
                    row.provider_id,
                    row.modpack_id
                );
                ProviderId {
                    plugin_id: "unknown".into(),
                    capability_id: "unknown".into(),
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_read_an_unparsable_provider_id_as_unknown() {
        let pack_info = PackInfo::from(SqlPackInfo {
            provider_id: "garbage".to_owned(),
            modpack_id: "m".to_owned(),
            version_id: "v".to_owned(),
        });

        assert_eq!(pack_info.provider_id.plugin_id, "unknown");
        assert_eq!(pack_info.modpack_id, "m");
    }

    #[test]
    fn should_read_core_modrinth_as_aether_modrinth_when_the_row_stores_the_old_id() {
        let pack_info = PackInfo::from(SqlPackInfo {
            provider_id: "core:modrinth#modrinth-content".to_owned(),
            modpack_id: "1KVo5zza".to_owned(),
            version_id: "v1".to_owned(),
        });

        assert_eq!(
            pack_info.provider_id,
            "aether#modrinth".parse::<ProviderId>().unwrap()
        );
        assert_eq!(SqlPackInfo::from(pack_info).provider_id, "aether#modrinth");
    }
}
