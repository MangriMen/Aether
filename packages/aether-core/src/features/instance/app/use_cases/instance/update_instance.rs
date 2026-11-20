use std::sync::Arc;

use async_trait::async_trait;

use super::super::content_provider::PlanProgress;
use crate::features::instance::app::ports::{InstallPlanExecutorPort, UpdateInstanceUseCasePort};
use crate::{
    features::{
        instance::{
            ContentProvider, ContentProviderRejection, ContentVersionId, Instance, InstanceError,
            InstanceId, InstanceInstallStage, InstanceLocks, InstanceStorage, InstanceStorageExt,
            PackInfo, PackSource, PlanOrigin, ResolveInstallPlanParams, ResolveTarget, Updater,
        },
        minecraft::ModLoader,
    },
    shared::capability::CapabilityRegistry,
};

/// Updates the pack an instance was installed from.
///
/// A pack whose provider resolves install plans is updated by running the plan of its next
/// version through the executor, like any other content; a pack of a plugin that deploys it
/// itself (packwiz) goes through that plugin's [`Updater`].
///
/// The whole update runs under the instance's lock ([`InstanceLocks`]): another plan waits for
/// it, and the instance is not launched halfway through.
pub struct UpdateInstanceUseCase {
    instance_storage: Arc<dyn InstanceStorage>,
    updaters_registry: Arc<dyn CapabilityRegistry<Arc<dyn Updater>>>,
    provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
    plan_executor: Arc<dyn InstallPlanExecutorPort>,
    instance_locks: Arc<InstanceLocks>,
}

/// How an update went through.
enum Updated {
    /// By a plan; the version it installed, when the provider named it.
    Plan {
        version_id: Option<ContentVersionId>,
    },
    /// By the plugin's updater, which keeps the pack info itself.
    Updater,
}

impl UpdateInstanceUseCase {
    pub fn new(
        instance_storage: Arc<dyn InstanceStorage>,
        updaters_registry: Arc<dyn CapabilityRegistry<Arc<dyn Updater>>>,
        provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
        plan_executor: Arc<dyn InstallPlanExecutorPort>,
        instance_locks: Arc<InstanceLocks>,
    ) -> Self {
        Self {
            instance_storage,
            updaters_registry,
            provider_registry,
            plan_executor,
            instance_locks,
        }
    }

    pub async fn update_by_plugin(
        &self,
        instance_id: &InstanceId,
        pack_info: &PackInfo,
    ) -> Result<(), InstanceError> {
        let updater = self
            .updaters_registry
            .find_by_plugin_and_capability_id(
                &pack_info.provider_id.plugin_id,
                &pack_info.provider_id.capability_id,
            )
            .await
            .map_err(|_| InstanceError::UpdaterNotFound {
                modpack_id: pack_info.modpack_id.to_string(),
            })?;

        updater.capability.update(instance_id).await
    }

    async fn update(
        &self,
        instance: &Instance,
        pack_info: &PackInfo,
    ) -> Result<Updated, InstanceError> {
        let provider = self
            .provider_registry
            .find_by_plugin_and_capability_id(
                &pack_info.provider_id.plugin_id,
                &pack_info.provider_id.capability_id,
            )
            .await;
        let Ok(provider) = provider else {
            self.update_by_plugin(instance.id(), pack_info).await?;
            return Ok(Updated::Updater);
        };
        let provider = provider.capability;

        // Asked first, so the plan is of the very version the pack info will name afterwards.
        let version_id = self
            .version_to_install(provider.as_ref(), pack_info)
            .await?;

        let requested = ResolveTarget::Existing {
            instance_id: instance.id().clone(),
            game_version: instance.game_version.clone(),
            loader: (instance.loader != ModLoader::Vanilla).then_some(instance.loader),
        };
        // Started before the plan is resolved: the pack's archive is downloaded then.
        let progress = self
            .plan_executor
            .begin(&requested, &pack_info.modpack_id)
            .await;
        let result = self
            .update_by_plan(
                instance,
                pack_info,
                provider.as_ref(),
                version_id,
                requested,
                &progress,
            )
            .await;
        progress.end(&result).await;
        result
    }

