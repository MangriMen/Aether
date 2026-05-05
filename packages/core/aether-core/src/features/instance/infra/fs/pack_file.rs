use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::features::instance::{
    ContentFileUpdateInfo, PackFile, PackFileDownload, PackFileOption, ProviderId,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PackFileV1 {
    pub file_name: String,
    pub name: Option<String>,
    pub hash: String,
    pub download: Option<PackFileDownload>,
    pub option: Option<PackFileOption>,
    pub side: Option<String>,
    pub update_provider: Option<String>,
    pub update: Option<HashMap<String, serde_json::Value>>,
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

                    let info = migrate_update_info(v);

                    (provider_id, info)
                })
                .collect()
        });

        Self {
            file_name: value.file_name,
            name: value.name,
            hash: value.hash,
            download: value.download,
            option: value.option,
            side: value.side,
            update_provider_id,
            update,
        }
    }
}

fn migrate_update_info(mut v: serde_json::Value) -> ContentFileUpdateInfo {
    if let Some(obj) = v.as_object_mut()
        && let Some(id) = obj.remove("project_id")
    {
        obj.insert("contentId".to_string(), id);
    }
    serde_json::from_value(v).expect("Failed to convert update info")
}
