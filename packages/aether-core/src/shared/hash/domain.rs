use std::{fmt, str::FromStr};

/// Digest algorithms the core can verify a download against.
///
/// The set: sha1 (Modrinth, `CurseForge`), sha256 (packwiz's default `hash-format`) and
/// sha512 (Modrinth). Md5 is deliberately absent until work on `CurseForge` resumes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashAlgorithm {
    Sha1,
    Sha256,
    Sha512,
}

impl HashAlgorithm {
    pub const ALL: [Self; 3] = [Self::Sha1, Self::Sha256, Self::Sha512];

    /// Lowercase name, as used in file names and messages.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sha1 => "sha1",
            Self::Sha256 => "sha256",
            Self::Sha512 => "sha512",
        }
    }

    /// Length of the hex-encoded digest.
    pub const fn hex_len(self) -> usize {
        match self {
            Self::Sha1 => 40,
            Self::Sha256 => 64,
            Self::Sha512 => 128,
        }
    }
}

impl fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for HashAlgorithm {
    type Err = InvalidFileHash;

    /// Parses the names [`HashAlgorithm::name`] produces — the spelling packwiz's `hash-format`
    /// and the keys of Modrinth's `hashes` use. Case is not significant.
    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|algorithm| algorithm.name().eq_ignore_ascii_case(name))
            .ok_or_else(|| InvalidFileHash::UnsupportedAlgorithm {
                name: name.to_owned(),
            })
    }
}

/// The expected digest of a file, hex-encoded.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileHash {
    Sha1(String),
    Sha256(String),
    Sha512(String),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidFileHash {
    #[error("Expected a {expected}-character hex {algorithm} digest, got {actual} characters")]
    WrongLength {
        algorithm: HashAlgorithm,
        expected: usize,
        actual: usize,
    },

    #[error("A {algorithm} digest must contain only hex digits")]
    NotHex { algorithm: HashAlgorithm },

    #[error("Unsupported hash algorithm `{name}`, expected one of sha1, sha256, sha512")]
    UnsupportedAlgorithm { name: String },

    #[error("Expected `<algorithm>:<hex digest>`, got `{value}`")]
    MissingAlgorithm { value: String },
}

impl FileHash {
    pub fn new(algorithm: HashAlgorithm, hex: impl Into<String>) -> Self {
        let hex = hex.into();
        match algorithm {
            HashAlgorithm::Sha1 => Self::Sha1(hex),
            HashAlgorithm::Sha256 => Self::Sha256(hex),
            HashAlgorithm::Sha512 => Self::Sha512(hex),
        }
    }

    /// A validated, lowercased hash from an algorithm name and a hex digest, the two strings a
    /// manifest keeps them as (packwiz's `hash-format` + `hash`, Modrinth's `hashes` map).
    pub fn parse(algorithm: &str, hex: &str) -> Result<Self, InvalidFileHash> {
        Self::new(algorithm.parse()?, hex).normalized()
    }

    pub fn algorithm(&self) -> HashAlgorithm {
        match self {
            Self::Sha1(_) => HashAlgorithm::Sha1,
            Self::Sha256(_) => HashAlgorithm::Sha256,
            Self::Sha512(_) => HashAlgorithm::Sha512,
        }
    }

    pub fn hex(&self) -> &str {
        match self {
            Self::Sha1(hex) | Self::Sha256(hex) | Self::Sha512(hex) => hex,
        }
    }

    /// Whether `actual_hex` is this digest. Hex case is not significant.
    pub fn matches(&self, actual_hex: &str) -> bool {
        self.hex().eq_ignore_ascii_case(actual_hex)
    }

    /// The same hash with its digest lowercased, after checking it really is a digest of its
    /// algorithm: exactly the right length, hex digits only.
    ///
    /// Anything that goes on to name a file must pass through here first — a validated digest
    /// cannot contain a separator, a `..` or a drive letter.
    pub fn normalized(&self) -> Result<Self, InvalidFileHash> {
        let algorithm = self.algorithm();
        let hex = self.hex();

        if hex.len() != algorithm.hex_len() {
            return Err(InvalidFileHash::WrongLength {
                algorithm,
                expected: algorithm.hex_len(),
                actual: hex.len(),
            });
        }

        if !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(InvalidFileHash::NotHex { algorithm });
        }

        Ok(Self::new(algorithm, hex.to_ascii_lowercase()))
    }
}

impl fmt::Display for FileHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.algorithm(), self.hex())
    }
}

impl FromStr for FileHash {
    type Err = InvalidFileHash;

    /// The inverse of `Display`: `<algorithm>:<hex digest>`, validated and lowercased like
    /// [`FileHash::parse`].
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (algorithm, hex) =
            value
                .split_once(':')
                .ok_or_else(|| InvalidFileHash::MissingAlgorithm {
                    value: value.to_owned(),
                })?;

