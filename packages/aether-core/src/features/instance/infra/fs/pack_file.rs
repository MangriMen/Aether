use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::features::instance::{
    ContentFileUpdateInfo, PackFile, PackFileDownload, PackFileOption, ProviderId, ProviderIdError,
    infra::{
        legacy_provider_id::{parse_stored_provider_id, parse_v1_provider_id},
        stored_content_id::{parse_stored_content_id, parse_stored_version},
        stored_hash::parse_stored_hash,
        stored_side::parse_stored_side,
    },
};
use crate::shared::hash::{FileHash, InvalidFileHash};

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

impl PackFileV1 {
    /// The record with `hash` in place of the stored one.
    pub fn into_pack_file(self, hash: FileHash) -> PackFile {
        let value = self;

        let update_provider_id = value
            .update_provider
            .and_then(|s| parse_v1_provider_id(&s).ok());

        let side = parse_stored_side(&value.file_name, value.side.as_deref());

        let update = value.update.map(|old_map| {
            old_map
                .into_iter()
                .filter_map(|(k, v)| {
                    let provider_id = parse_update_key(&value.file_name, &k, parse_v1_provider_id)?;
                    let Some(info) = migrate_update_info(&value.file_name, &v) else {
                        warn!(
                            "Skipping update info \"{k}\" of pack file \"{}\": not a table or without valid ids",
                            value.file_name
                        );
                        return None;
                    };

                    Some((provider_id, info))
                })
                .collect()
        });

        PackFile {
            file_name: value.file_name,
            name: value.name,
            hash,
            download: value.download.map(|d| PackFileDownload {
                hash: d.hash,
                url: d.url,
            }),
            option: value.option.map(|o| PackFileOption {
                optional: o.optional,
                default: o.default,
                description: o.description,
            }),
            side,
            update_provider_id,
            update,
            installed_by: None,
        }
    }
}

impl TryFrom<PackFileV1> for PackFile {
    type Error = InvalidFileHash;

    fn try_from(value: PackFileV1) -> Result<Self, Self::Error> {
        let hash = parse_stored_hash(&value.file_name, &value.hash)?;
        Ok(value.into_pack_file(hash))
    }
}

impl PackFileV2 {
    /// The record with `hash` in place of the stored one.
    pub fn into_pack_file(self, hash: FileHash) -> PackFile {
        let value = self;

        let update_provider_id = value
            .update_provider_id
            .as_ref()
            .and_then(|s| parse_stored_provider_id(s).ok());

        let side = parse_stored_side(&value.file_name, value.side.as_deref());

        let update = value.update.map(|old_map| {
            old_map
                .into_iter()
                .filter_map(|(k, v)| {
                    let provider_id =
                        parse_update_key(&value.file_name, &k, parse_stored_provider_id)?;

                    let owner = format!("pack file \"{}\"", value.file_name);
                    Some((
                        provider_id,
                        ContentFileUpdateInfo {
                            content_id: parse_stored_content_id(&owner, v.content_id)?,
                            version: parse_stored_version(&owner, v.version),
                        },
                    ))
                })
                .collect()
        });

        PackFile {
            file_name: value.file_name,
            name: value.name,
            hash,
            download: value.download.map(|d| PackFileDownload {
                hash: d.hash,
                url: d.url,
            }),
            option: value.option.map(|o| PackFileOption {
                optional: o.optional,
                default: o.default,
                description: o.description,
            }),
            side,
            update_provider_id,
            update,
            installed_by: None,
        }
    }
}

impl TryFrom<PackFileV2> for PackFile {
    type Error = InvalidFileHash;

    fn try_from(value: PackFileV2) -> Result<Self, Self::Error> {
        let hash = parse_stored_hash(&value.file_name, &value.hash)?;
        Ok(value.into_pack_file(hash))
    }
}

