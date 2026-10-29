use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::features::instance::{
    PackInfo, ProviderId,
    infra::legacy_provider_id::{
        parse_stored_provider_id_parts, parse_v1_provider_id, unknown_provider_id,
    },
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

impl From<PackInfoV1> for PackInfo {
    fn from(old: PackInfoV1) -> Self {
        // A plugin's id names its provider of the same id, as V1 found it.
        let provider_id = parse_v1_provider_id(&old.plugin_id)
            .or_else(|_| ProviderId::from_parts(&old.plugin_id, &old.plugin_id))
            .unwrap_or_else(|e| {
                warn!(
                    "Unparsable stored provider id of pack \"{}\": {e}",
                    old.modpack_id
                );
                unknown_provider_id()
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

impl From<ProviderId> for ProviderIdV1 {
    fn from(value: ProviderId) -> Self {
        Self {
            plugin_id: value.plugin_id.into(),
            capability_id: value.capability_id.into(),
        }
    }
}

impl From<PackInfoV2> for PackInfo {
    fn from(value: PackInfoV2) -> Self {
        let stored = &value.provider_id;
        let provider_id = parse_stored_provider_id_parts(&stored.plugin_id, &stored.capability_id)
            .unwrap_or_else(|e| {
                // The stored value is lost on the next save of this instance, so leave a trace.
                warn!(
                    "Unparsable stored provider id of pack \"{}\": {e}",
                    value.modpack_id
                );
                unknown_provider_id()
            });

        Self {
            provider_id,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_read_an_invalid_stored_provider_id_as_unknown() {
        let dto: PackInfoV2 = serde_json::from_str(
            r#"{
                "providerId": { "pluginId": "a:b", "capabilityId": "cap" },
                "modpackId": "1KVo5zza",
                "versionId": "v1"
            }"#,
        )
        .unwrap();

        let pack_info = PackInfo::from(dto);

        assert_eq!(pack_info.provider_id, unknown_provider_id());
        assert_eq!(pack_info.modpack_id, "1KVo5zza");
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
            PackInfo::from(dto).provider_id,
            "aether#modrinth".parse::<ProviderId>().unwrap()
        );
    }

    fn v1(plugin_id: &str) -> PackInfo {
        serde_json::from_str::<PackInfoV1>(&format!(
            r#"{{ "pluginId": "{plugin_id}", "modpackId": "1KVo5zza", "version": "v1" }}"#
        ))
        .unwrap()
        .into()
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
