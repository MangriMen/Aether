use crate::features::instance::PluginId;
use crate::features::plugins::{
    PluginError, PluginSourceType, PluginStorage, PluginVerification, PluginVerificationStorage,
};

/// Establishes trust on first use in a plugin's wasm blob; every later load refuses a file
/// whose digest changed.
///
/// Call right after a download from a remote provider is written, the one moment the bytes are
/// known to be the release's. Never for local imports: nothing vouches for those bytes.
pub async fn record_plugin_verification(
    plugin_storage: &dyn PluginStorage,
    verification_storage: &dyn PluginVerificationStorage,
    plugin_id: &PluginId,
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
