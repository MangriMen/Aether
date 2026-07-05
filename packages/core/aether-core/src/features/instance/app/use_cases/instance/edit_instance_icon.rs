use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    features::instance::{Instance, InstanceError, InstanceStorage},
    shared::cache::domain::AssetsStorage,
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

pub struct EditInstanceIconUseCase {
    instance_storage: Arc<dyn InstanceStorage>,
    assets_storage: Arc<dyn AssetsStorage>,
}

impl EditInstanceIconUseCase {
    pub fn new(
        instance_storage: Arc<dyn InstanceStorage>,
        assets_storage: Arc<dyn AssetsStorage>,
    ) -> Self {
        Self {
            instance_storage,
            assets_storage,
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
                    let asset_id = self
                        .assets_storage
                        .import_file(icon_path.as_ref())
                        .await
                        .map_err(|err| InstanceError::Storage(err.to_string()))?;

                    instance.set_icon(Some(asset_id));
                }
                None => instance.set_icon(None),
            }
        }

        self.instance_storage.upsert(&instance).await?;

        Ok(instance)
    }
}
