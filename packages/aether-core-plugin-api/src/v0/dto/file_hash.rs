use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The algorithm of a [`FileHashDto`].
///
/// Md5 is deliberately not offered.
#[derive(Serialize, JsonSchema, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum HashAlgorithmDto {
    /// 40 hex characters.
    Sha1,
    /// 64 hex characters.
    Sha256,
    /// 128 hex characters.
    Sha512,
}

/// An expected file digest, hex-encoded. Hex case does not matter.
#[derive(Serialize, JsonSchema, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileHashDto {
    pub algorithm: HashAlgorithmDto,
    pub hex: String,
}

impl FileHashDto {
    pub fn sha1(hex: impl Into<String>) -> Self {
        Self::new(HashAlgorithmDto::Sha1, hex)
    }

    pub fn sha256(hex: impl Into<String>) -> Self {
        Self::new(HashAlgorithmDto::Sha256, hex)
    }

    pub fn sha512(hex: impl Into<String>) -> Self {
        Self::new(HashAlgorithmDto::Sha512, hex)
    }

    fn new(algorithm: HashAlgorithmDto, hex: impl Into<String>) -> Self {
        Self {
            algorithm,
            hex: hex.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_write_a_hash_as_its_algorithm_and_hex() {
        assert_eq!(
            serde_json::to_string(&FileHashDto::sha1("ab")).unwrap(),
            r#"{"algorithm":"sha1","hex":"ab"}"#
        );
    }
}
