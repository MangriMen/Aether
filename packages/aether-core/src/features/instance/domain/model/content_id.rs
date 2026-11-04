use std::{fmt, ops::Deref, str::FromStr};

use super::InstanceError;

/// A value refused by [`ContentId::try_from`] or [`ContentVersionId::try_from`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "Invalid {kind} {value:?}: must be non-empty, without surrounding whitespace, control characters or any of / \\ ? # %"
)]
pub struct InvalidContentId {
    pub kind: &'static str,
    pub value: String,
}

impl From<InvalidContentId> for InstanceError {
    fn from(err: InvalidContentId) -> Self {
        InstanceError::InvalidContentId {
            kind: err.kind.to_owned(),
            value: err.value,
        }
    }
}

fn is_valid(value: &str) -> bool {
    !value.is_empty()
        && value.trim() == value
        && !value
            .chars()
            .any(|c| c.is_control() || matches!(c, '/' | '\\' | '?' | '#' | '%'))
}

macro_rules! define_content_id {
    ($(#[$meta:meta])* $name:ident, $kind:literal) => {
        $(#[$meta])*
        #[derive(
            Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
        )]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = InvalidContentId;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                if is_valid(&value) {
                    Ok(Self(value))
                } else {
                    Err(InvalidContentId { kind: $kind, value })
                }
            }
        }

        impl TryFrom<&str> for $name {
            type Error = InvalidContentId;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::try_from(value.to_owned())
            }
        }

        impl FromStr for $name {
            type Err = InvalidContentId;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::try_from(value)
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl Deref for $name {
            type Target = str;

            fn deref(&self) -> &str {
                &self.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

define_content_id!(
    /// The id a content provider gives a piece of content (a project, mod or pack).
    ContentId,
    "content id"
);

define_content_id!(
    /// The id a content provider gives one version of a piece of content.
    ContentVersionId,
    "content version id"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_accept_ids_when_they_follow_the_rule() {
        for ok in ["AANobbMI", "my-slug_1.2"] {
            assert_eq!(ContentId::try_from(ok).unwrap().as_str(), ok);
            assert_eq!(ContentVersionId::try_from(ok).unwrap().as_str(), ok);
        }
    }

    #[test]
    fn should_refuse_ids_when_they_break_the_rule() {
        for bad in [
            "", " a", "a ", "a/b", "a\\b", "a?b", "a#b", "a%2e", "a\u{0}b", "a\nb",
        ] {
            let err = ContentId::try_from(bad).unwrap_err();
            assert_eq!(err.value, bad);
            assert_eq!(err.kind, "content id");
            assert!(ContentVersionId::try_from(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn should_refuse_a_bad_id_when_deserialized() {
        assert!(serde_json::from_str::<ContentId>("\"a/b\"").is_err());
        let id: ContentId = serde_json::from_str("\"abc\"").unwrap();
        assert_eq!(serde_json::to_string(&id).unwrap(), "\"abc\"");
    }
}
