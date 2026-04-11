use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::features::settings::{ActionOnInstanceLaunch, WindowEffect};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "index.ts", rename = "EditAppSettingsDto")]
pub struct EditAppSettings {
    #[ts(optional)]
    pub action_on_instance_launch: Option<ActionOnInstanceLaunch>,
    #[ts(optional)]
    pub transparent: Option<bool>,
    #[ts(optional)]
    pub window_effect: Option<WindowEffect>,
}
