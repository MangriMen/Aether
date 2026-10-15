use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use async_trait::async_trait;

use crate::features::instance::app::ports::CheckContentCompatibilityUseCasePort;
use crate::{
    features::instance::{
        ContentProvider, Instance, InstanceError, InstanceStorage,
        app::{ContentCompatibilityCheckParams, ContentCompatibilityResult},
    },
    shared::capability::domain::CapabilityRegistry,
};

pub struct CheckContentCompatibilityUseCase {
    provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
    instance_storage: Arc<dyn InstanceStorage>,
}

impl CheckContentCompatibilityUseCase {
    pub fn new(
        provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
        instance_storage: Arc<dyn InstanceStorage>,
    ) -> Self {
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
                plugin_id: provider_id.plugin_id.clone(),
                capability_id: provider_id.capability_id.clone(),
            })?;

        let instances: Vec<Instance> = self
            .instance_storage
            .list()
            .await?
            .into_iter()
            .filter(|instance| instance_ids.contains(instance.id()))
            .collect();

        provider
            .capability
            .check_compatibility(&instances, &check_params)
            .await
    }
}

#[async_trait]
impl CheckContentCompatibilityUseCasePort for CheckContentCompatibilityUseCase {
    async fn execute(
        &self,
        instance_ids: HashSet<String>,
        check_params: ContentCompatibilityCheckParams,
    ) -> Result<HashMap<String, ContentCompatibilityResult>, InstanceError> {
        self.execute(instance_ids, check_params).await
    }
}
