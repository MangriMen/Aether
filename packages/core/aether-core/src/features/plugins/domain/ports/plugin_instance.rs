use crate::features::plugins::PluginError;

pub trait PluginInstance: Send + Sync {
    fn get_id(&self) -> String;
    fn supports(&self, name: &str) -> bool;

    fn on_load(&mut self) -> Result<(), PluginError>;
    fn on_unload(&mut self) -> Result<(), PluginError>;

    fn call_bytes<'b>(&'b mut self, name: &str, args: &[u8]) -> Result<&'b [u8], PluginError>;
}

pub trait PluginInstanceExt: PluginInstance {
    fn call<'a, 'b, T: extism::ToBytes<'a>, U: extism::FromBytes<'b>>(
        &'b mut self,
        name: &str,
        args: T,
    ) -> Result<U, PluginError> {
        let id = self.get_id();
        let map_err = |e: extism::Error| -> PluginError {
            PluginError::FunctionCallFailed {
                function_name: name.to_owned(),
                plugin_id: id.to_owned(),
                error: e.to_string(),
            }
        };

        use extism::{FromBytes as F, ToBytes as T};

        let args_b = T::to_bytes(&args).map_err(map_err)?;
        let result_b = self.call_bytes(name, args_b.as_ref())?;
        let result = F::from_bytes(result_b).map_err(map_err)?;
        Ok(result)
    }
}

impl<T: PluginInstance + ?Sized> PluginInstanceExt for T {}
