use std::{collections::HashMap, path::PathBuf};

use path_slash::PathBufExt;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::features::instance::{PackFile, ProviderId};

use super::ContentType;

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContentFile {
    pub content_path: String,
    pub content_type: ContentType,
    pub disabled: bool,
    pub file_name: String,
    pub hash: String,
    pub name: Option<String>,
    pub size: u64,
    pub version: Option<String>,
    pub update_provider_id: Option<ProviderId>,
    #[serde_as(as = "Option<HashMap<DisplayFromStr, _>>")]
    pub update: Option<HashMap<ProviderId, ContentFileUpdateInfo>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContentFileUpdateInfo {
    pub content_id: String,
    pub version: String,
}

pub struct CreateContentFileParams {
    pub name: Option<String>,
    pub file_name: String,
    pub size: u64,
    pub sha1: String,
    pub content_path: PathBuf,
    pub content_id: String,
    pub content_version: String,
    pub content_type: ContentType,
    pub provider_id: ProviderId,
}

impl ContentFile {
    pub fn from_params(params: CreateContentFileParams) -> Self {
        let update_info = ContentFileUpdateInfo {
            content_id: params.content_id,
            version: params.content_version.clone(),
        };

        let update = Some(HashMap::from([(params.provider_id.clone(), update_info)]));

        let path = params.content_path.to_slash_lossy().to_string();

        Self {
            content_path: path,
            name: params.name,
            hash: params.sha1,
            file_name: params.file_name,
            size: params.size,
            content_type: params.content_type,
            disabled: false,
            version: Some(params.content_version),
            update_provider_id: Some(params.provider_id),
            update,
        }
    }

    pub fn from_pack_file(
        pack_file: PackFile,
        content_path: String,
        content_type: ContentType,
        size: u64,
        disabled: bool,
    ) -> Self {
        let version = pack_file
            .update_provider_id
            .as_ref()
            .and_then(|id| pack_file.update.as_ref()?.get(id))
            .map(|info| info.version.clone());

        Self {
            content_path,
            name: pack_file.name,
            hash: pack_file.hash,
            file_name: pack_file.file_name,
            content_type,
            size,
            disabled,
            version,
            update_provider_id: pack_file.update_provider_id,
            update: pack_file.update,
        }
    }
}
