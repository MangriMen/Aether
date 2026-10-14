use std::{io::Read, path::Path};

use sha2::{Digest, Sha256, Sha512};
use tokio::task::{JoinError, spawn_blocking};

use crate::shared::hash::HashAlgorithm;

/// The hashing runs on the blocking pool, so the error is that task failing to finish: it
/// panicked, or the runtime shut down under it.
fn task_failed(err: JoinError) -> std::io::Error {
    std::io::Error::other(err)
}

/// Lowercase hex sha1 of `input`.
pub(crate) async fn sha1_async<T>(input: T) -> std::io::Result<String>
where
    T: AsRef<[u8]> + Send + 'static,
{
    spawn_blocking(move || sha1_smol::Sha1::from(input.as_ref()).hexdigest())
        .await
        .map_err(task_failed)
}

/// Lowercase hex sha256 of `input`.
///
/// Extism compares `Wasm::with_hash` to its lowercase digest as a plain string, so the casing
/// matters.
pub(crate) async fn sha256_async<T>(input: T) -> std::io::Result<String>
where
    T: AsRef<[u8]> + Send + 'static,
{
    hash_hex_async(HashAlgorithm::Sha256, input).await
}

/// Lowercase hex digest of `input` with `algorithm`.
pub(crate) async fn hash_hex_async<T>(algorithm: HashAlgorithm, input: T) -> std::io::Result<String>
where
    T: AsRef<[u8]> + Send + 'static,
{
    spawn_blocking(move || {
        let mut hasher = FileHasher::new(algorithm);
        hasher.update(input.as_ref());
        hasher.finalize_hex()
    })
    .await
    .map_err(task_failed)
}

/// Lowercase hex digest of the file at `path`, read in chunks rather than all at once.
pub(crate) async fn hash_file_hex_async(
    algorithm: HashAlgorithm,
    path: impl AsRef<Path>,
) -> std::io::Result<String> {
    let path = path.as_ref().to_path_buf();

    spawn_blocking(move || {
        let mut file = std::fs::File::open(path)?;
        let mut hasher = FileHasher::new(algorithm);
        let mut buffer = vec![0_u8; 64 * 1024];

        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }

        Ok(hasher.finalize_hex())
    })
    .await
    .map_err(task_failed)?
}

/// An incremental digest, so a download can be hashed while it streams to disk instead of
/// being held in memory first.
pub(crate) enum FileHasher {
    Sha1(sha1_smol::Sha1),
    Sha256(Sha256),
    Sha512(Sha512),
}

impl FileHasher {
    pub(crate) fn new(algorithm: HashAlgorithm) -> Self {
        match algorithm {
            HashAlgorithm::Sha1 => Self::Sha1(sha1_smol::Sha1::new()),
            HashAlgorithm::Sha256 => Self::Sha256(Sha256::new()),
            HashAlgorithm::Sha512 => Self::Sha512(Sha512::new()),
        }
    }

    pub(crate) fn update(&mut self, data: &[u8]) {
        match self {
            Self::Sha1(hasher) => hasher.update(data),
            Self::Sha256(hasher) => hasher.update(data),
            Self::Sha512(hasher) => hasher.update(data),
        }
    }

    /// Lowercase hex digest of everything passed to [`Self::update`].
    pub(crate) fn finalize_hex(self) -> String {
        match self {
            Self::Sha1(hasher) => hasher.hexdigest(),
            Self::Sha256(hasher) => to_lower_hex(&hasher.finalize()),
            Self::Sha512(hasher) => to_lower_hex(&hasher.finalize()),
        }
    }
}

fn to_lower_hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut acc, byte| {
        use std::fmt::Write as _;
        let _ = write!(acc, "{byte:02x}");
        acc
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_produce_lowercase_hex_when_hashing_with_sha256() {
        assert_eq!(
            sha256_async(b"abc".to_vec()).await.unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[tokio::test]
    async fn should_hash_empty_input_when_given_no_bytes() {
        assert_eq!(
            sha256_async(Vec::new()).await.unwrap(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[tokio::test]
    async fn should_produce_known_digests_when_hashing_with_every_algorithm() {
        assert_eq!(
            hash_hex_async(HashAlgorithm::Sha1, b"abc".to_vec())
                .await
                .unwrap(),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
        assert_eq!(
            hash_hex_async(HashAlgorithm::Sha512, b"abc".to_vec())
                .await
                .unwrap(),
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
             2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
        );
    }

    #[test]
    fn should_give_the_same_digest_when_fed_in_chunks() {
        let mut chunked = FileHasher::new(HashAlgorithm::Sha512);
        chunked.update(b"a");
        chunked.update(b"bc");

        let mut whole = FileHasher::new(HashAlgorithm::Sha512);
        whole.update(b"abc");

        assert_eq!(chunked.finalize_hex(), whole.finalize_hex());
    }

    #[tokio::test]
    async fn should_hash_a_file_when_it_is_read_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("abc.txt");
        std::fs::write(&path, b"abc").unwrap();

        assert_eq!(
            hash_file_hex_async(HashAlgorithm::Sha1, &path)
                .await
                .unwrap(),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
    }
}
