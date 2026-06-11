use std::collections::HashMap;

use crate::features::instance::{ContentFile, ContentFileUpdateInfo, ProviderId};

#[derive(Debug, Clone, Default)]
pub struct Pack {
    pub files: Vec<PackEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct PackEntry {
    pub file: String,
}

#[derive(Debug, Clone)]
pub struct PackFile {
    pub file_name: String,
    pub name: Option<String>,
    pub hash: String,
    pub download: Option<PackFileDownload>,
    pub option: Option<PackFileOption>,
    pub side: Option<String>,
    pub update_provider_id: Option<ProviderId>,
    pub update: Option<HashMap<ProviderId, ContentFileUpdateInfo>>,
}

#[derive(Debug, Clone)]
pub struct PackFileDownload {
    /// File hash for verification
    pub hash: String,
    /// URL to download the file from
    pub url: String,
}

#[derive(Debug, Clone)]
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
}
