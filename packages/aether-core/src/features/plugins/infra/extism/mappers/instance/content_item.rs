use aether_core_plugin_api::v0::ContentItemDto;

use super::UnknownValue;
use crate::features::instance::ContentItem;
use crate::shared::web_url::{display_icon_url, display_url};

impl From<ContentItem> for ContentItemDto {
    fn from(value: ContentItem) -> Self {
        Self {
            id: value.id,
            slug: value.slug,
            name: value.name,
            description: value.description,
            long_description: value.long_description,
            author: value.author,
            url: value.url.map(String::from),
            icon_url: value.icon_url.map(String::from),
            versions: value.versions,
            content_type: value.content_type.into(),
        }
    }
}

impl TryFrom<ContentItemDto> for ContentItem {
    type Error = UnknownValue;

    fn try_from(value: ContentItemDto) -> Result<Self, Self::Error> {
        Ok(Self {
            url: display_url("page of a plugin's content item", value.url),
            icon_url: display_icon_url("icon of a plugin's content item", value.icon_url),
            id: value.id,
            slug: value.slug,
            name: value.name,
            description: value.description,
            long_description: value.long_description,
            author: value.author,
            versions: value.versions,
            content_type: value.content_type.try_into()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use aether_core_plugin_api::v0::ContentTypeDto;

    use super::*;
    use crate::shared::web_url::{HttpUrl, HttpsUrl};

    fn dto(url: Option<&str>, icon_url: Option<&str>) -> ContentItemDto {
        ContentItemDto {
            id: "238222".into(),
            slug: "jei".into(),
            name: "JEI".into(),
            description: None,
            long_description: None,
            author: "mezz".into(),
            url: url.map(Into::into),
            icon_url: icon_url.map(Into::into),
            versions: Vec::new(),
            content_type: ContentTypeDto::Mod,
        }
    }

    #[test]
    fn should_keep_the_links_when_they_are_web_urls() {
        let item = ContentItem::try_from(dto(
            Some("http://example.com/jei"),
            Some("https://media.forgecdn.net/jei.png"),
        ))
        .unwrap();

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
    /// cleartext icon is dropped, and the item is kept without it. So is an empty string, which
    /// a plugin written before the links were optional sends for none.
    #[test]
    fn should_keep_the_item_without_a_link_when_the_link_is_not_a_web_url() {
        for (url, icon_url) in [
            (
                Some("javascript:alert(1)"),
                Some("http://example.com/jei.png"),
            ),
            (
                Some("file:///C:/Users/me/page.html"),
                Some("data:image/png;base64,AAAA"),
            ),
            (Some(""), Some("")),
            (None, None),
        ] {
            let item = ContentItem::try_from(dto(url, icon_url)).unwrap();

            assert_eq!(item.name, "JEI");
            assert_eq!(item.url, None, "{url:?}");
            assert_eq!(item.icon_url, None, "{icon_url:?}");
        }
    }

    #[test]
    fn should_send_no_link_as_absent_when_mapped_for_a_plugin() {
        let item = ContentItem::try_from(dto(None, None)).unwrap();

        let dto = ContentItemDto::from(item);

        assert_eq!(dto.url, None);
        assert_eq!(dto.icon_url, None);
    }
}
