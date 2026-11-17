use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::v0::ContentTypeDto;

#[derive(Debug, Clone, Serialize, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentItemDto {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub long_description: Option<String>,
    pub author: String,
    /// The content's page, an `http` or `https` URL; absent when it has none. Any other value is
    /// dropped by the host.
    #[serde(default)]
    pub url: Option<String>,
    /// An `https` URL of the icon on a public host; absent when it has none. Any other value, or
    /// one naming `localhost` or a private address, is dropped by the host.
    #[serde(default)]
    pub icon_url: Option<String>,
    pub versions: Vec<String>,
    pub content_type: ContentTypeDto,
}
