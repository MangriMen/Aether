use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::features::instance::app::IconAssetReaper;
use crate::features::instance::app::ports::EditInstanceIconUseCasePort;
use crate::{
    features::instance::{Instance, InstanceError, InstanceId, InstanceStorage},
    shared::cache::domain::AssetsStorage,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditInstanceIcon {
    pub instance_id: InstanceId,

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
    icon_reaper: IconAssetReaper,
}

impl EditInstanceIconUseCase {
    pub fn new(
        instance_storage: Arc<dyn InstanceStorage>,
        assets_storage: Arc<dyn AssetsStorage>,
    ) -> Self {
        Self {
            icon_reaper: IconAssetReaper::new(instance_storage.clone(), assets_storage.clone()),
            instance_storage,
            assets_storage,
        }
    }

    pub async fn execute(
        &self,
        edit_instance_icon: EditInstanceIcon,
    ) -> Result<Instance, InstanceError> {
        let instance_id = &edit_instance_icon.instance_id;
        let previous_icon = self
            .instance_storage
            .get(instance_id)
            .await?
            .icon_path()
            .map(ToOwned::to_owned);

        let new_icon = match &edit_instance_icon.icon_path {
            Some(Some(icon_path)) => Some(Some(
                self.assets_storage
                    .import_file(icon_path.as_ref())
                    .await
                    .map_err(|err| InstanceError::Storage(err.to_string()))?,
            )),
            Some(None) => Some(None),
            None => None,
        };

        let updated = self
            .instance_storage
            .update(instance_id, &mut |instance| {
                if let Some(icon) = &new_icon {
                    instance.set_icon(icon.clone());
                }
                Ok(true)
            })
            .await?;

        if let Some(previous) = previous_icon
            && updated.icon_path() != Some(previous.as_str())
        {
            self.icon_reaper.release(&previous).await;
        }
        Ok(updated)
    }
}

#[async_trait]
impl EditInstanceIconUseCasePort for EditInstanceIconUseCase {
    async fn execute(
        &self,
        edit_instance_icon: EditInstanceIcon,
    ) -> Result<Instance, InstanceError> {
        self.execute(edit_instance_icon).await
    }
}
