use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::features::instance::app::IconAssetReaper;
use crate::features::instance::app::ports::EditInstanceIconUseCasePort;
use crate::features::instance::{Instance, InstanceError, InstanceId, InstanceStorage};

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
    icon_reaper: IconAssetReaper,
}

impl EditInstanceIconUseCase {
    pub fn new(instance_storage: Arc<dyn InstanceStorage>, icon_reaper: IconAssetReaper) -> Self {
        Self {
            instance_storage,
            icon_reaper,
        }
    }

    pub async fn execute(
        &self,
        edit_instance_icon: EditInstanceIcon,
    ) -> Result<Instance, InstanceError> {
        let instance_id = &edit_instance_icon.instance_id;
        // Fail on a missing instance before importing the icon, or its asset would be left behind.
        self.instance_storage.get(instance_id).await?;

        let mut previous_icon = None;
        let instance = {
            // Until the instance stores the icon, a release must not delete a reused file.
            let guard = self.icon_reaper.hold().await;

            let new_icon = match &edit_instance_icon.icon_path {
                Some(Some(icon_path)) => Some(Some(
                    self.icon_reaper
                        .import_file(&guard, icon_path.as_ref())
                        .await
                        .map_err(|err| InstanceError::Storage(err.to_string()))?,
                )),
                Some(None) => Some(None),
                None => None,
            };

            self.instance_storage
                .update(
                    instance_id,
                    Box::new(|instance| {
                        let Some(icon) = new_icon else {
                            return Ok(false);
                        };
                        previous_icon = instance.icon_path().map(ToOwned::to_owned);
                        instance.set_icon(icon);
                        Ok(true)
                    }),
                )
                .await?
        };

        if let Some(previous) = previous_icon
            && instance.icon_path() != Some(previous.as_str())
        {
            self.icon_reaper.release(&previous).await;
        }
        Ok(instance)
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
