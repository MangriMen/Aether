use std::{fmt, ops::Deref, str::FromStr};

use crate::shared::safe_path;

/// The longest id a [`GameVersion`] may have, in bytes. Real ids are at most about 20.
pub const GAME_VERSION_MAX_LEN: usize = 64;

/// The id of a Minecraft version, as the version manifest spells it: `1.21.1`, `24w14a`,
/// `1.14 Pre-Release 1`, `b1.7.3`, `rd-132211`, `3D Shareware v1.34`.
///
/// Ids have no fixed grammar (old, April Fools' and pre-release ids all differ), so any text is
/// accepted that is safe as a single folder name, inside a URL and as a launch argument: the id
/// names the folder of the installed version (`versions/<id>/<id>.jar`). Real ids use only
/// letters, digits, `.`, `-`, `_` and spaces; other ASCII punctuation and invisible format
/// characters are refused, other non-ASCII letters (`20w14∞`) are kept. See
/// [`GameVersionRejection`] for what is refused.
// No `Ord`: the text order of ids is not the order of versions (`1.10` < `1.9`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct GameVersion(String);

/// Why a text is not a [`GameVersion`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum GameVersionRejection {
    #[error("it is empty")]
    Empty,
    #[error("it is longer than {GAME_VERSION_MAX_LEN} bytes")]
    TooLong,
    #[error("it starts or ends with whitespace")]
    SurroundingWhitespace,
    #[error("it contains a control character")]
    ControlCharacter,
    #[error("it contains {0:?}")]
    ForbiddenCharacter(char),
    // Windows drops a trailing dot from a folder name, and `..` must never reach a path.
    #[error("it ends with a dot or contains \"..\"")]
    Dots,
    #[error("it is a reserved Windows device name")]
    ReservedName,
}

/// A text refused by [`GameVersion::parse`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Invalid Minecraft version {value:?}: {reason}")]
pub struct InvalidGameVersion {
    pub value: String,
    pub reason: GameVersionRejection,
}

/// ASCII punctuation no real id uses: separators, characters Windows refuses in a file name,
/// ones that end or escape a URL path, and shell or argument metacharacters.
const FORBIDDEN_ASCII: &str = r#"/\:*?"<>|#%;&'`$,[](){}!@^=+~"#;

/// Invisible format characters (zero-width, bidi overrides and isolates, BOM): they make an id
/// look like another one in a path or a log.
fn is_invisible_format(c: char) -> bool {
    matches!(
        c,
        '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{2069}'
            | '\u{FEFF}'
    )
}

fn check(value: &str) -> Result<(), GameVersionRejection> {
    if value.is_empty() {
        return Err(GameVersionRejection::Empty);
    }
    if value.len() > GAME_VERSION_MAX_LEN {
        return Err(GameVersionRejection::TooLong);
    }
    if value.trim() != value {
        return Err(GameVersionRejection::SurroundingWhitespace);
    }
    if value.chars().any(char::is_control) {
        return Err(GameVersionRejection::ControlCharacter);
    }
    if let Some(c) = value
        .chars()
        .find(|&c| FORBIDDEN_ASCII.contains(c) || is_invisible_format(c))
    {
        return Err(GameVersionRejection::ForbiddenCharacter(c));
    }
    if value.ends_with('.') || value.contains("..") {
        return Err(GameVersionRejection::Dots);
    }
    if safe_path::is_reserved_windows_name(value) {
        return Err(GameVersionRejection::ReservedName);
    }
    Ok(())
}

impl GameVersion {
    pub fn parse(value: &str) -> Result<Self, InvalidGameVersion> {
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

    /// The valid ids of a list a provider gives (the versions a file runs on), in order. An id
    /// that is not a [`GameVersion`] can be the version of no instance, so it is dropped with a
    /// warning instead of failing the whole list; `source` names the list in the warning. A list
    /// of only invalid ids gives an empty list: whatever it belongs to (a content version) is
    /// kept, as running on no known version.
    pub fn parse_list<I>(ids: I, source: &str) -> Vec<Self>
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        ids.into_iter()
            .filter_map(|id| match Self::try_from(id.into()) {
                Ok(version) => Some(version),
                Err(err) => {
                    tracing::warn!(source, error = %err, "Dropping an invalid game version");
                    None
                }
            })
            .collect()
    }
}

impl TryFrom<String> for GameVersion {
    type Error = InvalidGameVersion;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match check(&value) {
            Ok(()) => Ok(Self(value)),
            Err(reason) => Err(InvalidGameVersion { value, reason }),
        }
    }
}

