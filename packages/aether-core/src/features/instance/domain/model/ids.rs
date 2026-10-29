use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

/// An id outside the scheme of [`is_valid_id`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Invalid {kind} id `{value}`: expected [a-z0-9][a-z0-9_-]*")]
pub struct InvalidId {
    pub kind: &'static str,
    pub value: String,
}

/// Whether `value` fits the scheme shared by `plugin_id`, `capability_id` and platform names:
/// `[a-z0-9][a-z0-9_-]*`.
///
/// The scheme leaves no room for [`ProviderId::SEPARATOR`] or `:`, so the string form of a
/// [`ProviderId`] always parses back into the same pair.
///
/// [`ProviderId`]: super::ProviderId
/// [`ProviderId::SEPARATOR`]: super::ProviderId::SEPARATOR
pub fn is_valid_id(value: &str) -> bool {
    let mut chars = value.chars();

    chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident, $kind:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            /// An id spelled out in the code.
            ///
            /// # Panics
            ///
            /// If `value` is outside the scheme: a literal that is not an id is a bug.
            pub fn from_static(value: &'static str) -> Self {
                assert!(is_valid_id(value), "invalid {} id literal `{value}`", $kind);
                Self(value.to_owned())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = InvalidId;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                if is_valid_id(&value) {
                    Ok(Self(value))
                } else {
                    Err(InvalidId { kind: $kind, value })
                }
            }
        }

        impl TryFrom<&str> for $name {
            type Error = InvalidId;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::try_from(value.to_owned())
            }
        }

        impl FromStr for $name {
            type Err = InvalidId;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::try_from(value)
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> Self {
                id.0
            }
        }





        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

define_id!(
    /// The id a plugin declares in its manifest; also names its directory under `plugins/`.
    PluginId,
    "plugin"
);

define_id!(
    /// The id of a capability within the plugin that declares it.
    CapabilityId,
    "capability"
);

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("aether")]
    #[case("packwiz")]
    #[case("curseforge_plugin")]
    #[case("modrinth-content")]
    #[case("0x")]
    #[case("a")]
    fn should_accept_an_id_when_it_matches_the_scheme(#[case] value: &str) {
        assert!(is_valid_id(value));
        assert_eq!(PluginId::try_from(value).unwrap().as_str(), value);
        assert_eq!(CapabilityId::try_from(value).unwrap().as_str(), value);
    }

    #[rstest]
    #[case("")]
    #[case("core:modrinth")]
    #[case("a:b")]
    #[case("a#b")]
    #[case("Packwiz")]
    #[case("AETHER")]
    #[case("-leading-dash")]
    #[case("_leading_underscore")]
    #[case("with space")]
    #[case("dotted.id")]
    #[case("..")]
    #[case("../escape")]
    #[case("кириллица")]
    fn should_reject_an_id_when_it_is_outside_the_scheme(#[case] value: &str) {
        assert!(!is_valid_id(value));
        assert_eq!(
            PluginId::try_from(value).unwrap_err(),
            InvalidId {
                kind: "plugin",
                value: value.to_owned()
            }
        );
        assert_eq!(
            value.parse::<CapabilityId>().unwrap_err().kind,
            "capability"
        );
    }

    #[test]
    fn should_name_the_kind_and_the_value_when_an_id_is_refused() {
        assert_eq!(
            PluginId::try_from("Bad Id").unwrap_err().to_string(),
            "Invalid plugin id `Bad Id`: expected [a-z0-9][a-z0-9_-]*"
        );
    }

    #[test]
    fn should_serialize_as_a_plain_string_when_round_tripped() {
        let id = PluginId::try_from("packwiz").unwrap();
        assert_eq!(serde_json::to_string(&id).unwrap(), "\"packwiz\"");
        assert_eq!(serde_json::from_str::<PluginId>("\"packwiz\"").unwrap(), id);
    }

    #[rstest]
    #[case("\"\"")]
    #[case("\"a:b\"")]
    #[case("\"..\"")]
    #[case("\"Upper\"")]
    fn should_refuse_to_deserialize_when_the_id_is_invalid(#[case] json: &str) {
        assert!(serde_json::from_str::<PluginId>(json).is_err());
        assert!(serde_json::from_str::<CapabilityId>(json).is_err());
    }

    #[test]
    fn should_display_the_id_when_formatted() {
        assert_eq!(PluginId::from_static("modrinth").to_string(), "modrinth");
    }

    #[test]
    #[should_panic(expected = "invalid capability id literal")]
    fn should_panic_when_a_literal_is_not_an_id() {
        let _ = CapabilityId::from_static("Not An Id");
    }
}
