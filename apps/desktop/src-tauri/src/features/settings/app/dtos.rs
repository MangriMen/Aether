use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::features::settings::{ActionOnInstanceLaunch, WindowEffect};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "index.ts", rename = "EditAppSettingsDto")]
pub struct EditAppSettings {
    pub action_on_instance_launch: Option<ActionOnInstanceLaunch>,
    pub transparent: Option<bool>,
    pub window_effect: Option<WindowEffect>,
}
