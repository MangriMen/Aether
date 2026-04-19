use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    features::instance::{
        ContentProvider, Instance, InstanceError, InstanceStorage,
        app::{ContentCompatibilityCheckParams, ContentCompatibilityResult},
    },
    shared::CapabilityRegistry,
};

pub struct CheckContentCompatibilityUseCase<
    CP: CapabilityRegistry<Arc<dyn ContentProvider>>,
    IS: InstanceStorage,
> {
    provider_registry: Arc<CP>,
    instance_storage: Arc<IS>,
}

impl<CP: CapabilityRegistry<Arc<dyn ContentProvider>>, IS: InstanceStorage>
    CheckContentCompatibilityUseCase<CP, IS>
{
    pub fn new(provider_registry: Arc<CP>, instance_storage: Arc<IS>) -> Self {
        Self {
            provider_registry,
            instance_storage,
        }
    }

    pub async fn execute(
        &self,
        instance_ids: HashSet<String>,
        check_params: ContentCompatibilityCheckParams,
    ) -> Result<HashMap<String, ContentCompatibilityResult>, InstanceError> {
        let provider_id = &check_params.provider_id;

        let provider = self
            .provider_registry
            .find_by_plugin_and_capability_id(&provider_id.plugin_id, &provider_id.capability_id)
            .await
            .map_err(|_| InstanceError::ContentProviderNotFound {
                plugin_id: provider_id.plugin_id.to_string(),
                capability_id: provider_id.capability_id.to_string(),
            })?;

        let instances: Vec<Instance> = self
            .instance_storage
            .list()
            .await?
            .into_iter()
            .filter(|instance| instance_ids.contains(&instance.id))
            .collect();

        provider
            .capability
            .check_compatibility(&instances, &check_params)
            .await
    }
}
