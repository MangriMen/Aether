use crate::features::plugins::{
    PluginError, PluginSourceType, PluginStorage, PluginVerification, PluginVerificationStorage,
};

/// Establishes trust in a plugin's wasm blob (TOFU, `OPEN_QUESTIONS.md` Q4).
///
/// Call this right after a plugin has been downloaded from a remote provider
/// and written to its directory — that is the one moment the bytes on disk are
/// known to be the ones the release served. From then on every load re-hashes
/// the file and refuses to run it if the digest changed.
///
/// Must not be called for locally imported plugins: nothing vouches for those
/// bytes, so recording their hash would only manufacture false confidence.
pub async fn record_plugin_verification(
    plugin_storage: &dyn PluginStorage,
    verification_storage: &dyn PluginVerificationStorage,
    plugin_id: &str,
    source_type: PluginSourceType,
) -> Result<(), PluginError> {
    let wasm_sha256 = plugin_storage.wasm_sha256(plugin_id).await?;

    tracing::info!(
        "Recorded wasm sha256 {} for plugin '{}' downloaded from {}",
        wasm_sha256,
        plugin_id,
        source_type
    );

    verification_storage
        .save(
            plugin_id,
            &PluginVerification::new(wasm_sha256, source_type),
        )
        .await
}
