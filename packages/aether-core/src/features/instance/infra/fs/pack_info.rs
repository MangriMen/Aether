use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::features::instance::{
    PackInfo, ProviderId,
    infra::legacy_provider_id::{parse_stored_provider_id_parts, parse_v1_provider_id},
};

/// The pack of a V1 instance. Its `plugin_id` was a bare id: `modrinth` for the built-in Modrinth
/// provider, a plugin's own id for its packs.
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackInfoV1 {
    pub plugin_id: String,
    pub modpack_id: String,
    pub version: String,
}

impl PackInfoV1 {
    /// The pack, or `None` when its stored provider id does not parse.
    pub fn into_pack_info(self) -> Option<PackInfo> {
        // A plugin's id names its provider of the same id, as V1 found it.
        let provider_id = parse_v1_provider_id(&self.plugin_id)
            .or_else(|_| ProviderId::from_parts(&self.plugin_id, &self.plugin_id))
            .inspect_err(|e| {
                warn!(
                    "Unparsable stored provider id \"{}\" of pack \"{}\": {e}",
                    self.plugin_id, self.modpack_id
                );
            })
            .ok()?;

        Some(PackInfo {
            provider_id,
            modpack_id: self.modpack_id,
            version_id: self.version,
        })
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

impl From<ProviderId> for ProviderIdV1 {
    fn from(value: ProviderId) -> Self {
        Self {
            plugin_id: value.plugin_id.into(),
            capability_id: value.capability_id.into(),
        }
    }
}

impl PackInfoV2 {
    /// The pack, or `None` when its stored provider id does not parse.
    pub fn into_pack_info(self) -> Option<PackInfo> {
        let stored = &self.provider_id;
        let provider_id = parse_stored_provider_id_parts(&stored.plugin_id, &stored.capability_id)
            .inspect_err(|e| {
                // The stored value is lost on the next save of this instance, so leave a trace.
                warn!(
                    "Unparsable stored provider id \"{}#{}\" of pack \"{}\": {e}",
                    stored.plugin_id, stored.capability_id, self.modpack_id
                );
            })
            .ok()?;

        Some(PackInfo {
            provider_id,
            modpack_id: self.modpack_id,
            version_id: self.version_id,
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_read_no_pack_when_the_stored_provider_id_is_invalid() {
        let dto: PackInfoV2 = serde_json::from_str(
            r#"{
                "providerId": { "pluginId": "a:b", "capabilityId": "cap" },
                "modpackId": "1KVo5zza",
                "versionId": "v1"
            }"#,
        )
        .unwrap();

        assert!(dto.into_pack_info().is_none());
    }

    #[test]
    fn should_read_core_modrinth_as_aether_modrinth_when_the_pack_stores_the_old_id() {
        let dto: PackInfoV2 = serde_json::from_str(
            r#"{
                "providerId": { "pluginId": "core:modrinth", "capabilityId": "modrinth-content" },
                "modpackId": "1KVo5zza",
                "versionId": "v1"
            }"#,
        )
        .unwrap();

        assert_eq!(
            dto.into_pack_info().unwrap().provider_id,
            "aether#modrinth".parse::<ProviderId>().unwrap()
        );
    }

    fn v1(plugin_id: &str) -> PackInfo {
        serde_json::from_str::<PackInfoV1>(&format!(
            r#"{{ "pluginId": "{plugin_id}", "modpackId": "1KVo5zza", "version": "v1" }}"#
        ))
        .unwrap()
        .into_pack_info()
        .unwrap()
    }

    /// V1's `ModrinthContentProvider` wrote its packs as `modrinth`; read as `modrinth#modrinth`,
    /// such a pack would find no provider to update it, or a plugin's of that id.
    #[test]
    fn should_read_a_v1_pack_of_bare_modrinth_as_aether_modrinth_when_it_has_no_prefix() {
        let pack_info = v1("modrinth");

        assert_eq!(
            pack_info.provider_id,
            "aether#modrinth".parse::<ProviderId>().unwrap()
        );
        assert_eq!(pack_info.modpack_id, "1KVo5zza");
        assert_eq!(pack_info.version_id, "v1");
    }

    #[test]
    fn should_read_a_v1_pack_of_a_plugin_as_its_provider_when_the_id_is_not_modrinth() {
        assert_eq!(
            v1("packwiz").provider_id,
            "packwiz#packwiz".parse::<ProviderId>().unwrap()
        );
    }
}
