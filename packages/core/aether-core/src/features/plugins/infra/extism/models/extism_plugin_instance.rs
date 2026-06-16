use super::PluginInstanceExt;
use crate::features::plugins::{PluginError, PluginInstance, PluginInternalEvent};

#[derive(Debug)]
pub struct ExtismPluginInstance {
    id: String,
    inner: extism::Plugin,
}

impl ExtismPluginInstance {
    pub fn new(plugin: extism::Plugin, id: String) -> Self {
        Self { inner: plugin, id }
    }
}

impl PluginInstance for ExtismPluginInstance {
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn supports(&self, name: &str) -> bool {
        self.inner.function_exists(name)
    }

    fn call_bytes<'b>(&'b mut self, name: &str, args: &[u8]) -> Result<&'b [u8], PluginError> {
        self.inner
            .call(name, args)
            .map_err(|e| PluginError::FunctionCallFailed {
                function_name: name.to_owned(),
                plugin_id: self.id.clone(),
                error: e.to_string(),
            })
    }

    fn handle_event(&mut self, event: &PluginInternalEvent) -> Result<(), PluginError> {
        let handle = "handle_event";

        if !self.supports(handle) {
            return Ok(());
        }

        let dto: aether_core_plugin_api::v0::PluginInternalEventDto = event.into();
        self.call::<extism_convert::Msgpack<_>, ()>(handle, extism_convert::Msgpack(dto))
    }
}
