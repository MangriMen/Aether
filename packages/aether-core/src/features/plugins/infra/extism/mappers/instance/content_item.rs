use aether_core_plugin_api::v0::ContentItemDto;

use crate::features::instance::ContentItem;

impl From<ContentItem> for ContentItemDto {
    fn from(value: ContentItem) -> Self {
        Self {
            id: value.id,
            slug: value.slug,
            name: value.name,
            description: value.description,
            long_description: value.long_description,
            author: value.author,
            url: value.url,
            icon_url: value.icon_url,
            versions: value.versions,
            content_type: value.content_type.into(),
        }
    }
}

impl From<ContentItemDto> for ContentItem {
    fn from(value: ContentItemDto) -> Self {
        Self {
            id: value.id,
            slug: value.slug,
            name: value.name,
            description: value.description,
            long_description: value.long_description,
            author: value.author,
            url: value.url,
            icon_url: value.icon_url,
            versions: value.versions,
            content_type: value.content_type.into(),
        }
    }
}
