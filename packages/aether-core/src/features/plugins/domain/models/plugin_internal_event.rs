use crate::features::instance::InstanceId;

#[derive(Clone, Debug)]
pub enum PluginInternalEvent {
    Loaded,
    Unloaded,
    BeforeInstanceLaunch { instance_id: InstanceId },
    AfterInstanceLaunch { instance_id: InstanceId },
}
