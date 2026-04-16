use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::settings::{ActionOnInstanceLaunch, WindowEffect};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditAppSettingsDto {
    // #[serde(skip_serializing_if = "Option::is_none")]
    #[specta(optional)]
    pub action_on_instance_launch: Option<ActionOnInstanceLaunch>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    #[specta(optional)]
    pub transparent: Option<bool>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    #[specta(optional)]
    pub window_effect: Option<WindowEffect>,
}
