use std::{
    fmt,
    ops::Deref,
    path::{Component, Path},
};

use serde::{Deserialize, Serialize};

use super::{InstanceError, validate_instance_name};

/// An id that names a directory directly under `instances/`.
///
/// Instance ids also come from plugins and the UI. `..`, a separator or an absolute path would
/// make `LocationInfo::instance_dir` point anywhere in the config directory, e.g. at another
/// plugin's `verification.json`, so whatever takes an id from outside parses it here first.
/// An id that does not parse names no instance, hence [`InstanceError::NotFound`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct InstanceId(String);

const WINDOWS_RESERVED_NAMES: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

impl InstanceId {
    /// A fresh id for a new instance.
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// The id a new instance named `name` gets.
    ///
    /// The name is what the user typed and is shown as it is; the id is the directory it lives
    /// in, so it is derived here rather than validated: a name that cannot be a directory is
    /// sanitized into one. A blank name has no id to derive, hence the validation error.
    pub fn from_name(name: &str) -> Result<Self, InstanceError> {
        validate_instance_name(name)?;

        Self::try_from(sanitize_name(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn is_plain(id: &str) -> bool {
        // `:` makes a drive-relative `C:x` or an alternate data stream on Windows.
        let forbidden: &[char] = if cfg!(windows) {
            &['/', '\\', '\0', ':']
        } else {
            &['/', '\\', '\0']
        };
        if id.is_empty() || id.contains(forbidden) {
            return false;
        }

        let mut components = Path::new(id).components();
        matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
    }
}

impl TryFrom<String> for InstanceId {
    type Error = InstanceError;

    fn try_from(id: String) -> Result<Self, Self::Error> {
        if Self::is_plain(&id) {
            Ok(Self(id))
        } else {
            Err(InstanceError::NotFound { instance_id: id })
        }
    }
}

/// The directory name for an instance name: no characters or endings Windows refuses, and
/// never a reserved device name.
///
/// Applied on every platform, not only on Windows: an instance directory that a config copied
/// to Windows cannot hold is worse than a name that differs from the one the user typed.
fn sanitize_name(name: &str) -> String {
    let replaced: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | '?' | '*' | ':' | '\'' | '\"' | '|' | '<' | '>' | '!' => '_',
            // A control character, `\0` included, is not a name a directory can hold.
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    let trimmed = replaced.trim().trim_end_matches(['.', ' ']);
    if trimmed.is_empty() {
        return "instance".to_owned();
    }

    // `NUL.txt` is the device too: Windows ignores the extension.
    let stem = trimmed.split_once('.').map_or(trimmed, |(stem, _)| stem);
    if WINDOWS_RESERVED_NAMES
        .iter()
        .any(|reserved| reserved.eq_ignore_ascii_case(stem))
    {
        return format!("_{trimmed}");
    }
    trimmed.to_owned()
}

impl TryFrom<&str> for InstanceId {
    type Error = InstanceError;

    fn try_from(id: &str) -> Result<Self, Self::Error> {
        Self::try_from(id.to_owned())
    }
}

impl From<InstanceId> for String {
    fn from(id: InstanceId) -> Self {
        id.0
    }
}

impl Deref for InstanceId {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for InstanceId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::instance::{InstanceField, InstanceValidationErrorReason};

    #[test]
    fn should_accept_an_instance_id_when_it_is_a_single_name() {
        for id in ["inst-1", "My Pack", "pack.v2", ".hidden", "Сборка"] {
            assert_eq!(InstanceId::try_from(id).unwrap().as_str(), id);
        }
    }

    #[test]
    fn should_reject_an_instance_id_when_it_leaves_the_instances_directory() {
        for id in [
            "",
            ".",
            "..",
            "../plugins/x",
            "..\\plugins\\x",
            "a/b",
            "a\\b",
            "/etc",
            "C:\\Windows",
            "\\\\server\\share",
            "a\0b",
        ] {
            let err = InstanceId::try_from(id).unwrap_err();
            assert!(
                matches!(&err, InstanceError::NotFound { instance_id } if instance_id == id),
                "{id:?}: {err:?}"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn should_reject_an_instance_id_when_it_has_a_drive_or_stream_on_windows() {
        for id in ["C:", "C:x", "pack:stream"] {
            assert!(InstanceId::try_from(id).is_err(), "{id:?}");
        }
    }

    #[test]
    fn should_accept_a_generated_id() {
        let id = InstanceId::generate();
        assert_eq!(InstanceId::try_from(id.as_str()).unwrap(), id);
    }

    // ── from_name ──

    #[test]
    fn should_replace_forbidden_characters_when_deriving_an_id() {
        assert_eq!(InstanceId::from_name("a/b:c").unwrap().as_str(), "a_b_c");
        assert_eq!(InstanceId::from_name("a\0b").unwrap().as_str(), "a_b");
    }

    #[test]
    fn should_prefix_a_reserved_windows_name_when_deriving_an_id() {
        assert_eq!(InstanceId::from_name("con").unwrap().as_str(), "_con");
        assert_eq!(
            InstanceId::from_name("NUL.txt").unwrap().as_str(),
            "_NUL.txt"
        );
        assert_eq!(
            InstanceId::from_name("console").unwrap().as_str(),
            "console"
        );
    }

    #[test]
    fn should_strip_trailing_dots_and_spaces_when_deriving_an_id() {
        assert_eq!(
            InstanceId::from_name("My pack. . ").unwrap().as_str(),
            "My pack"
        );
    }

    #[test]
    fn should_fall_back_when_nothing_is_left_of_the_name() {
        assert_eq!(InstanceId::from_name("...").unwrap().as_str(), "instance");
    }

    #[test]
    fn should_refuse_a_blank_name_when_deriving_an_id() {
        let err = InstanceId::from_name("   ").unwrap_err();
        assert!(matches!(
            err,
            InstanceError::ValidationError {
                field: InstanceField::Name,
                reason: InstanceValidationErrorReason::CannotBeEmpty,
            }
        ));
    }

    #[test]
    fn should_serialize_as_a_plain_string_and_check_it_on_the_way_back() {
        let id = InstanceId::try_from("inst-1").unwrap();
        assert_eq!(serde_json::to_string(&id).unwrap(), "\"inst-1\"");
        assert_eq!(
            serde_json::from_str::<InstanceId>("\"inst-1\"").unwrap(),
            id
        );
        assert!(serde_json::from_str::<InstanceId>("\"../x\"").is_err());
    }
}
