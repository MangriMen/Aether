use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, de::IgnoredAny};

use super::{CommandRefusalDto, HttpErrorDto};

#[derive(Serialize, JsonSchema, Deserialize, Debug, Clone)]
#[serde(tag = "status", content = "data", rename_all = "camelCase")]
pub enum HostResult<T> {
    Ok(T),
    Err(HostError),
}

impl<T> HostResult<T> {
    /// What a host function returned, as a `Result`.
    pub fn into_result(self) -> Result<T, HostError> {
        match self {
            HostResult::Ok(value) => Ok(value),
            HostResult::Err(error) => Err(error),
        }
    }
}

/// Why a host function failed. The kind tells a plugin what it can do about it; `message` is
/// text for the log or the user, not something to match on.
#[derive(Debug, Serialize, JsonSchema, Deserialize, Clone, PartialEq, Eq)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum HostError {
    /// What the plugin sent is refused: an invalid id, path, version or page, a value the host
    /// does not know or does not support. Sending it again fails again.
    InvalidArgument { message: String },
    /// What the plugin named does not exist: an instance, a plugin, an importer, a provider.
    NotFound { message: String },
    /// The plugin may not do this: a path outside what it was given, a program it is not
    /// allowed. A host an HTTP function refuses is `http` with `hostNotAllowed` instead.
    AccessDenied { message: String },
    /// The user did not allow the plugin's command, or was not there to.
    CommandNotApproved {
        reason: CommandRefusalDto,
        message: String,
    },
    /// A file or an answer is larger than the host accepts.
    TooLarge { message: String },
    /// The state does not allow it now: the instance is running or still installing, a plugin
    /// is loading or busy. It may succeed later.
    Conflict { message: String },
    /// The work was called off before it finished.
    Cancelled { message: String },
    /// The work did not finish in the time it was given.
    TimedOut { message: String },
    /// A call into a plugin failed: the other plugin crashed, hit a limit or reported an error.
    PluginCallFailed { message: String },
    /// A network request failed; `error` tells how.
    Http {
        message: String,
        error: HttpErrorDto,
    },
    /// The host failed on its own: storage, the file system, anything the plugin can do nothing
    /// about.
    Internal { message: String },
    /// A kind this version of the API does not know: a newer launcher sent it. Read it, never
    /// write it.
    // `other` alone reads only a kind without `data`; every kind has a `message`, so a newer
    // one comes with `data`, which is stepped over.
    #[serde(other, deserialize_with = "ignore_data")]
    Unknown,
}

impl HostError {
    /// The text of the error; `None` for [`HostError::Unknown`], which carries none this version
    /// can read.
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::InvalidArgument { message }
            | Self::NotFound { message }
            | Self::AccessDenied { message }
            | Self::CommandNotApproved { message, .. }
            | Self::TooLarge { message }
            | Self::Conflict { message }
            | Self::Cancelled { message }
            | Self::TimedOut { message }
            | Self::PluginCallFailed { message }
            | Self::Http { message, .. }
            | Self::Internal { message } => Some(message),
            Self::Unknown => None,
        }
    }
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            self.message()
                .unwrap_or("the host failed in a way this API does not know"),
        )
    }
}

/// The `data` of a kind this version does not know: whatever it holds, or none.
pub(super) fn ignore_data<'de, D: Deserializer<'de>>(deserializer: D) -> Result<(), D::Error> {
    IgnoredAny::deserialize(deserializer).map(|_| ())
}
