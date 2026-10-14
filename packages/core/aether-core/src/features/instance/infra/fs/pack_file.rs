use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::features::instance::{
    ContentFileUpdateInfo, PackFile, PackFileDownload, PackFileOption, ProviderId,
    infra::legacy_provider_id::{parse_stored_provider_id, parse_v1_provider_id},
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub(super) struct PackV2 {
    pub files: Vec<PackEntryV2>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub(super) struct PackEntryV2 {
    pub file: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PackFileV1 {
    pub file_name: String,
    pub name: Option<String>,
    pub hash: String,
    pub download: Option<PackFileDownloadV1>,
    pub option: Option<PackFileOptionV1>,
    pub side: Option<String>,
    pub update_provider: Option<String>,
    pub update: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PackFileDownloadV1 {
    pub hash: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PackFileOptionV1 {
    pub optional: bool,
    pub default: Option<bool>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PackFileV2 {
    pub file_name: String,
    pub name: Option<String>,
    pub hash: String,
    pub download: Option<PackFileDownloadV1>,
    pub option: Option<PackFileOptionV1>,
    pub side: Option<String>,
    pub update_provider_id: Option<String>,
    pub update: Option<HashMap<String, ContentFileUpdateInfoV1>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ContentFileUpdateInfoV1 {
    pub content_id: String,
    pub version: String,
}

impl From<PackFileV1> for PackFile {
    fn from(value: PackFileV1) -> Self {
        let update_provider_id = value
            .update_provider
            .and_then(|s| parse_v1_provider_id(&s).ok());

        let update = value.update.map(|old_map| {
            old_map
                .into_iter()
                .filter_map(|(k, v)| {
                    let provider_id = parse_update_key(&value.file_name, &k, parse_v1_provider_id)?;
                    let Some(info) = migrate_update_info(&v) else {
                        warn!(
                            "Skipping update info \"{k}\" of pack file \"{}\": not a table",
                            value.file_name
                        );
                        return None;
                    };

                    Some((provider_id, info))
                })
                .collect()
        });

        Self {
            file_name: value.file_name,
            name: value.name,
            hash: value.hash,
            download: value.download.map(|d| PackFileDownload {
                hash: d.hash,
                url: d.url,
            }),
            option: value.option.map(|o| PackFileOption {
                optional: o.optional,
                default: o.default,
                description: o.description,
            }),
            side: value.side,
            update_provider_id,
            update,
            installed_by: None,
        }
    }
}

impl From<PackFileV2> for PackFile {
    fn from(value: PackFileV2) -> Self {
        let update_provider_id = value
            .update_provider_id
            .as_ref()
            .and_then(|s| parse_stored_provider_id(s).ok());

        let update = value.update.map(|old_map| {
            old_map
                .into_iter()
                .filter_map(|(k, v)| {
                    let provider_id =
                        parse_update_key(&value.file_name, &k, parse_stored_provider_id)?;

                    Some((
                        provider_id,
                        ContentFileUpdateInfo {
                            content_id: v.content_id,
                            version: v.version,
                        },
                    ))
                })
                .collect()
        });

        Self {
            file_name: value.file_name,
            name: value.name,
            hash: value.hash,
            download: value.download.map(|d| PackFileDownload {
                hash: d.hash,
                url: d.url,
            }),
            option: value.option.map(|o| PackFileOption {
                optional: o.optional,
                default: o.default,
                description: o.description,
            }),
            side: value.side,
            update_provider_id,
            update,
            installed_by: None,
        }
    }
}

impl From<PackFile> for PackFileV2 {
    fn from(value: PackFile) -> Self {
        Self {
            file_name: value.file_name,
            name: value.name,
            hash: value.hash,
            download: value.download.map(|d| PackFileDownloadV1 {
                hash: d.hash,
                url: d.url,
            }),
            option: value.option.map(|o| PackFileOptionV1 {
                optional: o.optional,
                default: o.default,
                description: o.description,
            }),
            side: value.side,
            update_provider_id: value.update_provider_id.map(|id| id.to_string()),
            update: value.update.map(|updates| {
                updates
                    .into_iter()
                    .map(|(k, v)| {
                        (
                            k.to_string(),
                            ContentFileUpdateInfoV1 {
                                content_id: v.content_id,
                                version: v.version,
                            },
                        )
                    })
                    .collect()
            }),
        }
    }
}

/// The provider id an `update` entry is keyed by, or `None` — with a warning — when the key is
/// not one (a hand-edited or damaged pack file): one bad entry costs that entry, not the read.
fn parse_update_key(
    file_name: &str,
    key: &str,
    parse: fn(&str) -> Result<ProviderId, String>,
) -> Option<ProviderId> {
    parse(key)
        .inspect_err(|e| warn!("Skipping update info of pack file \"{file_name}\": {e}"))
        .ok()
}

fn migrate_update_info(v: &serde_json::Value) -> Option<ContentFileUpdateInfo> {
    let obj = v.as_object()?;
    let content_id = obj
        .get("contentId")
        .or_else(|| obj.get("project_id"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let version = obj
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    Some(ContentFileUpdateInfo {
        content_id,
        version,
    })
}

#[cfg(test)]
mod tests {
    use crate::features::instance::ProviderId;

    use super::*;

    fn aether_modrinth() -> ProviderId {
        "aether#modrinth".parse().unwrap()
    }

    #[test]
    fn should_read_v2_core_modrinth_as_aether_modrinth_when_the_pack_stores_the_old_id() {
        let dto: PackFileV2 = toml::from_str(
            r#"
            file-name = "sodium.jar"
            hash = "abc"
            update-provider-id = "core:modrinth#modrinth-content"

            [update."core:modrinth#modrinth-content"]
            content-id = "AANobbMI"
            version = "v1"
            "#,
        )
        .unwrap();

        let pack_file = PackFile::from(dto);

        assert_eq!(pack_file.update_provider_id, Some(aether_modrinth()));
        let update = pack_file.update.unwrap();
        assert_eq!(update.len(), 1);
        assert_eq!(update[&aether_modrinth()].content_id, "AANobbMI");
    }

    #[test]
    fn should_read_v1_bare_modrinth_as_aether_modrinth_when_the_pack_stores_no_prefix() {
        let dto: PackFileV1 = toml::from_str(
            r#"
            file-name = "sodium.jar"
            hash = "abc"
            update-provider = "modrinth"

            [update.modrinth]
            project_id = "AANobbMI"
            version = "v1"
            "#,
        )
        .unwrap();

        let pack_file = PackFile::from(dto);

        assert_eq!(pack_file.update_provider_id, Some(aether_modrinth()));
        let update = pack_file.update.unwrap();
        assert_eq!(update[&aether_modrinth()].content_id, "AANobbMI");
    }

    /// A damaged entry costs that entry, not the whole read.
    #[test]
    fn should_skip_v2_update_entry_when_its_key_is_unparsable() {
        let dto: PackFileV2 = toml::from_str(
            r#"
            file-name = "sodium.jar"
            hash = "abc"

            [update.no-separator]
            content-id = "broken"
            version = "v0"

            [update."aether#modrinth"]
            content-id = "AANobbMI"
            version = "v1"
            "#,
        )
        .unwrap();

        let update = PackFile::from(dto).update.unwrap();

        assert_eq!(update.len(), 1);
        assert_eq!(update[&aether_modrinth()].content_id, "AANobbMI");
    }

    #[test]
    fn should_skip_v1_update_entry_when_its_key_is_unparsable() {
        let dto: PackFileV1 = toml::from_str(
            r#"
            file-name = "sodium.jar"
            hash = "abc"

            [update.no-separator]
            project_id = "broken"

            [update.modrinth]
            project_id = "AANobbMI"
            version = "v1"
            "#,
        )
        .unwrap();

        let update = PackFile::from(dto).update.unwrap();

        assert_eq!(update.len(), 1);
        assert_eq!(update[&aether_modrinth()].content_id, "AANobbMI");
    }

    #[test]
    fn should_skip_v1_update_entry_when_it_is_not_a_table() {
        let dto: PackFileV1 = toml::from_str(
            r#"
            file-name = "sodium.jar"
            hash = "abc"

            [update]
            "curseforge#cf" = "not a table"

            [update.modrinth]
            project_id = "AANobbMI"
            version = "v1"
            "#,
        )
        .unwrap();

        let update = PackFile::from(dto).update.unwrap();

        assert_eq!(update.len(), 1);
        assert_eq!(update[&aether_modrinth()].content_id, "AANobbMI");
    }

    #[test]
    fn should_write_aether_modrinth_back_when_the_pack_is_saved() {
        let dto: PackFileV2 = toml::from_str(
            r#"
            file-name = "sodium.jar"
            hash = "abc"
            update-provider-id = "core:modrinth#modrinth-content"
            "#,
        )
        .unwrap();

        let written = PackFileV2::from(PackFile::from(dto));

        assert_eq!(
            written.update_provider_id.as_deref(),
            Some("aether#modrinth")
        );
    }
}
