use crate::features::plugins::{PluginError, PluginInstance};

/// Typed call helper for Extism plugins.
/// Converts args/result using extism's `ToBytes`/`FromBytes` (`MessagePack`).
/// This lives in the infra layer because it depends on extism serialization.
pub trait PluginInstanceExt: PluginInstance {
    fn call<'a, 'b, T: extism::ToBytes<'a>, U: extism::FromBytes<'b>>(
        &'b mut self,
        name: &str,
        args: T,
    ) -> Result<U, PluginError> {
        use extism::{FromBytes as F, ToBytes as T};

        let id = self.get_id();
        let map_err = |e: extism::Error| -> PluginError {
            PluginError::FunctionCallFailed {
                function_name: name.to_owned(),
                plugin_id: id.clone(),
                error: e.to_string(),
            }
        };

        let args_b = T::to_bytes(&args).map_err(map_err)?;
        let result_b = self.call_bytes(name, args_b.as_ref())?;
        let result = F::from_bytes(result_b).map_err(map_err)?;
        Ok(result)
    }
}

impl<T: PluginInstance + ?Sized> PluginInstanceExt for T {}
