use tracing::warn;

use crate::features::instance::{PackInfo, infra::legacy_provider_id::parse_stored_provider_id};

#[allow(clippy::struct_field_names)]
pub struct SqlPackInfo {
    pub provider_id: String,
    pub modpack_id: String,
    pub version_id: String,
}

impl SqlPackInfo {
    /// The pack, or `None` when its stored provider id does not parse.
    pub fn into_pack_info(self) -> Option<PackInfo> {
        let Ok(provider_id) = parse_stored_provider_id(&self.provider_id) else {
            // The stored value is lost on the next save of this instance, so leave a trace.
            warn!(
                "Unparsable stored provider id \"{}\" of pack \"{}\"",
                self.provider_id, self.modpack_id
            );
            return None;
        };

        Some(PackInfo {
            provider_id,
            modpack_id: self.modpack_id,
            version_id: self.version_id,
        })
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
    use crate::features::instance::ProviderId;

    #[test]
    fn should_read_no_pack_when_the_provider_id_is_unparsable() {
        let pack_info = SqlPackInfo {
            provider_id: "garbage".to_owned(),
            modpack_id: "m".to_owned(),
            version_id: "v".to_owned(),
        }
        .into_pack_info();

        assert!(pack_info.is_none());
    }

    #[test]
    fn should_read_core_modrinth_as_aether_modrinth_when_the_row_stores_the_old_id() {
        let pack_info = SqlPackInfo {
            provider_id: "core:modrinth#modrinth-content".to_owned(),
            modpack_id: "1KVo5zza".to_owned(),
            version_id: "v1".to_owned(),
        }
        .into_pack_info()
        .unwrap();

        assert_eq!(
            pack_info.provider_id,
            "aether#modrinth".parse::<ProviderId>().unwrap()
        );
        assert_eq!(SqlPackInfo::from(pack_info).provider_id, "aether#modrinth");
    }
}
