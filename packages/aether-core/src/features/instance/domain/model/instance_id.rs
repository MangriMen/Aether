use std::{
    fmt,
    ops::Deref,
    path::{Component, Path},
};

use serde::{Deserialize, Serialize};

use super::InstanceError;

/// An id that names a directory directly under `instances/`.
///
/// Instance ids also come from plugins and the UI. `..`, a separator or an absolute path would
/// make `LocationInfo::instance_dir` point anywhere in the config directory, e.g. at another
/// plugin's `verification.json`, so whatever takes an id from outside parses it here first.
/// An id that does not parse names no instance, hence [`InstanceError::NotFound`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct InstanceId(String);

impl InstanceId {
    /// A fresh id for a new instance.
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
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
