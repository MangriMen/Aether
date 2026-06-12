#[derive(Clone, Debug)]
pub enum PluginInternalEvent {
    BeforeInstanceLaunch { instance_id: String },
    AfterInstanceLaunch { instance_id: String },
}