        Self::parse(algorithm, hex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    const SHA1_ABC: &str = "a9993e364706816aba3e25717850c26c9cd0d89d";

    #[test]
    fn should_lowercase_a_digest_when_it_is_valid() {
        let hash = FileHash::Sha1(SHA1_ABC.to_ascii_uppercase());
        assert_eq!(hash.normalized().unwrap(), FileHash::Sha1(SHA1_ABC.into()));
    }

    #[test]
    fn should_reject_a_digest_when_it_has_the_wrong_length() {
        // A sha1-length digest labelled sha256 must not be accepted as a short sha256.
        let err = FileHash::Sha256(SHA1_ABC.into()).normalized().unwrap_err();
        assert_eq!(
            err,
            InvalidFileHash::WrongLength {
                algorithm: HashAlgorithm::Sha256,
                expected: 64,
                actual: 40,
            }
        );
    }

    #[rstest]
    #[case::unix_traversal("../../../../../../../../../../../../etc/passwd")]
    #[case::windows_traversal(r"..\..\..\..\..\..\..\..\..\..\windows")]
    #[case::drive_prefix("C:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    fn should_reject_a_digest_when_it_is_path_like(#[case] hex: &str) {
        let padded = format!("{hex:a<40}");
        let padded: String = padded.chars().take(40).collect();
        assert!(
            FileHash::Sha1(padded.clone()).normalized().is_err(),
            "`{padded}` must not pass as a digest"
        );
    }

    #[test]
    fn should_match_when_hex_case_differs() {
        let hash = FileHash::Sha1(SHA1_ABC.into());
        assert!(hash.matches(&SHA1_ABC.to_ascii_uppercase()));
        assert!(!hash.matches("0000000000000000000000000000000000000000"));
    }

    #[test]
    fn should_display_with_the_algorithm_prefix_when_formatted() {
        assert_eq!(
            FileHash::Sha1(SHA1_ABC.into()).to_string(),
            format!("sha1:{SHA1_ABC}")
        );
    }

    const SHA256_ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    #[test]
    fn should_parse_every_algorithm_name_when_the_case_differs() {
        for algorithm in HashAlgorithm::ALL {
            assert_eq!(algorithm.name().parse(), Ok(algorithm));
            assert_eq!(algorithm.name().to_ascii_uppercase().parse(), Ok(algorithm));
        }
    }

    // packwiz's `hash-format` may also be md5 or murmur2; neither is verifiable here.
    #[rstest]
    #[case::md5("md5")]
    #[case::murmur2("murmur2")]
    #[case::empty("")]
    #[case::truncated("sha")]
    fn should_refuse_an_algorithm_when_it_is_outside_the_supported_set(#[case] name: &str) {
        assert_eq!(
            name.parse::<HashAlgorithm>(),
            Err(InvalidFileHash::UnsupportedAlgorithm { name: name.into() })
        );
    }

    #[test]
    fn should_parse_a_hash_when_given_its_format_and_digest() {
        assert_eq!(
            FileHash::parse("sha256", &SHA256_ABC.to_ascii_uppercase()),
            Ok(FileHash::Sha256(SHA256_ABC.into()))
        );
    }

    #[test]
    fn should_validate_the_digest_when_parsing_from_a_format() {
        assert_eq!(
            FileHash::parse("sha256", SHA1_ABC),
            Err(InvalidFileHash::WrongLength {
                algorithm: HashAlgorithm::Sha256,
                expected: 64,
                actual: 40,
            })
        );
        assert_eq!(
            FileHash::parse("md5", "900150983cd24fb0d6963f7d28e17f72"),
            Err(InvalidFileHash::UnsupportedAlgorithm { name: "md5".into() })
        );
    }

    #[rstest]
    #[case::sha1(FileHash::Sha1(SHA1_ABC.into()))]
    #[case::sha256(FileHash::Sha256(SHA256_ABC.into()))]
    #[case::sha512(FileHash::Sha512("a".repeat(128)))]
    fn should_round_trip_when_converted_through_its_string_form(#[case] hash: FileHash) {
        assert_eq!(hash.to_string().parse::<FileHash>(), Ok(hash));
    }

    #[test]
    fn should_lowercase_when_parsing_the_string_form() {
        let upper = format!("SHA1:{}", SHA1_ABC.to_ascii_uppercase());
        assert_eq!(upper.parse(), Ok(FileHash::Sha1(SHA1_ABC.into())));
    }

    #[test]
    fn should_refuse_a_string_form_when_it_has_no_algorithm() {
        assert_eq!(
            SHA1_ABC.parse::<FileHash>(),
            Err(InvalidFileHash::MissingAlgorithm {
                value: SHA1_ABC.into()
            })
        );
    }
}