    async fn update_by_plan(
        &self,
        instance: &Instance,
        pack_info: &PackInfo,
        provider: &dyn ContentProvider,
        version_id: Option<ContentVersionId>,
        requested: ResolveTarget,
        progress: &PlanProgress,
    ) -> Result<Updated, InstanceError> {
        let plan = match progress
            .resolve_plan(
                provider,
                ResolveInstallPlanParams {
                    content_id: pack_info.modpack_id.clone(),
                    version: version_id.clone(),
                    installed_version: installed_version_of(pack_info),
                    target: requested.clone(),
                },
            )
            .await
        {
            Ok(plan) => plan,
            // A plugin that deploys its packs itself has no plans to give.
            Err(InstanceError::UnsupportedOperation {
                operation: "resolve_install_plan",
                ..
            }) => {
                progress.hand_over_to_plugin().await;
                self.update_by_plugin(instance.id(), pack_info).await?;
                return Ok(Updated::Updater);
            }
            Err(err) => return Err(err),
        };

        // A built-in provider is not behind the plugin proxy that checks this.
        if !requested.is_answered_by(&plan.target) {
            return Err(InstanceError::ContentProviderError {
                reason: ContentProviderRejection::WrongTarget {
                    provider_id: provider.provider_id(),
                },
            });
        }

        let origin = PlanOrigin {
            provider_id: pack_info.provider_id.clone(),
            content_id: pack_info.modpack_id.clone(),
        };
        self.plan_executor
            .execute_in(plan, origin, progress)
            .await?;
        Ok(Updated::Plan { version_id })
    }

    /// The version an update takes, as a check for updates finds it: by the release channel of
    /// the installed one, never older. `None` — the provider cannot tell without the plan, which
    /// then picks it the same way.
    async fn version_to_install(
        &self,
        provider: &dyn ContentProvider,
        pack_info: &PackInfo,
    ) -> Result<Option<ContentVersionId>, InstanceError> {
        match provider
            .resolve_pack_metadata(PackSource::Content {
                content_id: pack_info.modpack_id.clone(),
                version: None,
                installed_version: installed_version_of(pack_info),
            })
            .await
        {
            Ok(metadata) => Ok(metadata.version_id),
            Err(InstanceError::UnsupportedOperation {
                operation: "resolve_pack_metadata",
                ..
            }) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

#[async_trait]
impl UpdateInstanceUseCasePort for UpdateInstanceUseCase {
    async fn execute(&self, instance_id: InstanceId) -> Result<(), InstanceError> {
        // Taken before the instance is read: a plan that ran meanwhile changes its stage.
        let _lock = self.instance_locks.lock(&instance_id).await;
        let instance = self.instance_storage.get(&instance_id).await?;

        let original_stage = instance.install_stage;
        let pack_info = instance
            .pack_info
            .clone()
            .ok_or(InstanceError::PackInfoNotFound)?;

        self.instance_storage
            .upsert_with(&instance_id, |instance| {
                instance.install_stage = InstanceInstallStage::PackInstalling;
                Ok(())
            })
            .await?;

        // Installing a new game on the way keeps the instance `PackInstalling`: the stage is
        // settled here, once the whole update is in or has failed.
        let result = self.update(&instance, &pack_info).await;

        let (final_stage, new_version) = match &result {
            Ok(Updated::Plan { version_id }) => {
                (InstanceInstallStage::Installed, version_id.clone())
            }
            Ok(Updated::Updater) => (InstanceInstallStage::Installed, None),
            // A failed plan has put the instance back as it was (`InstallPlanExecutor`); a
            // plugin's updater answers for its own.
            Err(_) => (original_stage, None),
        };
        self.instance_storage
            .upsert_with(&instance_id, |instance| {
                instance.install_stage = final_stage;
                if let Some(version_id) = new_version
                    && let Some(current) = instance.pack_info.as_mut()
                    && current.modpack_id == pack_info.modpack_id
                {
                    current.version_id = version_id.into();
                }
                Ok(())
            })
            .await?;

        if let Ok(Updated::Plan { version_id: None }) = &result {
            tracing::warn!(
                "Pack \"{}\" of \"{instance_id}\" updated, but its provider did not name the \
                 version: the instance still records \"{}\"",
                pack_info.modpack_id,
                pack_info.version_id
            );
        }

        result.map(|_| ())
    }
}

/// The pack's version label as a provider's version id, when it is one.
fn installed_version_of(pack_info: &PackInfo) -> Option<ContentVersionId> {
    ContentVersionId::try_from(pack_info.version_id.as_str()).ok()
}
