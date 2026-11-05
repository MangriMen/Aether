use serde::{Deserialize, Serialize};

use crate::v0::ContentTypeDto;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentItemDto {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub long_description: Option<String>,
    pub author: String,
    /// The content's page, an `http` or `https` URL; empty when it has none. Any other value is
    /// dropped by the host.
    pub url: String,
    /// An `https` URL of the icon on a public host; empty when it has none. Any other value, or
    /// one naming `localhost` or a private address, is dropped by the host.
    pub icon_url: String,
    pub versions: Vec<String>,
    pub content_type: ContentTypeDto,
}
