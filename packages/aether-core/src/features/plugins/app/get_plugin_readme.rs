use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::PluginId;
use crate::features::plugins::{PluginError, PluginStorage};

use super::ports::GetPluginReadmeUseCasePort;

pub(crate) struct GetPluginReadmeUseCase {
    plugin_storage: Arc<dyn PluginStorage>,
}

impl GetPluginReadmeUseCase {
    pub(crate) fn new(plugin_storage: Arc<dyn PluginStorage>) -> Self {
        Self { plugin_storage }
    }

    pub(crate) async fn execute(&self, plugin_id: PluginId) -> Result<Option<String>, PluginError> {
        self.plugin_storage.read_readme(&plugin_id).await
    }
}

#[async_trait]
impl GetPluginReadmeUseCasePort for GetPluginReadmeUseCase {
    async fn execute(&self, plugin_id: PluginId) -> Result<Option<String>, PluginError> {
        self.execute(plugin_id).await
    }
}
