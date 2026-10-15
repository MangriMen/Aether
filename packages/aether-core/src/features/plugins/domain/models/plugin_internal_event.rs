#[derive(Clone, Debug)]
pub enum PluginInternalEvent {
    Loaded,
    Unloaded,
    BeforeInstanceLaunch { instance_id: String },
    AfterInstanceLaunch { instance_id: String },
}