impl TryFrom<&str> for GameVersion {
    type Error = InvalidGameVersion;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl FromStr for GameVersion {
    type Err = InvalidGameVersion;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl From<GameVersion> for String {
    fn from(version: GameVersion) -> Self {
        version.0
    }
}

impl Deref for GameVersion {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for GameVersion {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for GameVersion {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for GameVersion {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl fmt::Display for GameVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_accept_real_ids_when_parsed() {
        for id in [
            "1.21.1",
            "26.1",
            "1.0",
            "24w14a",
            "23w13a_or_b",
            "24w14potato",
            "20w14infinite",
            "20w14∞",
            "1.14 Pre-Release 1",
            "1.14.2 Pre-Release 4",
            "1.14-pre1",
            "1.21-rc1",
            "1.RV-Pre1",
            "3D Shareware v1.34",
            "b1.7.3",
            "b1.8-pre1-2",
            "a1.0.4",
            "a1.2.6",
            "c0.0.11a",
            "c0.30_01c",
            "inf-20100618",
            "rd-132211",
            "rd-20090515",
            "1.20.1-forge-47.2.0",
            "fabric-loader-0.16.10-1.21.4",
            "neoforge-21.1.77",
        ] {
            let version = GameVersion::parse(id).unwrap_or_else(|e| panic!("{id:?}: {e}"));
            assert_eq!(version.as_str(), id);
        }
    }

    #[test]
    fn should_refuse_ids_when_they_are_unsafe_as_a_folder_or_url_part() {
        let too_long = "1".repeat(GAME_VERSION_MAX_LEN + 1);
        for (id, reason) in [
            ("", GameVersionRejection::Empty),
            (too_long.as_str(), GameVersionRejection::TooLong),
            (" 1.21", GameVersionRejection::SurroundingWhitespace),
            ("1.21 ", GameVersionRejection::SurroundingWhitespace),
            ("   ", GameVersionRejection::SurroundingWhitespace),
            ("1.21\n", GameVersionRejection::SurroundingWhitespace),
            ("1.\u{0}21", GameVersionRejection::ControlCharacter),
            ("1\u{1b}21", GameVersionRejection::ControlCharacter),
            ("../1.21", GameVersionRejection::ForbiddenCharacter('/')),
            ("1.21/x", GameVersionRejection::ForbiddenCharacter('/')),
            (r"1.21\x", GameVersionRejection::ForbiddenCharacter('\\')),
            ("C:1.21", GameVersionRejection::ForbiddenCharacter(':')),
            ("1.21?", GameVersionRejection::ForbiddenCharacter('?')),
            ("1.21#a", GameVersionRejection::ForbiddenCharacter('#')),
            ("1.21%2e", GameVersionRejection::ForbiddenCharacter('%')),
            ("1.21*", GameVersionRejection::ForbiddenCharacter('*')),
            ("1<2", GameVersionRejection::ForbiddenCharacter('<')),
            ("a|b", GameVersionRejection::ForbiddenCharacter('|')),
            ("\"1\"", GameVersionRejection::ForbiddenCharacter('"')),
            (".", GameVersionRejection::Dots),
            ("..", GameVersionRejection::Dots),
            ("1.21.", GameVersionRejection::Dots),
            ("1..21", GameVersionRejection::Dots),
            ("CON", GameVersionRejection::ReservedName),
            ("nul.1", GameVersionRejection::ReservedName),
            ("com1", GameVersionRejection::ReservedName),
        ] {
            let err = GameVersion::parse(id).unwrap_err();
            assert_eq!(err.reason, reason, "{id:?}");
            assert_eq!(err.value, id);
        }
    }

    #[test]
    fn should_refuse_ids_when_they_hold_shell_or_argument_metacharacters() {
        for (id, c) in [
            ("1.21;rm", ';'),
            ("1.21&x", '&'),
            ("1.21'", '\''),
            ("1.21`id`", '`'),
            ("$HOME", '$'),
            ("1,21", ','),
            ("[1.21]", '['),
            ("1.21(x)", '('),
            ("{1}", '{'),
            ("1.21!", '!'),
            ("a@b", '@'),
            ("1^2", '^'),
            ("a=b", '='),
            ("1+2", '+'),
            ("~1", '~'),
        ] {
            let err = GameVersion::parse(id).unwrap_err();
            assert_eq!(
                err.reason,
                GameVersionRejection::ForbiddenCharacter(c),
                "{id:?}"
            );
        }
    }

    #[test]
    fn should_refuse_ids_when_they_hold_invisible_format_characters() {
        for c in [
            '\u{200B}', '\u{200F}', '\u{202E}', '\u{2060}', '\u{2066}', '\u{FEFF}',
        ] {
            let id = format!("1.2{c}1");
            let err = GameVersion::parse(&id).unwrap_err();
            assert_eq!(
                err.reason,
                GameVersionRejection::ForbiddenCharacter(c),
                "{id:?}"
            );
        }
    }

    #[test]
    fn should_not_take_a_dotted_id_for_a_device_name_when_its_stem_is_a_number() {
        for id in ["1.21", "1.0", "b1.7.3", "a1.0.4", "26.1"] {
            assert!(GameVersion::parse(id).is_ok(), "{id:?}");
        }
        assert_eq!(
            GameVersion::parse("aux.1").unwrap_err().reason,
            GameVersionRejection::ReservedName
        );
    }

    #[test]
    fn should_drop_only_the_invalid_ids_when_parsing_a_list() {
        let versions = GameVersion::parse_list(["1.21.1", "", "../x", "24w14a"], "test");

        assert_eq!(versions, ["1.21.1", "24w14a"]);
    }

    #[test]
    fn should_keep_the_wire_format_a_plain_string_when_serialized() {
        let version: GameVersion = serde_json::from_str("\"1.21.1\"").unwrap();
        assert_eq!(version, "1.21.1");
        assert_eq!(serde_json::to_string(&version).unwrap(), "\"1.21.1\"");
    }

    #[test]
    fn should_refuse_a_bad_id_when_deserialized() {
        assert!(serde_json::from_str::<GameVersion>("\"../evil\"").is_err());
        assert!(serde_json::from_str::<GameVersion>("\"\"").is_err());
    }
}
