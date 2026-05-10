use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    features::instance::{Instance, InstanceError, InstanceStorage},
    shared::AssetProcessor,
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

pub struct EditInstanceIconUseCase<IS, AP> {
    instance_storage: Arc<IS>,
    asset_processor: Arc<AP>,
}

impl<IS: InstanceStorage, AP: AssetProcessor> EditInstanceIconUseCase<IS, AP> {
    pub fn new(instance_storage: Arc<IS>, asset_processor: Arc<AP>) -> Self {
        Self {
            instance_storage,
            asset_processor,
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
                        .asset_processor
                        .import_file(icon_path)
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
