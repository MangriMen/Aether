use std::collections::HashMap;

use crate::features::instance::{ContentMetadata, ContentVersionMetadata, PackFile, ProviderId};

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
    /// What the provider of the file's project says about it, as last cached: the project of
    /// `update[update_provider_id].content_id`. `None` — nothing cached yet, or the file is of
    /// no provider.
    pub metadata: Option<ContentMetadata>,
    /// The local copy of `metadata`'s icon, as an id of the asset store: the file's path
    /// relative to the store, `/`-separated, with a `?v=` query that changes when a file under
    /// that path may have been replaced, so the UI does not keep showing an old copy. Only when
    /// the file is there: `None` while the icon has not arrived, or there is none.
    pub icon_path: Option<String>,
    /// What the provider of the file says about the version it was installed as, as last
    /// cached: the version of `update[update_provider_id].version`. `None` — nothing cached
    /// yet, or the file is of no provider.
    pub version_metadata: Option<ContentVersionMetadata>,
    /// Whether `version_metadata` is being fetched now, and an instance event will tell when the
    /// fetch is over. `false` when nothing is to come: the file is of no provider, its provider
    /// cannot tell or does not know the version, or asking it failed lately.
    pub version_metadata_pending: bool,
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
            metadata: None,
            icon_path: None,
            version_metadata: None,
            version_metadata_pending: false,
        }
    }
}
