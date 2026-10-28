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
    use rstest::rstest;

    #[rstest]
    #[case::all_fields_none(None, None, None, None, None)]
    #[case::all_fields_some(
        Some(ActionOnInstanceLaunchDto::Hide),
        Some(true),
        Some(WindowEffectDto::Acrylic),
        Some(ActionOnInstanceLaunch::Hide),
        Some(WindowEffect::Acrylic)
    )]
    #[case::partial_fields(None, Some(false), None, None, None)]
    fn edit_dto_into_request(
        #[case] action: Option<ActionOnInstanceLaunchDto>,
        #[case] transparent: Option<bool>,
        #[case] effect: Option<WindowEffectDto>,
        #[case] expected_action: Option<ActionOnInstanceLaunch>,
        #[case] expected_effect: Option<WindowEffect>,
    ) {
        let dto = EditAppSettingsDto {
            action_on_instance_launch: action,
            transparent,
            window_effect: effect,
        };
        let request: EditAppSettingsRequest = dto.into();
        assert_eq!(request.action_on_instance_launch, expected_action);
        assert_eq!(request.transparent, transparent);
        assert_eq!(request.window_effect, expected_effect);
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
