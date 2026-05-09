use std::{path::Path, sync::Arc};

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::{
    features::instance::{Instance, InstanceError, InstanceStorage},
    shared::{CacheId, CacheKey, FileStore, read_async, sha1_async},
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditInstanceIcon {
    pub instance_id: String,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub icon_path: Option<Option<String>>,
}

pub const ASSETS_CACHE_NAMESPACE: &str = "assets";

pub struct EditInstanceIconUseCase<IS, FS> {
    instance_storage: Arc<IS>,
    assets_cache: Arc<FS>,
}

impl<IS: InstanceStorage, FS: FileStore> EditInstanceIconUseCase<IS, FS> {
    pub fn new(instance_storage: Arc<IS>, assets_cache: Arc<FS>) -> Self {
        Self {
            instance_storage,
            assets_cache,
        }
    }

    pub async fn execute(
        &self,
        edit_instance_icon: EditInstanceIcon,
    ) -> Result<Instance, InstanceError> {
        let mut instance = self
            .instance_storage
            .get(&edit_instance_icon.instance_id)
            .await?;

        if let Some(icon_path) = edit_instance_icon.icon_path {
            match icon_path {
                Some(icon_path) => {
                    let file_extension = Path::new(&icon_path)
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("png");

                    let raw_bytes = read_async(&icon_path)
                        .await
                        .map_err(|err| InstanceError::Storage(err.to_string()))?;

                    let bytes = Bytes::from_owner(raw_bytes);

                    let hash = sha1_async(bytes.clone()).await;

                    let internal_name = format!("{hash}.{file_extension}");

                    let cache_key = CacheKey::new(
                        ASSETS_CACHE_NAMESPACE,
                        CacheId::Named(internal_name.clone()),
                    );

                    if !self.assets_cache.exists(&cache_key).await {
                        self.assets_cache.write(&cache_key, bytes).await;
                    }

                    instance.icon_path = Some(internal_name);
                }
                None => instance.icon_path = None,
            }
        }

        self.instance_storage.upsert(&instance).await?;

        Ok(instance)
    }
}
