use crate::features::plugins::{PluginError, PLUGIN_API_VERSION};

#[derive(Default)]
pub struct GetPluginApiVersionUseCase {}

impl GetPluginApiVersionUseCase {
    pub async fn execute(&self) -> Result<semver::Version, PluginError> {
        Ok(PLUGIN_API_VERSION.clone())
    }
}
