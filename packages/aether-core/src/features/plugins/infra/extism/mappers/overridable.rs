use aether_core_plugin_api::v0::OverridableDto;

use crate::shared::overridable::Overridable;

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
