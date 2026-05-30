use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::settings::EditAppSettingsRequest;

use super::{ActionOnInstanceLaunchDto, WindowEffectDto};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditAppSettingsDto {
    // #[serde(skip_serializing_if = "Option::is_none")]
    #[specta(optional)]
    pub action_on_instance_launch: Option<ActionOnInstanceLaunchDto>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    #[specta(optional)]
    pub transparent: Option<bool>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    #[specta(optional)]
    pub window_effect: Option<WindowEffectDto>,
}

impl From<EditAppSettingsDto> for EditAppSettingsRequest {
    fn from(value: EditAppSettingsDto) -> Self {
        Self {
            action_on_instance_launch: value.action_on_instance_launch.map(Into::into),
            transparent: value.transparent,
            window_effect: value.window_effect.map(Into::into),
        }
    }
}
