use aether_core::features::instance::EditInstanceIcon;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditInstanceIconDto {
    pub instance_id: String,

    #[specta(optional, type = Option<String>)]
    #[serde(default, with = "::serde_with::rust::double_option")]
    pub icon_path: Option<Option<String>>,
}

impl From<EditInstanceIconDto> for EditInstanceIcon {
    fn from(value: EditInstanceIconDto) -> Self {
        Self {
            instance_id: value.instance_id,
            icon_path: value.icon_path,
        }
    }
}
