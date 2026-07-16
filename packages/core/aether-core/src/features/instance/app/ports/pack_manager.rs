use async_trait::async_trait;

use crate::features::instance::domain::{
    DownloadContext, InstanceError, PackInstallParams, PackManagerCapabilityMetadata, PackMetadata,
    UpdateStatus,
};

/// Unified contract for pack lifecycle management.
///
/// Replaces the old split between `Importer` (import), `ContentProvider::install_modpack`
/// (install from registry), and `Updater` (update).
///
/// Implementations are registered per `ProviderId` and dispatched via
/// `PackManagerUseCase`.
#[async_trait]
pub trait PackManager: Send + Sync {
    /// Metadata about this manager for UI display and capability gating.
    fn metadata(&self) -> &PackManagerCapabilityMetadata;

    /// Resolve pack metadata from the given source _before_ instance creation.
    ///
    /// The core calls this first to learn the game version, mod loader, and
    /// instance name — so it can create a properly configured instance without
    /// requiring the frontend to guess these values.
    /// Returns `Err` if the manager cannot resolve metadata independently.
    async fn resolve_pack_metadata(
        &self,
        _params: &PackInstallParams,
    ) -> Result<PackMetadata, InstanceError> {
        Err(InstanceError::UnsupportedOperation(
            "resolve_pack_metadata is not implemented by this provider".into(),
        ))
    }

    /// Install a pack into an already-created instance (`IoC` — instance
    /// creation is handled by the core, not by the manager).
    ///
    /// The instance directory already exists and Minecraft+loader are
    /// installed. The manager should download/extract all pack files
    /// into the instance directory.
    async fn install(
        &self,
        instance_id: &str,
        params: &PackInstallParams,
        ctx: &DownloadContext,
    ) -> Result<(), InstanceError>;

    /// Update an existing pack-managed instance to the latest version.
    /// Optional — default returns an error.
    async fn update(
        &self,
        _instance_id: &str,
        _ctx: &DownloadContext,
    ) -> Result<(), InstanceError> {
        Err(InstanceError::UnsupportedOperation(
            "Update is not supported by this provider".into(),
        ))
    }

    /// Check whether a newer version of the pack is available.
    /// Optional — default returns `Unsupported`.
    async fn check_updates(&self, _instance_id: &str) -> Result<UpdateStatus, InstanceError> {
        Ok(UpdateStatus::Unsupported)
    }
}
