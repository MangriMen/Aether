use crate::features::plugins::{PluginError, PluginInternalEvent};

/// Core plugin instance trait with minimal interface.
/// The `call_bytes` method takes raw bytes — serialization is handled
/// by the infra layer via `PluginInstanceExt`.
pub trait PluginInstance: Send + Sync {
    fn get_id(&self) -> String;
    fn supports(&self, name: &str) -> bool;

    fn call_bytes<'b>(&'b mut self, name: &str, args: &[u8]) -> Result<&'b [u8], PluginError>;

    /// Dispatch a lifecycle event to the plugin.
    /// Returns `Ok(())` if the plugin doesn't implement `handle_event`.
    fn handle_event(&mut self, event: &PluginInternalEvent) -> Result<(), PluginError>;
}