impl From<PackFile> for PackFileV2 {
    fn from(value: PackFile) -> Self {
        Self {
            file_name: value.file_name,
            name: value.name,
            hash: value.hash.to_string(),
            download: value.download.map(|d| PackFileDownloadV1 {
                hash: d.hash,
                url: d.url,
            }),
            option: value.option.map(|o| PackFileOptionV1 {
                optional: o.optional,
                default: o.default,
                description: o.description,
            }),
            side: value.side.map(|side| side.as_str().to_owned()),
            update_provider_id: value.update_provider_id.map(|id| id.to_string()),
            update: value.update.map(|updates| {
                updates
                    .into_iter()
                    .map(|(k, v)| {
                        (
                            k.to_string(),
                            ContentFileUpdateInfoV1 {
                                content_id: v.content_id.into(),
                                version: v.version.map(String::from).unwrap_or_default(),
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
    parse: fn(&str) -> Result<ProviderId, ProviderIdError>,
) -> Option<ProviderId> {
    parse(key)
        .inspect_err(|e| warn!("Skipping update info of pack file \"{file_name}\": {e}"))
        .ok()
}

fn migrate_update_info(file_name: &str, v: &serde_json::Value) -> Option<ContentFileUpdateInfo> {
    let obj = v.as_object()?;
    let stored = |value: Option<&serde_json::Value>| {
        value
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    };
    let owner = format!("pack file \"{file_name}\"");

    Some(ContentFileUpdateInfo {
        content_id: parse_stored_content_id(
            &owner,
            stored(obj.get("contentId").or_else(|| obj.get("project_id"))),
        )?,
        version: parse_stored_version(&owner, stored(obj.get("version"))),
    })
}

#[cfg(test)]
mod tests {
    use crate::features::instance::ContentId;
    use crate::features::instance::ProviderId;
    use rstest::rstest;

    use super::*;

    fn aether_modrinth() -> ProviderId {
        "aether#modrinth".parse().unwrap()
    }

    #[test]
    fn should_drop_the_update_info_when_a_stored_id_is_not_a_valid_id() {
        let dto: PackFileV2 = toml::from_str(
            r#"
            file-name = "sodium.jar"
            hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            update-provider-id = "aether#modrinth"

            [update."aether#modrinth"]
            content-id = ""
            version = "v1"
            "#,
        )
        .unwrap();

        let pack_file = PackFile::try_from(dto).unwrap();

        assert!(pack_file.update.unwrap().is_empty());
    }

    #[test]
    fn should_read_v2_core_modrinth_as_aether_modrinth_when_the_pack_stores_the_old_id() {
        let dto: PackFileV2 = toml::from_str(
            r#"
            file-name = "sodium.jar"
            hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            update-provider-id = "core:modrinth#modrinth-content"

            [update."core:modrinth#modrinth-content"]
            content-id = "AANobbMI"
            version = "v1"
            "#,
        )
        .unwrap();

        let pack_file = PackFile::try_from(dto).unwrap();

        assert_eq!(pack_file.update_provider_id, Some(aether_modrinth()));
        let update = pack_file.update.unwrap();
        assert_eq!(update.len(), 1);
        assert_eq!(
            update[&aether_modrinth()].content_id,
            ContentId::try_from("AANobbMI").unwrap()
        );
    }

    #[test]
    fn should_keep_the_update_info_without_a_version_when_a_v1_record_has_an_empty_one() {
        let pack_file = read_v1(
            r#"
            file-name = "sodium.jar"
            hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            update-provider = "modrinth"

            [update.modrinth]
            project_id = "AANobbMI"
            version = ""
            "#,
        );

        let update = pack_file.update.unwrap();
        let info = &update[&aether_modrinth()];
        assert_eq!(info.content_id, ContentId::try_from("AANobbMI").unwrap());
        assert_eq!(info.version, None);
    }

    #[test]
    fn should_keep_the_update_info_without_a_version_when_a_v2_version_is_invalid() {
        let pack_file = read_v2(
            r#"
            file-name = "sodium.jar"
            hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            update-provider-id = "aether#modrinth"

            [update."aether#modrinth"]
            content-id = "AANobbMI"
            version = "a/b"
            "#,
        );

        let update = pack_file.update.unwrap();
        assert_eq!(update[&aether_modrinth()].version, None);
    }

    #[test]
    fn should_read_v1_bare_modrinth_as_aether_modrinth_when_the_pack_stores_no_prefix() {
        let dto: PackFileV1 = toml::from_str(
            r#"
            file-name = "sodium.jar"
            hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            update-provider = "modrinth"

            [update.modrinth]
            project_id = "AANobbMI"
            version = "v1"
            "#,
        )
        .unwrap();

        let pack_file = PackFile::try_from(dto).unwrap();

        assert_eq!(pack_file.update_provider_id, Some(aether_modrinth()));
        let update = pack_file.update.unwrap();
        assert_eq!(
            update[&aether_modrinth()].content_id,
            ContentId::try_from("AANobbMI").unwrap()
        );
    }

    #[test]
    fn should_refuse_the_pack_file_when_its_hash_is_not_a_digest() {
        let dto: PackFileV2 = toml::from_str(
            r#"
            file-name = "sodium.jar"
            hash = "abc"
            "#,
        )
        .unwrap();

        assert!(PackFile::try_from(dto).is_err());
    }

    fn read_v1(text: &str) -> PackFile {
        PackFile::try_from(toml::from_str::<PackFileV1>(text).unwrap()).unwrap()
    }

    fn read_v2(text: &str) -> PackFile {
        PackFile::try_from(toml::from_str::<PackFileV2>(text).unwrap()).unwrap()
    }

    /// A damaged entry costs that entry, not the whole read.
    #[rstest]
    #[case::v2_unparsable_key(
        read_v2,
        r#"
        file-name = "sodium.jar"
        hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"

        [update.no-separator]
        content-id = "broken"
        version = "v0"

        [update."aether#modrinth"]
        content-id = "AANobbMI"
        version = "v1"
        "#
    )]
    #[case::v1_unparsable_key(
        read_v1,
        r#"
        file-name = "sodium.jar"
        hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"

        [update.no-separator]
        project_id = "broken"

        [update.modrinth]
        project_id = "AANobbMI"
        version = "v1"
        "#
    )]
    #[case::v1_entry_is_not_a_table(
        read_v1,
        r#"
        file-name = "sodium.jar"
        hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"

        [update]
        "curseforge#cf" = "not a table"

        [update.modrinth]
        project_id = "AANobbMI"
        version = "v1"
        "#
    )]
    fn should_skip_the_damaged_update_entry_and_keep_the_rest(
        #[case] read: fn(&str) -> PackFile,
        #[case] text: &str,
    ) {
        let update = read(text).update.unwrap();

        assert_eq!(update.len(), 1);
        assert_eq!(
            update[&aether_modrinth()].content_id,
            ContentId::try_from("AANobbMI").unwrap()
        );
    }

    #[test]
    fn should_write_aether_modrinth_back_when_the_pack_is_saved() {
        let dto: PackFileV2 = toml::from_str(
            r#"
            file-name = "sodium.jar"
            hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            update-provider-id = "core:modrinth#modrinth-content"
            "#,
        )
        .unwrap();

        let written = PackFileV2::from(PackFile::try_from(dto).unwrap());

        assert_eq!(
            written.update_provider_id.as_deref(),
            Some("aether#modrinth")
        );
    }
}
