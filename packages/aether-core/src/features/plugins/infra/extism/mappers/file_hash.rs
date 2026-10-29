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
