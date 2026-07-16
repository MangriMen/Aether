use std::{ops::Deref, str::FromStr};

use crate::features::minecraft::ModLoader;

use super::ContentType;

#[derive(Clone, Debug)]
pub struct ContentSearchParams {
    pub content_type: ContentType,
    pub provider_id: ProviderId,
    pub page: i64,
    pub page_size: i64,
    pub query: Option<String>,
    pub game_versions: Option<Vec<String>>,
    pub loader: Option<ModLoader>,
}

#[derive(Clone, Debug)]
pub struct ContentSearchResult {
    pub page: i64,
    pub page_size: i64,
    pub page_count: i64,
    pub provider_id: ProviderId,
    pub items: Vec<ContentItem>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ProviderId {
    pub plugin_id: String,
    pub capability_id: String,
}

impl ProviderId {
    pub const SEPARATOR: char = '#';
}

impl std::fmt::Display for ProviderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.plugin_id,
            Self::SEPARATOR,
            self.capability_id
        )
    }
}

impl FromStr for ProviderId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (plugin, cap) = s.split_once(Self::SEPARATOR).ok_or_else(|| {
            format!(
                "Invalid format '{}'. Expected plugin{}capability",
                s,
                Self::SEPARATOR
            )
        })?;

        Ok(Self {
            plugin_id: plugin.to_string(),
            capability_id: cap.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_id_display() {
        let id = ProviderId {
            plugin_id: "my_plugin".to_string(),
            capability_id: "mod_provider".to_string(),
        };
        assert_eq!(id.to_string(), "my_plugin#mod_provider");
    }

    #[test]
    fn provider_id_from_str_valid() {
        let id: ProviderId = "my_plugin#mod_provider".parse().unwrap();
        assert_eq!(id.plugin_id, "my_plugin");
        assert_eq!(id.capability_id, "mod_provider");
    }

    #[test]
    fn provider_id_from_str_invalid_no_separator() {
        let err = "my_plugin".parse::<ProviderId>().unwrap_err();
        assert!(err.contains("Invalid format"));
        assert!(err.contains("my_plugin"));
    }

    #[test]
    fn provider_id_from_str_empty_parts() {
        let id: ProviderId = "#only_cap".parse().unwrap();
        assert!(id.plugin_id.is_empty());
        assert_eq!(id.capability_id, "only_cap");
    }

    #[test]
    fn provider_id_roundtrip() {
        let original = ProviderId {
            plugin_id: "test".to_string(),
            capability_id: "cap".to_string(),
        };
        let serialized = original.to_string();
        let deserialized: ProviderId = serialized.parse().unwrap();
        assert_eq!(original.plugin_id, deserialized.plugin_id);
        assert_eq!(original.capability_id, deserialized.capability_id);
    }
}

#[derive(Clone, Debug)]
pub struct BaseContentParams {
    pub content_id: String,
    pub content_version: Option<String>,
    pub provider_id: ProviderId,
}

#[derive(Clone, Debug)]
pub struct AtomicInstallParams {
    pub base: BaseContentParams,
    pub instance_id: String,
    pub game_version: String,
    pub loader: Option<String>,
    pub content_type: ContentType,
}

impl Deref for AtomicInstallParams {
    type Target = BaseContentParams;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

#[derive(Clone, Debug)]
pub struct ContentItem {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub long_description: Option<String>,
    pub author: String,
    pub url: String,
    pub icon_url: String,
    pub versions: Vec<String>,
    pub content_type: ContentType,
}
