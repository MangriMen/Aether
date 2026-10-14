use serde::{Deserialize, Serialize};

/// An expected file digest, hex-encoded. Hex case does not matter.
///
/// Md5 is deliberately not offered.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum FileHashDto {
    /// 40 hex characters.
    Sha1(String),
    /// 64 hex characters.
    Sha256(String),
    /// 128 hex characters.
    Sha512(String),
}
