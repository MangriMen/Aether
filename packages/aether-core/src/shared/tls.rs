/// Installs `ring` as the process-wide rustls crypto provider.
///
/// reqwest is built without a bundled provider and panics when a client is built before one
/// is installed, so call this ahead of every `reqwest::Client::builder()`. It is idempotent:
/// a provider already installed (by an earlier call, or by the Tauri updater, which installs
/// the same one) is kept.
pub fn install_crypto_provider() {
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        // Losing a race to another thread installing it is fine: the outcome is the same.
        let _ = rustls::crypto::ring::default_provider().install_default();
    }
}
