use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Why a command was not run.
#[derive(Debug, Serialize, JsonSchema, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CommandRefusalDto {
    /// The user said no, or said it earlier and the plugin has not been reloaded since.
    Denied,
    /// Nobody answered in time.
    TimedOut,
    /// The plugin was stopped or removed.
    Cancelled,
    /// The jar changed while the user was being asked.
    JarChanged,
    /// A value this version of the API does not know: a newer launcher sent it. Read it, never
    /// write it.
    #[serde(other)]
    Unknown,
}
