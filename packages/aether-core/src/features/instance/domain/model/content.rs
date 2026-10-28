use std::{ops::Deref, str::FromStr};

use crate::features::minecraft::ModLoader;

use super::{ContentType, InstanceId};

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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProviderId {
    pub plugin_id: String,
    pub capability_id: String,
}

impl ProviderId {
    pub const SEPARATOR: char = '#';

    /// The `plugin_id` of the providers built into Aether.
    ///
    /// Reserved: a plugin declaring it is refused, so no plugin can pass itself off as a
    /// built-in provider.
    pub const BUILTIN_PLUGIN_ID: &'static str = "aether";

    /// Whether this is a provider built into Aether rather than one a plugin registered.
    ///
    /// Safe to decide trust on: [`Self::BUILTIN_PLUGIN_ID`] is reserved, so a plugin cannot
    /// claim it.
    pub fn is_builtin(&self) -> bool {
        self.plugin_id == Self::BUILTIN_PLUGIN_ID
    }
}

/// Whether `value` fits the scheme shared by `plugin_id`, `capability_id` and platform names:
/// `[a-z0-9][a-z0-9_-]*`.
///
/// The scheme leaves no room for [`ProviderId::SEPARATOR`] or `:`, so the string form of a
/// [`ProviderId`] always parses back into the same pair.
pub fn is_valid_id(value: &str) -> bool {
    let mut chars = value.chars();

    chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
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
        let invalid = || {
            format!(
                "Invalid format '{}'. Expected plugin{}capability",
                s,
                Self::SEPARATOR
            )
        };

        let (plugin, cap) = s.split_once(Self::SEPARATOR).ok_or_else(invalid)?;
        if plugin.is_empty() || cap.is_empty() {
            return Err(invalid());
        }

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
    fn should_display_provider_id_when_formatted() {
        let id = ProviderId {
            plugin_id: "my_plugin".to_string(),
            capability_id: "mod_provider".to_string(),
        };
        assert_eq!(id.to_string(), "my_plugin#mod_provider");
    }

    #[test]
    fn should_parse_provider_id_when_string_is_valid() {
        let id: ProviderId = "my_plugin#mod_provider".parse().unwrap();
        assert_eq!(id.plugin_id, "my_plugin");
        assert_eq!(id.capability_id, "mod_provider");
    }

    #[test]
    fn should_fail_to_parse_provider_id_when_separator_is_missing() {
        let err = "my_plugin".parse::<ProviderId>().unwrap_err();
        assert!(err.contains("Invalid format"));
        assert!(err.contains("my_plugin"));
    }

    #[test]
    fn should_fail_to_parse_provider_id_when_a_part_is_empty() {
        for value in ["#only_cap", "only_plugin#", "#"] {
            let err = value.parse::<ProviderId>().unwrap_err();
            assert!(err.contains("Invalid format"), "{value}: {err}");
        }
    }

    #[test]
    fn should_roundtrip_provider_id_when_displayed_and_parsed() {
        let original = ProviderId {
            plugin_id: "test".to_string(),
            capability_id: "cap".to_string(),
        };
        let serialized = original.to_string();
        let deserialized: ProviderId = serialized.parse().unwrap();
        assert_eq!(original.plugin_id, deserialized.plugin_id);
        assert_eq!(original.capability_id, deserialized.capability_id);
    }

    #[test]
    fn should_accept_ids_when_they_match_the_scheme() {
        for id in [
            "aether",
            "packwiz",
            "curseforge_plugin",
            "modrinth-content",
            "0x",
            "a",
        ] {
            assert!(is_valid_id(id), "{id:?} should be valid");
        }
    }

    #[test]
    fn should_reject_ids_when_they_are_outside_the_scheme() {
        for id in [
            "",
            "core:modrinth",
            "a#b",
            "Packwiz",
            "AETHER",
            "-leading-dash",
            "_leading_underscore",
            "with space",
            "dotted.id",
            "../escape",
            "кириллица",
        ] {
            assert!(!is_valid_id(id), "{id:?} should be invalid");
        }
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
    pub instance_id: InstanceId,
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
pub struct ModpackInstallParams {
    pub base: BaseContentParams,
}

impl Deref for ModpackInstallParams {
    type Target = BaseContentParams;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

#[derive(Clone, Debug)]
pub enum ContentInstallParams {
    Atomic(AtomicInstallParams),
    Modpack(ModpackInstallParams),
}

impl ContentInstallParams {
    pub fn provider(&self) -> &ProviderId {
        match self {
            Self::Atomic(p) => &p.base.provider_id,
            Self::Modpack(p) => &p.base.provider_id,
        }
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
