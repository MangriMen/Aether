use std::{fmt, ops::Deref, str::FromStr};

/// The longest id a [`LoaderVersionId`] may have, in bytes. Real ids are at most about 30
/// (`1.7.10-10.13.4.1614-1.7.10`).
pub const LOADER_VERSION_ID_MAX_LEN: usize = 128;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum LoaderVersionPreference {
    Latest,
    #[default]
    Stable,
    Exact(LoaderVersionId),
}

impl LoaderVersionPreference {
    /// An exact preference read back from storage. An id that is not a valid
    /// [`LoaderVersionId`] is dropped with a warning instead of making the whole record
    /// unreadable: the instance is kept, and its loader version is chosen again. `source` names
    /// the record in the warning.
    pub fn exact_from_stored(id: String, source: &str) -> Option<Self> {
        match LoaderVersionId::try_from(id) {
            Ok(id) => Some(Self::Exact(id)),
            Err(err) => {
                tracing::warn!(source, error = %err, "Dropping an invalid stored loader version");
                None
            }
        }
    }
}

/// The id of a mod loader version, as the loader's manifest spells it: `0.16.10`, `21.1.77`,
/// `1.20.1-47.2.0`, `20.2.3-beta`.
///
/// Ids have no common grammar across loaders, so any single word is accepted: an id is only
/// compared with the ids of a loader manifest, never parsed. See [`LoaderVersionIdRejection`]
/// for what is refused.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct LoaderVersionId(String);

/// Why a text is not a [`LoaderVersionId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum LoaderVersionIdRejection {
    #[error("it is empty")]
    Empty,
    #[error("it is longer than {LOADER_VERSION_ID_MAX_LEN} bytes")]
    TooLong,
    #[error("it contains a control character")]
    ControlCharacter,
    #[error("it contains whitespace")]
    Whitespace,
}

/// A text refused by [`LoaderVersionId::parse`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Invalid loader version {value:?}: {reason}")]
pub struct InvalidLoaderVersionId {
    pub value: String,
    pub reason: LoaderVersionIdRejection,
}

fn check(value: &str) -> Result<(), LoaderVersionIdRejection> {
    if value.is_empty() {
        return Err(LoaderVersionIdRejection::Empty);
    }
    if value.len() > LOADER_VERSION_ID_MAX_LEN {
        return Err(LoaderVersionIdRejection::TooLong);
    }
    // Before whitespace: `\n` and `\t` are both.
    if value.chars().any(char::is_control) {
        return Err(LoaderVersionIdRejection::ControlCharacter);
    }
    if value.chars().any(char::is_whitespace) {
        return Err(LoaderVersionIdRejection::Whitespace);
    }
    Ok(())
}

impl LoaderVersionId {
    pub fn parse(value: &str) -> Result<Self, InvalidLoaderVersionId> {
        Self::try_from(value.to_owned())
    }

    /// # Panics
    ///
    /// If `value` is not a valid id: a literal that is not one is a bug.
    pub fn from_static(value: &'static str) -> Self {
        Self::parse(value).unwrap_or_else(|err| panic!("{err}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for LoaderVersionId {
    type Error = InvalidLoaderVersionId;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match check(&value) {
            Ok(()) => Ok(Self(value)),
            Err(reason) => Err(InvalidLoaderVersionId { value, reason }),
        }
    }
}

impl TryFrom<&str> for LoaderVersionId {
    type Error = InvalidLoaderVersionId;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl FromStr for LoaderVersionId {
    type Err = InvalidLoaderVersionId;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl From<LoaderVersionId> for String {
    fn from(id: LoaderVersionId) -> Self {
        id.0
    }
}

impl Deref for LoaderVersionId {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for LoaderVersionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for LoaderVersionId {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for LoaderVersionId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl fmt::Display for LoaderVersionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_refuse_an_id_when_it_is_empty_or_holds_whitespace() {
        for (id, reason) in [
            ("", LoaderVersionIdRejection::Empty),
            (" ", LoaderVersionIdRejection::Whitespace),
            (" 0.16.10", LoaderVersionIdRejection::Whitespace),
            ("0.16.10 ", LoaderVersionIdRejection::Whitespace),
            ("0.16 10", LoaderVersionIdRejection::Whitespace),
            ("\u{a0}0.16.10", LoaderVersionIdRejection::Whitespace),
            ("0.16.10\n", LoaderVersionIdRejection::ControlCharacter),
            ("0.16\u{0}", LoaderVersionIdRejection::ControlCharacter),
        ] {
            let err = LoaderVersionId::parse(id).unwrap_err();
            assert_eq!(err.reason, reason, "{id:?}");
            assert_eq!(err.value, id);
        }
    }

    #[test]
    fn should_accept_real_ids_when_parsed() {
        let longest = "1".repeat(LOADER_VERSION_ID_MAX_LEN);
        for id in [
            "0.16.10",
            "21.1.77",
            "1.20.1-47.2.0",
            "1.7.10-10.13.4.1614-1.7.10",
            "20.2.3-beta",
            "0.26.4+build.1",
            longest.as_str(),
        ] {
            let parsed = LoaderVersionId::parse(id).unwrap_or_else(|e| panic!("{id:?}: {e}"));
            assert_eq!(parsed.as_str(), id);
        }
    }

    #[test]
    fn should_refuse_an_id_when_it_is_longer_than_the_limit() {
        let too_long = "1".repeat(LOADER_VERSION_ID_MAX_LEN + 1);

        assert_eq!(
            LoaderVersionId::parse(&too_long).unwrap_err().reason,
            LoaderVersionIdRejection::TooLong
        );
    }

    #[test]
    fn should_keep_the_wire_format_a_plain_string_when_serialized() {
        let id: LoaderVersionId = serde_json::from_str("\"0.16.10\"").unwrap();

        assert_eq!(id, "0.16.10");
        assert_eq!(serde_json::to_string(&id).unwrap(), "\"0.16.10\"");
        assert!(serde_json::from_str::<LoaderVersionId>("\" 0.16\"").is_err());
    }

    #[test]
    fn should_drop_the_preference_when_the_stored_exact_id_is_invalid() {
        assert_eq!(
            LoaderVersionPreference::exact_from_stored("0.16 .5".into(), "test"),
            None
        );
        assert_eq!(
            LoaderVersionPreference::exact_from_stored("0.16.5".into(), "test"),
            Some(LoaderVersionPreference::Exact(
                LoaderVersionId::from_static("0.16.5")
            ))
        );
    }
}
