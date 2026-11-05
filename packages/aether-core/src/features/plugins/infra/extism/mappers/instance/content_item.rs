use aether_core_plugin_api::v0::ContentItemDto;

use crate::features::instance::ContentItem;
use crate::shared::web_url::display_url;

/// A link or an icon the item has none of goes to the plugin as an empty string.
impl From<ContentItem> for ContentItemDto {
    fn from(value: ContentItem) -> Self {
        Self {
            id: value.id,
            slug: value.slug,
            name: value.name,
            description: value.description,
            long_description: value.long_description,
            author: value.author,
            url: value.url.map(String::from).unwrap_or_default(),
            icon_url: value.icon_url.map(String::from).unwrap_or_default(),
            versions: value.versions,
            content_type: value.content_type.into(),
        }
    }
}

impl From<ContentItemDto> for ContentItem {
    fn from(value: ContentItemDto) -> Self {
        Self {
            url: display_url("page of a plugin's content item", Some(value.url)),
            icon_url: display_url("icon of a plugin's content item", Some(value.icon_url)),
            id: value.id,
            slug: value.slug,
            name: value.name,
            description: value.description,
            long_description: value.long_description,
            author: value.author,
            versions: value.versions,
            content_type: value.content_type.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use aether_core_plugin_api::v0::ContentTypeDto;

    use super::*;
    use crate::shared::web_url::{HttpUrl, HttpsUrl};

    fn dto(url: &str, icon_url: &str) -> ContentItemDto {
        ContentItemDto {
            id: "238222".into(),
            slug: "jei".into(),
            name: "JEI".into(),
            description: None,
            long_description: None,
            author: "mezz".into(),
            url: url.into(),
            icon_url: icon_url.into(),
            versions: Vec::new(),
            content_type: ContentTypeDto::Mod,
        }
    }

    #[test]
    fn should_keep_the_links_when_they_are_web_urls() {
        let item = ContentItem::from(dto(
            "http://example.com/jei",
            "https://media.forgecdn.net/jei.png",
        ));

        assert_eq!(
            item.url.as_ref().map(HttpUrl::as_str),
            Some("http://example.com/jei")
        );
        assert_eq!(
            item.icon_url.as_ref().map(HttpsUrl::as_str),
            Some("https://media.forgecdn.net/jei.png")
        );
    }

    /// A page is a link the UI opens and an icon one the host fetches: a script, a file or a
    /// cleartext icon is dropped, and the item is kept without it.
    #[test]
    fn should_keep_the_item_without_a_link_when_the_link_is_not_a_web_url() {
        for (url, icon_url) in [
            ("javascript:alert(1)", "http://example.com/jei.png"),
            (
                "file:///C:/Users/me/page.html",
                "data:image/png;base64,AAAA",
            ),
            ("", ""),
        ] {
            let item = ContentItem::from(dto(url, icon_url));

            assert_eq!(item.name, "JEI");
            assert_eq!(item.url, None, "{url}");
            assert_eq!(item.icon_url, None, "{icon_url}");
        }
    }

    #[test]
    fn should_send_no_link_as_an_empty_string_when_mapped_for_a_plugin() {
        let item = ContentItem::from(dto("", ""));

        let dto = ContentItemDto::from(item);

        assert_eq!(dto.url, "");
        assert_eq!(dto.icon_url, "");
    }
}
