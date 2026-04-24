use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub action_on_instance_launch: ActionOnInstanceLaunch,
    pub transparent: bool,
    pub window_effect: WindowEffect,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ActionOnInstanceLaunch {
    #[default]
    Nothing,
    Hide,
    Close,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WindowEffect {
    #[default]
    Off,
    MicaLight,
    MicaDark,
    Mica,
    Acrylic,
}
