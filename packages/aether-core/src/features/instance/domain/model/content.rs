use std::{ops::Deref, str::FromStr};

use crate::features::minecraft::ModLoader;

use super::{CapabilityId, ContentType, InstanceId, InvalidId, PluginId};

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
    pub plugin_id: PluginId,
    pub capability_id: CapabilityId,
}

impl ProviderId {
    pub const SEPARATOR: char = '#';

    /// The `plugin_id` of the providers built into Aether.
    ///
    /// Reserved: a plugin declaring it is refused, so no plugin can pass itself off as a
    /// built-in provider.
    pub const BUILTIN_PLUGIN_ID: &'static str = "aether";

    pub fn new(plugin_id: PluginId, capability_id: CapabilityId) -> Self {
        Self {
            plugin_id,
            capability_id,
        }
    }

    /// The provider of `capability_id` in `plugin_id`, both still to be checked.
    pub fn from_parts(plugin_id: &str, capability_id: &str) -> Result<Self, InvalidId> {
        Ok(Self::new(plugin_id.parse()?, capability_id.parse()?))
    }

    /// Whether this is a provider built into Aether rather than one a plugin registered.
    ///
    /// Safe to decide trust on: [`Self::BUILTIN_PLUGIN_ID`] is reserved, so a plugin cannot
    /// claim it.
    pub fn is_builtin(&self) -> bool {
        self.plugin_id.as_str() == Self::BUILTIN_PLUGIN_ID
    }
}

/// A string that is not the `plugin#capability` form of a [`ProviderId`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProviderIdError {
    #[error("Invalid format '{0}'. Expected plugin{sep}capability", sep = ProviderId::SEPARATOR)]
    Format(String),
    #[error(transparent)]
    InvalidPart(#[from] InvalidId),
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
    type Err = ProviderIdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (plugin, cap) = s
            .split_once(Self::SEPARATOR)
            .ok_or_else(|| ProviderIdError::Format(s.to_owned()))?;
        if plugin.is_empty() || cap.is_empty() {
            return Err(ProviderIdError::Format(s.to_owned()));
        }

        Ok(Self::from_parts(plugin, cap)?)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn should_display_provider_id_when_formatted() {
        let id = ProviderId::new(
            PluginId::from_static("my_plugin"),
            CapabilityId::from_static("mod_provider"),
        );
        assert_eq!(id.to_string(), "my_plugin#mod_provider");
    }

    #[test]
    fn should_parse_provider_id_when_string_is_valid() {
        let id: ProviderId = "my_plugin#mod_provider".parse().unwrap();
        assert_eq!(id.plugin_id.as_str(), "my_plugin");
        assert_eq!(id.capability_id.as_str(), "mod_provider");
    }

    #[rstest]
    #[case("aether#modrinth")]
    #[case("packwiz#packwiz")]
    #[case("0x#mod-provider_2")]
    fn should_parse_back_the_same_id_when_displayed(#[case] value: &str) {
        let id: ProviderId = value.parse().unwrap();
        assert_eq!(id.to_string(), value);
        assert_eq!(id.to_string().parse::<ProviderId>().unwrap(), id);
    }

    #[test]
    fn should_fail_to_parse_provider_id_when_separator_is_missing() {
        let err = "my_plugin".parse::<ProviderId>().unwrap_err();
        assert_eq!(err, ProviderIdError::Format("my_plugin".to_owned()));
        assert!(err.to_string().contains("Invalid format"));
        assert!(err.to_string().contains("my_plugin"));
    }

    #[rstest]
    #[case("#only_cap")]
    #[case("only_plugin#")]
    #[case("#")]
    fn should_fail_to_parse_provider_id_when_a_part_is_empty(#[case] value: &str) {
        let err = value.parse::<ProviderId>().unwrap_err();
        assert_eq!(err, ProviderIdError::Format(value.to_owned()));
    }

    #[rstest]
    #[case("core:modrinth#modrinth-content", "plugin", "core:modrinth")]
    #[case("Bad Id#cap", "plugin", "Bad Id")]
    #[case("plugin#a#b", "capability", "a#b")]
    #[case("plugin#Cap", "capability", "Cap")]
    fn should_fail_to_parse_provider_id_when_a_part_is_not_an_id(
        #[case] value: &str,
        #[case] kind: &'static str,
        #[case] part: &str,
    ) {
        let err = value.parse::<ProviderId>().unwrap_err();
        assert_eq!(
            err,
            ProviderIdError::InvalidPart(InvalidId {
                kind,
                value: part.to_owned(),
            }),
            "{value}"
        );
    }

    #[rstest]
    #[case("aether#modrinth", true)]
    #[case("aetherx#modrinth", false)]
    #[case("packwiz#aether", false)]
    fn should_tell_a_builtin_provider_when_its_plugin_is_aether(
        #[case] value: &str,
        #[case] is_builtin: bool,
    ) {
        assert_eq!(
            value.parse::<ProviderId>().unwrap().is_builtin(),
            is_builtin
        );
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
    pub loader: Option<ModLoader>,
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
