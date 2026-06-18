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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::settings::{ActionOnInstanceLaunch, WindowEffect};

    #[test]
    fn edit_dto_all_fields_none() {
        let dto = EditAppSettingsDto {
            action_on_instance_launch: None,
            transparent: None,
            window_effect: None,
        };
        let request: EditAppSettingsRequest = dto.into();
        assert!(request.action_on_instance_launch.is_none());
        assert!(request.transparent.is_none());
        assert!(request.window_effect.is_none());
    }

    #[test]
    fn edit_dto_all_fields_some() {
        let dto = EditAppSettingsDto {
            action_on_instance_launch: Some(ActionOnInstanceLaunchDto::Hide),
            transparent: Some(true),
            window_effect: Some(WindowEffectDto::Acrylic),
        };
        let request: EditAppSettingsRequest = dto.into();
        assert_eq!(
            request.action_on_instance_launch,
            Some(ActionOnInstanceLaunch::Hide)
        );
        assert_eq!(request.transparent, Some(true));
        assert_eq!(request.window_effect, Some(WindowEffect::Acrylic));
    }

    #[test]
    fn edit_dto_partial_fields() {
        let dto = EditAppSettingsDto {
            action_on_instance_launch: None,
            transparent: Some(false),
            window_effect: None,
        };
        let request: EditAppSettingsRequest = dto.into();
        assert!(request.action_on_instance_launch.is_none());
        assert_eq!(request.transparent, Some(false));
        assert!(request.window_effect.is_none());
    }

    #[test]
    fn edit_dto_serialize_camel_case() {
        let dto = EditAppSettingsDto {
            action_on_instance_launch: Some(ActionOnInstanceLaunchDto::Close),
            transparent: Some(true),
            window_effect: None,
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("actionOnInstanceLaunch"));
        assert!(
            json.contains("windowEffect"),
            "Should contain windowEffect field (null): {json}"
        );
    }

    #[test]
    fn edit_dto_deserialize_camel_case() {
        let json = r#"{
            "actionOnInstanceLaunch": "hide",
            "windowEffect": "mica"
        }"#;
        let dto: EditAppSettingsDto = serde_json::from_str(json).unwrap();
        assert_eq!(
            dto.action_on_instance_launch,
            Some(ActionOnInstanceLaunchDto::Hide)
        );
        assert!(dto.transparent.is_none());
        assert_eq!(dto.window_effect, Some(WindowEffectDto::Mica));
    }
}
