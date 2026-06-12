#[derive(Debug, Clone)]
pub enum PluginEvent {
    Add { plugin_id: String },
    Edit { plugin_id: String },
    Remove { plugin_id: String },
    Sync,
}
