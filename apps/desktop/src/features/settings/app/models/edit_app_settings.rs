use crate::features::settings::{ActionOnInstanceLaunch, WindowEffect};

#[derive(Debug, Clone)]
pub struct EditAppSettingsRequest {
    pub action_on_instance_launch: Option<ActionOnInstanceLaunch>,

    pub transparent: Option<bool>,

    pub window_effect: Option<WindowEffect>,
}
