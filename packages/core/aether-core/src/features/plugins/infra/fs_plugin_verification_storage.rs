use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        plugins::{PluginError, PluginVerification, PluginVerificationStorage},
        settings::LocationInfo,
    },
    shared::io::{
        domain::IoError,
        infra::{read_json_async, write_json_async},
    },
};

/// Stores the TOFU record as `verification.json` inside the plugin directory,
/// next to `source.json`.
///
/// Living in the plugin directory means the record dies with the plugin: both
/// `PluginStorage::remove` and the remove-then-add of an update wipe it, so a
/// reinstall always re-establishes trust from freshly downloaded bytes instead
/// of matching against a stale digest.
pub struct FsPluginVerificationStorage {
    location_info: Arc<LocationInfo>,
}

impl FsPluginVerificationStorage {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    fn get_verification_path(&self, plugin_id: &str) -> PathBuf {
        self.location_info
            .plugin_dir(plugin_id)
            .join("verification.json")
    }
}

#[async_trait]
impl PluginVerificationStorage for FsPluginVerificationStorage {
    async fn save(
        &self,
        plugin_id: &str,
        verification: &PluginVerification,
    ) -> Result<(), PluginError> {
        write_json_async(self.get_verification_path(plugin_id), verification)
            .await
            .map_err(PluginError::Storage)
    }

    async fn get(&self, plugin_id: &str) -> Result<Option<PluginVerification>, PluginError> {
        match read_json_async::<PluginVerification>(self.get_verification_path(plugin_id)).await {
            Ok(verification) => Ok(Some(verification)),
            // No record: the plugin was never installed through a remote
            // provider. That is "unverified", not a failure.
            Err(e) if is_not_found(&e) => Ok(None),
            Err(e) => Err(PluginError::Storage(e)),
        }
    }
}

fn is_not_found(error: &IoError) -> bool {
    match error {
        IoError::IoPathError { source, .. } | IoError::IoError(source) => {
            source.kind() == std::io::ErrorKind::NotFound
        }
        _ => false,
    }
}
