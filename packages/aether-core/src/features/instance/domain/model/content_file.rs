use std::collections::HashMap;

use crate::features::instance::{PackFile, ProviderId};

use super::ContentType;

#[derive(Debug, Clone)]
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
    pub update: Option<HashMap<ProviderId, ContentFileUpdateInfo>>,
}

#[derive(Debug, Clone)]
pub struct ContentFileUpdateInfo {
    pub content_id: String,
    pub version: String,
}

impl ContentFile {
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
