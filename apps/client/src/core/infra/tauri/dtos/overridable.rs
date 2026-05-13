use aether_core::shared::Overridable;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct OverridableDto<T> {
    pub is_active: bool,
    pub data: T,
}

impl<T, U> From<Overridable<T>> for OverridableDto<U>
where
    T: Into<U>,
{
    fn from(value: Overridable<T>) -> Self {
        Self {
            is_active: value.is_active,
            data: value.data.into(),
        }
    }
}
