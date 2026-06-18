use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::features::instance::{
    ContentFileUpdateInfo, PackFile, PackFileDownload, PackFileOption, ProviderId,
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct PackV2 {
    pub files: Vec<PackEntryV2>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct PackEntryV2 {
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
        let update_provider_id = value.update_provider.and_then(|s| {
            if s == "modrinth" {
                Some(ProviderId {
                    plugin_id: "core:modrinth".to_string(),
                    capability_id: "modrinth-content".to_string(),
                })
            } else {
                ProviderId::from_str(&s).ok()
            }
        });

        let update = value.update.map(|old_map| {
            old_map
                .into_iter()
                .map(|(k, v)| {
                    let new_key_str = if k == "modrinth" {
                        "core:modrinth#modrinth-content".to_string()
                    } else {
                        k
                    };

                    let provider_id = ProviderId::from_str(&new_key_str)
                        .expect("Failed to parse ProviderId during migration");

                    let info = migrate_update_info(&v);

                    (provider_id, info)
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
        }
    }
}

impl From<PackFileV2> for PackFile {
    fn from(value: PackFileV2) -> Self {
        let update_provider_id = value
            .update_provider_id
            .as_ref()
            .and_then(|s| ProviderId::from_str(s).ok());

        let update = value.update.map(|old_map| {
            old_map
                .into_iter()
                .map(|(k, v)| {
                    let provider_id = ProviderId::from_str(&k)
                        .expect("Failed to parse ProviderId during deserialization");
                    (
                        provider_id,
                        ContentFileUpdateInfo {
                            content_id: v.content_id,
                            version: v.version,
                        },
                    )
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

fn migrate_update_info(v: &serde_json::Value) -> ContentFileUpdateInfo {
    let obj = v.as_object().expect("Expected JSON object for update info");
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

    ContentFileUpdateInfo {
        content_id,
        version,
    }
}
