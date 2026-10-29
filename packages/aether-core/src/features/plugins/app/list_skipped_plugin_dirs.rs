use std::sync::Arc;

use async_trait::async_trait;

use crate::features::plugins::{PluginError, PluginStorage, SkippedPluginDir};

use super::ports::ListSkippedPluginDirsUseCasePort;

pub(crate) struct ListSkippedPluginDirsUseCase {
    plugin_storage: Arc<dyn PluginStorage>,
}

impl ListSkippedPluginDirsUseCase {
    pub(crate) fn new(plugin_storage: Arc<dyn PluginStorage>) -> Self {
        Self { plugin_storage }
    }

    pub(crate) async fn execute(&self) -> Result<Vec<SkippedPluginDir>, PluginError> {
        self.plugin_storage.list_skipped_dirs().await
    }
}

#[async_trait]
impl ListSkippedPluginDirsUseCasePort for ListSkippedPluginDirsUseCase {
    async fn execute(&self) -> Result<Vec<SkippedPluginDir>, PluginError> {
        self.execute().await
    }
}
