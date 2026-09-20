use sha2::{Digest, Sha256};
use tokio::task::spawn_blocking;

pub async fn sha1_async<T>(input: T) -> String
where
    T: AsRef<[u8]> + Send + 'static,
{
    spawn_blocking(move || sha1_smol::Sha1::from(input.as_ref()).hexdigest())
        .await
        .expect("sha1_async: task panicked")
}

/// Lowercase hex sha256 of `input`.
///
/// The hex casing matters: Extism compares `Wasm::with_hash` against its own
/// lowercase hex digest with a plain string comparison
/// (`extism-1.30.0/src/manifest.rs`, `check_hash`).
pub async fn sha256_async<T>(input: T) -> String
where
    T: AsRef<[u8]> + Send + 'static,
{
    spawn_blocking(move || {
        let digest = Sha256::digest(input.as_ref());
        digest.iter().fold(String::new(), |mut acc, byte| {
            use std::fmt::Write as _;
            let _ = write!(acc, "{byte:02x}");
            acc
        })
    })
    .await
    .expect("sha256_async: task panicked")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_produce_lowercase_hex_sha256() {
        assert_eq!(
            sha256_async(b"abc".to_vec()).await,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[tokio::test]
    async fn should_hash_empty_input() {
        assert_eq!(
            sha256_async(Vec::new()).await,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
