use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;

use crate::{
    features::instance::{ContentFile, ContentFileUpdateInfo, ProviderId},
    shared::sha1_async,
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Pack {
    pub files: Vec<PackEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct PackEntry {
    pub file: String,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PackFile {
    pub file_name: String,
    pub name: Option<String>,
    pub hash: String,
    pub download: Option<PackFileDownload>,
    pub option: Option<PackFileOption>,
    pub side: Option<String>,
    pub update_provider_id: Option<ProviderId>,
    #[serde_as(as = "Option<HashMap<DisplayFromStr, _>>")]
    pub update: Option<HashMap<ProviderId, ContentFileUpdateInfo>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PackFileDownload {
    /// File hash for verification
    pub hash: String,
    /// URL to download the file from
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PackFileOption {
    /// Whether this file is optional
    pub optional: bool,
    /// Download by default if the file is optional
    pub default: Option<bool>,
    pub description: Option<String>,
}

impl From<ContentFile> for PackFile {
    fn from(value: ContentFile) -> Self {
        Self {
            name: value.name,
            file_name: value.file_name,
            hash: value.hash,
            download: None,
            option: None,
            side: None,
            update_provider_id: value.update_provider_id,
            update: value.update,
        }
    }
}

impl PackFile {
    pub fn from_hash(file_name: String, hash: String) -> Self {
        Self {
            name: None,
            file_name,
            hash,
            download: None,
            option: None,
            side: None,
            update_provider_id: None,
            update: None,
        }
    }

    pub async fn from_contents(file_name: String, contents: Vec<u8>) -> Self {
        Self::from_hash(file_name, sha1_async(contents).await)
    }
}
