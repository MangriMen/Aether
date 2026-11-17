use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::error::ignore_data;

/// How a network request failed: the `error` of the `http` kind of `HostError`. A plugin matches
/// on the kind; a `reason` is text for the log or the user, like the `message`.
#[derive(Debug, Serialize, JsonSchema, Deserialize, Clone, PartialEq, Eq)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum HttpErrorDto {
    /// The host is not in the plugin's `allowedHosts`: the URL asked for, or a redirect to it.
    /// The user can allow it in the plugin's settings.
    HostNotAllowed { host: String },
    /// The host resolves to an address a plugin may not reach (loopback, private, link-local,
    /// …), or to none at all.
    AddressBlocked { host: String },
    /// The request itself is refused before anything is sent: the URL, its scheme or host, a
    /// header, the JSON body or its size, the expected hash of a download. Sending it again fails
    /// again.
    InvalidRequest { reason: String },
    /// A redirect that is not followed: to `http` from `https`, without a usable `Location`, more
    /// than the limit, or any 3xx where none is followed (`http_post_json`, `304`, …).
    Redirect { reason: String },
    /// The answer is larger than the host accepts, in bytes.
    ResponseTooLarge { limit: u64 },
    /// The final answer has an error status. `http_get` and `http_post_json` hand a status back
    /// as an answer instead; a download, and the launcher's own requests for a plugin, fail with it.
    Status { status: u16 },
    /// The downloaded file is not the one the hash names.
    HashMismatch,
    /// The answer could not be read as what was expected: not the JSON, TOML or text the launcher
    /// asked for, or a URL in it that cannot be used.
    InvalidResponse { reason: String },
    /// No usable answer came back: the host could not be resolved or reached, the connection or
    /// TLS failed, the request timed out or broke off. It may succeed later.
    Network { reason: String },
    /// A kind this version of the API does not know: a newer launcher sent it. Read it, never
    /// write it.
    #[serde(other, deserialize_with = "ignore_data")]
    Unknown,
}
