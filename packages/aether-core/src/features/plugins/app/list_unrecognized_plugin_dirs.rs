use std::sync::Arc;

use async_trait::async_trait;

use crate::features::plugins::{PluginError, PluginStorage};

use super::ports::ListUnrecognizedPluginDirsUseCasePort;

pub(crate) struct ListUnrecognizedPluginDirsUseCase {
    plugin_storage: Arc<dyn PluginStorage>,
}

impl ListUnrecognizedPluginDirsUseCase {
    pub(crate) fn new(plugin_storage: Arc<dyn PluginStorage>) -> Self {
        Self { plugin_storage }
    }

    pub(crate) async fn execute(&self) -> Result<Vec<String>, PluginError> {
        self.plugin_storage.list_unrecognized_dirs().await
    }
}

#[async_trait]
impl ListUnrecognizedPluginDirsUseCasePort for ListUnrecognizedPluginDirsUseCase {
    async fn execute(&self) -> Result<Vec<String>, PluginError> {
        self.execute().await
    }
}
