use async_trait::async_trait;

use crate::features::instance::domain::InstanceError;

/// Handles the lifecycle of a specific modpack format on the **core side**.
///
/// Each implementation knows how to parse and deploy a particular pack format
/// (e.g. packwiz `pack.toml`, Modrinth `.mrpack`, `CurseForge` `.zip`).
///
/// The `ContentSource` plugin returns raw manifest bytes via `VersionPayload::Modpack`,
/// and the core dispatches them to the matching `PackLifecycleHandler` by `format_id`.
#[async_trait]
pub trait PackLifecycleHandler: Send + Sync {
    /// Unique identifier for the pack format this handler supports
    /// (e.g. "packwiz", "mrpack", "curseforge").
    fn format_id(&self) -> &str;

    /// Deploy a pack from its raw manifest bytes into an existing instance directory.
    ///
    /// The instance already has Minecraft + loader installed. The handler should
    /// download all pack files and place them in the correct locations.
    async fn deploy_pack(
        &self,
        manifest_bytes: &[u8],
        instance_id: &str,
    ) -> Result<(), InstanceError>;

    /// Update an existing pack-managed instance with a new manifest.
    /// Optional — default returns an error.
    async fn update_pack(
        &self,
        _new_manifest_bytes: &[u8],
        _instance_id: &str,
    ) -> Result<(), InstanceError> {
        Err(InstanceError::UnsupportedOperation(
            "Update is not supported by this handler".into(),
        ))
    }
}
