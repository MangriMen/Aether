use super::PluginInstanceExt;
use crate::features::instance::PluginId;
use crate::features::plugins::{
    PluginCallError, PluginError, PluginInstance, PluginInternalEvent,
    infra::extism::call_error::classify_call_error,
};

#[derive(Debug)]
pub(crate) struct ExtismPluginInstance {
    id: PluginId,
    inner: extism::Plugin,
}

impl ExtismPluginInstance {
    pub(crate) fn new(plugin: extism::Plugin, id: PluginId) -> Self {
        Self { inner: plugin, id }
    }
}

impl PluginInstance for ExtismPluginInstance {
    fn get_id(&self) -> PluginId {
        self.id.clone()
    }
    fn supports(&self, name: &str) -> bool {
        self.inner.function_exists(name)
    }

    fn call_bytes<'b>(&'b mut self, name: &str, args: &[u8]) -> Result<&'b [u8], PluginError> {
        let failed = |error| PluginError::FunctionCallFailed {
            function_name: name.to_owned(),
            plugin_id: self.id.clone(),
            error,
        };

        // Extism reports a missing export with the same code as an error the plugin returned,
        // so it is ruled out before the call rather than told apart after it.
        if !self.inner.function_exists(name) {
            return Err(failed(PluginCallError::MissingFunction));
        }

        self.inner
            .call_get_error_code(name, args)
            .map_err(|(error, code)| {
                tracing::debug!(
                    "Call to \"{name}\" in plugin \"{}\" failed with code {code}: {error:?}",
                    self.id
                );
                failed(classify_call_error(&error, code))
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
