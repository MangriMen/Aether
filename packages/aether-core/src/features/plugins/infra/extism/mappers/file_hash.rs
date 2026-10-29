use aether_core_plugin_api::v0::FileHashDto;

use crate::shared::hash::{FileHash, HashAlgorithm, InvalidFileHash};

impl TryFrom<FileHashDto> for FileHash {
    type Error = InvalidFileHash;

    fn try_from(value: FileHashDto) -> Result<Self, Self::Error> {
        match value {
            FileHashDto::Sha1(hex) => Self::new(HashAlgorithm::Sha1, hex),
            FileHashDto::Sha256(hex) => Self::new(HashAlgorithm::Sha256, hex),
            FileHashDto::Sha512(hex) => Self::new(HashAlgorithm::Sha512, hex),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    const SHA1_HEX: &str = "a9993e364706816aba3e25717850c26c9cd0d89d";
    const SHA256_HEX: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    fn sha512_hex() -> String {
        "ab".repeat(64)
    }

    #[rstest]
    #[case::sha1(FileHashDto::Sha1(SHA1_HEX.into()), HashAlgorithm::Sha1, SHA1_HEX.to_owned())]
    #[case::sha256(FileHashDto::Sha256(SHA256_HEX.into()), HashAlgorithm::Sha256, SHA256_HEX.to_owned())]
    #[case::sha512(FileHashDto::Sha512(sha512_hex()), HashAlgorithm::Sha512, sha512_hex())]
    fn should_map_to_the_matching_algorithm_when_the_digest_is_valid(
        #[case] dto: FileHashDto,
        #[case] algorithm: HashAlgorithm,
        #[case] hex: String,
    ) {
        let hash = FileHash::try_from(dto).unwrap();

        assert_eq!(hash.algorithm(), algorithm);
        assert_eq!(hash.hex(), hex);
    }

    #[test]
    fn should_lowercase_the_digest_when_it_is_uppercase() {
        let hash = FileHash::try_from(FileHashDto::Sha1(SHA1_HEX.to_ascii_uppercase())).unwrap();

        assert_eq!(hash.hex(), SHA1_HEX);
    }

    #[test]
    fn should_reject_the_digest_when_it_has_the_wrong_length() {
        // A sha1-length digest labelled sha256.
        let err = FileHash::try_from(FileHashDto::Sha256(SHA1_HEX.into())).unwrap_err();

        assert_eq!(
            err,
            InvalidFileHash::WrongLength {
                algorithm: HashAlgorithm::Sha256,
                expected: 64,
                actual: 40,
            }
        );
    }

    #[test]
    fn should_reject_the_digest_when_it_is_not_hex() {
        let err = FileHash::try_from(FileHashDto::Sha1("z".repeat(40))).unwrap_err();

        assert_eq!(
            err,
            InvalidFileHash::NotHex {
                algorithm: HashAlgorithm::Sha1
            }
        );
    }
}
