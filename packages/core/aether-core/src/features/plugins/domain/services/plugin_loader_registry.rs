use std::collections::HashMap;

use crate::features::plugins::{LoadConfigType, PluginError, PluginLoader};

#[derive(Default)]
pub struct PluginLoaderRegistry<PL> {
    loaders: HashMap<LoadConfigType, PL>,
}

impl<PL: PluginLoader> PluginLoaderRegistry<PL> {
    pub fn new(loaders: HashMap<LoadConfigType, PL>) -> Self {
        Self { loaders }
    }

    pub fn get(&self, load_config_type: &LoadConfigType) -> Result<&PL, PluginError> {
        self.loaders
            .get(load_config_type)
            .ok_or(PluginError::LoaderNotFound {
                config_type: *load_config_type,
            })
    }

    pub fn register(&mut self, load_config_type: LoadConfigType, provider: PL) {
        self.loaders.insert(load_config_type, provider);
    }

    pub fn unregister(&mut self, load_config_type: &LoadConfigType) {
        self.loaders.remove(load_config_type);
    }
}
