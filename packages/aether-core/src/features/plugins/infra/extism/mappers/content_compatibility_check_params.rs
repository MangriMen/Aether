use aether_core_plugin_api::v0::ContentCompatibilityCheckParamsDto;

use crate::features::instance::ContentCompatibilityCheckParams;

impl From<ContentCompatibilityCheckParams> for ContentCompatibilityCheckParamsDto {
    fn from(value: ContentCompatibilityCheckParams) -> Self {
        Self {
            provider_id: value.provider_id.into(),
            content_item: value.content_item.into(),
        }
    }
}
