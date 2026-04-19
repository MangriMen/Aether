use crate::features::plugins::{PLUGIN_API_VERSION, PluginError};

#[derive(Default)]
pub struct GetPluginApiVersionUseCase {}

impl GetPluginApiVersionUseCase {
    pub async fn execute(&self) -> Result<semver::Version, PluginError> {
        Ok(PLUGIN_API_VERSION.clone())
    }
}
