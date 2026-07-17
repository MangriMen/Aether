use std::sync::Arc;

use async_trait::async_trait;
use path_slash::PathBufExt;

use crate::features::instance::app::dtos::InstallContentRequest;
use crate::features::instance::app::ports::{
    ContentFileService, CreateInstanceUseCasePort, InstallContentUseCasePort,
};
use crate::features::instance::app::services::PackLifecycleHandlerRegistry;
use crate::features::instance::domain::{
    ContentFile, DownloadInstruction, InstanceError, VersionPayload,
};
use crate::features::instance::{
    ContentSource, CreateContentFileParams, InstanceStorage, InstanceStorageExt, NewInstance,
    PackFile, PackStorage,
};
use crate::shared::capability::domain::CapabilityRegistry;
use crate::shared::request_client::{Request, RequestClient};

/// Unified use case for installing any content — single assets or modpacks.
///
/// Flow:
///   1. Resolves `ContentSource` from the registry
///   2. Calls `get_version_info` to determine the payload type
///   3. For `VersionPayload::Asset`: downloads and installs into the target instance
///   4. For `VersionPayload::Modpack`: creates an instance (if needed), then
///      delegates to the matching `PackLifecycleHandler`
pub struct InstallContentUseCase {
    request_client: Arc<dyn RequestClient>,
    pack_storage: Arc<dyn PackStorage>,
    // TODO(step-3): Replace ContentProvider registry with ContentSource registry
    content_source_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentSource>>>,
    content_file_service: Arc<dyn ContentFileService>,
    create_instance_uc: Arc<dyn CreateInstanceUseCasePort>,
    instance_storage: Arc<dyn InstanceStorage>,
    pack_lifecycle_handlers: Arc<PackLifecycleHandlerRegistry>,
}

impl InstallContentUseCase {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        request_client: Arc<dyn RequestClient>,
        pack_storage: Arc<dyn PackStorage>,
        content_source_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentSource>>>,
        content_file_service: Arc<dyn ContentFileService>,
        create_instance_uc: Arc<dyn CreateInstanceUseCasePort>,
        instance_storage: Arc<dyn InstanceStorage>,
        pack_lifecycle_handlers: Arc<PackLifecycleHandlerRegistry>,
    ) -> Self {
        Self {
            request_client,
            pack_storage,
            content_source_registry,
            content_file_service,
            create_instance_uc,
            instance_storage,
            pack_lifecycle_handlers,
        }
    }

    pub async fn execute(&self, request: InstallContentRequest) -> Result<(), InstanceError> {
        let source_id = request.source_id.clone();
        let target_instance_id = request.target_instance_id.clone();

        // 1. Resolve the ContentSource
        let source = self
            .content_source_registry
            .find_by_plugin_and_capability_id(&source_id.plugin_id, &source_id.capability_id)
            .await
            .map_err(|_| InstanceError::ContentProviderNotFound {
                plugin_id: source_id.plugin_id.clone(),
                capability_id: source_id.capability_id.clone(),
            })?;

        // 2. Get version info from the source
        let version_info = source
            .capability
            .get_version_info(&request.content_id, &request.version_id)
            .await?;

        // 3. Validate request against the payload type
        request.validate_for_payload(&version_info.payload)?;

        match version_info.payload {
            // ── Single asset: download and install into an existing instance ──
            VersionPayload::Asset(instruction) => {
                let instance_id =
                    target_instance_id.expect("target_instance_id is required; validated above");
                self.install_asset(&instance_id, &instruction).await
            }

            // ── Modpack: create instance (if needed), delegate to handler ──
            VersionPayload::Modpack(payload) => {
                let instance_id = if let Some(id) = target_instance_id {
                    id
                } else {
                    let new_instance = NewInstance {
                        name: version_info.version_name,
                        game_version: version_info
                            .game_versions
                            .first()
                            .cloned()
                            .unwrap_or_default(),
                        mod_loader: version_info
                            .loaders
                            .first()
                            .copied()
                            .unwrap_or(crate::features::minecraft::ModLoader::Vanilla),
                        loader_version: None,
                        icon_path: None,
                        skip_install_instance: None,
                        pack_info: None,
                    };
                    self.create_instance_uc.execute(new_instance).await?
                };

                // Find the pack lifecycle handler for this format
                let handler = self.pack_lifecycle_handlers.get(&payload.format_id)?;

                // TODO(step-3): wire DownloadContext properly
                handler
                    .deploy_pack(&payload.manifest_bytes, &instance_id)
                    .await?;

                // Mark as installed
                self.instance_storage
                    .upsert_with(&instance_id, |instance| {
                        instance.install_stage =
                            crate::features::instance::InstanceInstallStage::Installed;
                        Ok(())
                    })
                    .await?;

                Ok(())
            }
        }
    }

    /// Download a single asset and install it into the target instance.
    async fn install_asset(
        &self,
        instance_id: &str,
        instruction: &DownloadInstruction,
    ) -> Result<(), InstanceError> {
        use std::io::Write;

        // Remove old file if present
        let old_file = self
            .pack_storage
            .find_by_provider_id(
                instance_id,
                &instruction.provider_id,
                &instruction.content_id,
            )
            .await?;

        if let Some(old_file) = old_file {
            self.pack_storage
                .remove_pack_file(
                    instance_id,
                    &instruction
                        .content_type
                        .get_relative_path(&old_file.file_name)
                        .to_slash_lossy(),
                )
                .await?;
        }

        // Create temp file
        let mut temp_file = tempfile::NamedTempFile::new()
            .map_err(|err| InstanceError::Storage(format!("Failed to create temp file: {err}")))?;

        let file_bytes = self
            .request_client
            .fetch_bytes(Request::get(&instruction.url))
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.to_string()))?;

        // Verify checksum if provided
        if let Some(ref checksum) = instruction.checksum {
            let actual_hash = sha1_smol::Sha1::from(&file_bytes[..]).digest().to_string();
            if actual_hash != checksum.value {
                return Err(InstanceError::ContentDownloadError(format!(
                    "Checksum mismatch: expected {} ({}), got {}",
                    checksum.value, checksum.algorithm, actual_hash
                )));
            }
        }

        // Write to temp file
        temp_file
            .write_all(&file_bytes)
            .map_err(|err| InstanceError::Storage(format!("Failed to write temp file: {err}")))?;

        let temp_path = temp_file.path().to_owned();
        let content_path = instruction.content_path.to_slash_lossy().to_string();

        // Install file into instance
        self.content_file_service
            .install_content_file(instance_id, &content_path, &temp_path)
            .await?;

        // Record in pack storage
        let content_file = ContentFile::from_params(CreateContentFileParams {
            name: instruction.name.clone(),
            file_name: instruction.file_name.clone(),
            size: instruction.size,
            sha1: instruction
                .checksum
                .as_ref()
                .map(|c| c.value.clone())
                .unwrap_or_default(),
            content_path: instruction.content_path.clone(),
            content_id: instruction.content_id.clone(),
            content_version: instruction.content_version.clone(),
            content_type: instruction.content_type,
            provider_id: instruction.provider_id.clone(),
        });

        let pack_file: PackFile = content_file.into();

        self.pack_storage
            .update_pack_file(instance_id, &content_path, &pack_file)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl InstallContentUseCasePort for InstallContentUseCase {
    async fn execute(&self, request: InstallContentRequest) -> Result<(), InstanceError> {
        self.execute(request).await
    }
}
