use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::app::ports::CreateInstanceUseCasePort;
use crate::features::instance::app::ports::InstallPackUseCasePort;
use crate::features::instance::domain::InstanceInstallStage;
use crate::{
    features::instance::{
        DownloadContext, InstanceError, InstanceStorage, InstanceStorageExt, NewInstance,
        PackInstallParams, PackManager, ProviderId,
    },
    shared::capability::domain::CapabilityRegistry,
};

use crate::features::minecraft::LoaderVersionPreference;

/// Input for the `InstallPackUseCase`.
#[derive(Debug, Clone)]
pub struct InstallPackRequest {
    /// Optional instance params (name, game version, loader).
    ///
    /// If `None`, the use case will call `PackManager::resolve_pack_metadata`
    /// to extract these from the pack source. This is the preferred path for
    /// pack managers that can parse packs (e.g. packwiz).
    ///
    /// If `Some`, the provided values are used directly — this allows
    /// frontends to override or provide metadata for managers that don't
    /// support resolution yet.
    pub new_instance: Option<NewInstance>,
    /// The pack parameters to pass to the `PackManager`.
    pub pack_params: PackInstallParams,
    /// Which `PackManager` to use (resolved from the registry).
    pub provider_id: ProviderId,
}

/// Orchestrates the full pack installation flow:
///   1. Resolves pack metadata (name, game version, loader) from the manager
///   2. Creates the instance (Minecraft + loader install → `PackInstalling`)
///   3. Delegates pack content installation to the `PackManager`
///   4. Marks the instance as `Installed` on success
///
/// This replaces the old split between `ContentProvider::install_modpack`
/// and `Importer::import`.
pub struct InstallPackUseCase {
    create_instance_uc: Arc<dyn CreateInstanceUseCasePort>,
    pack_manager_registry: Arc<dyn CapabilityRegistry<Arc<dyn PackManager>>>,
    instance_storage: Arc<dyn InstanceStorage>,
}

impl InstallPackUseCase {
    pub fn new(
        create_instance_uc: Arc<dyn CreateInstanceUseCasePort>,
        pack_manager_registry: Arc<dyn CapabilityRegistry<Arc<dyn PackManager>>>,
        instance_storage: Arc<dyn InstanceStorage>,
    ) -> Self {
        Self {
            create_instance_uc,
            pack_manager_registry,
            instance_storage,
        }
    }

    pub async fn execute(&self, request: InstallPackRequest) -> Result<(), InstanceError> {
        let InstallPackRequest {
            new_instance,
            pack_params,
            provider_id,
        } = request;

        // 1. Look up the pack manager
        let manager = self
            .pack_manager_registry
            .find_by_plugin_and_capability_id(&provider_id.plugin_id, &provider_id.capability_id)
            .await
            .map_err(|_| InstanceError::PackManagerNotFound {
                plugin_id: provider_id.plugin_id.clone(),
                capability_id: provider_id.capability_id.clone(),
            })?;

        // 2. Resolve pack metadata — try the manager first, fall back to
        //    the `new_instance` provided by the caller (e.g. frontend).
        let new_instance = match manager.capability.resolve_pack_metadata(&pack_params).await {
            Ok(metadata) => NewInstance {
                name: metadata.name,
                game_version: metadata.game_version,
                mod_loader: metadata.mod_loader,
                loader_version: metadata.loader_version.map(LoaderVersionPreference::Exact),
                icon_path: None,
                skip_install_instance: None,
                pack_info: metadata.pack_info,
            },
            Err(_) => {
                // Fallback: caller-provided values (or error if neither available)
                new_instance.ok_or_else(|| {
                    InstanceError::UnsupportedOperation(
                        "Pack manager cannot resolve metadata and no instance params provided"
                            .into(),
                    )
                })?
            }
        };

        // 3. Create instance (Minecraft + loader install → PackInstalling stage)
        let instance_id = self.create_instance_uc.execute(new_instance).await?;

        // 4. Install pack content via the manager
        let result = manager
            .capability
            .install(&instance_id, &pack_params, &DownloadContext {})
            .await;

        // 5. On success → Installed, on failure → NotInstalled
        let final_stage = match &result {
            Ok(()) => InstanceInstallStage::Installed,
            Err(_) => InstanceInstallStage::NotInstalled,
        };

        self.instance_storage
            .upsert_with(&instance_id, |instance| {
                instance.install_stage = final_stage;
                Ok(())
            })
            .await?;

        result
    }
}

#[async_trait]
impl InstallPackUseCasePort for InstallPackUseCase {
    async fn execute(&self, request: InstallPackRequest) -> Result<(), InstanceError> {
        self.execute(request).await
    }
}
