use async_trait::async_trait;

use crate::features::plugins::{PLUGIN_API_VERSION, PluginError};

use super::ports::GetPluginApiVersionUseCasePort;

#[derive(Default)]
pub(crate) struct GetPluginApiVersionUseCase {}

impl GetPluginApiVersionUseCase {
    pub(crate) async fn execute(&self) -> Result<semver::Version, PluginError> {
        Ok(PLUGIN_API_VERSION.clone())
    }
}

#[async_trait]
impl GetPluginApiVersionUseCasePort for GetPluginApiVersionUseCase {
    async fn execute(&self) -> Result<semver::Version, PluginError> {
        self.execute().await
    }
}
