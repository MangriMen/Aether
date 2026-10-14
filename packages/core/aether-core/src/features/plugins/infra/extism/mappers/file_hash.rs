use aether_core_plugin_api::v0::FileHashDto;

use crate::shared::hash::FileHash;

impl From<FileHashDto> for FileHash {
    fn from(value: FileHashDto) -> Self {
        match value {
            FileHashDto::Sha1(hex) => Self::Sha1(hex),
            FileHashDto::Sha256(hex) => Self::Sha256(hex),
            FileHashDto::Sha512(hex) => Self::Sha512(hex),
        }
    }
}
