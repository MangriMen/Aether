use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::features::settings::{ActionOnInstanceLaunch, WindowEffect};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "index.ts")]
pub struct EditAppSettingsDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub action_on_instance_launch: Option<ActionOnInstanceLaunch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub transparent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub window_effect: Option<WindowEffect>,
}
