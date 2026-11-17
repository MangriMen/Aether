use aether_core::features::instance::{EditInstanceIcon, InstanceError, InstanceId};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditInstanceIconDto {
    pub instance_id: String,

    #[expect(
        clippy::option_option,
        reason = "`None` leaves the icon, `Some(None)` clears it"
    )]
    #[specta(optional, type = Option<String>)]
    #[serde(default, with = "::serde_with::rust::double_option")]
    pub icon_path: Option<Option<String>>,
}

impl TryFrom<EditInstanceIconDto> for EditInstanceIcon {
    type Error = InstanceError;

    fn try_from(value: EditInstanceIconDto) -> Result<Self, Self::Error> {
        Ok(Self {
            instance_id: InstanceId::try_from(value.instance_id)?,
            icon_path: value.icon_path,
        })
    }
}
