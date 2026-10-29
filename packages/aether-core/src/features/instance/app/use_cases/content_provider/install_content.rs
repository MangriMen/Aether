use async_trait::async_trait;
use std::sync::Arc;

use super::install_plan_executor::is_pack_of;
use crate::features::instance::app::ports::{InstallContentUseCasePort, InstallPlanExecutorPort};
use crate::features::minecraft::ModLoader;
use crate::{
    features::instance::{
        AtomicInstallParams, ContentInstallParams, ContentProvider, ContentSource, ContentType,
        InstallFileEntry, InstallPlan, InstallTarget, InstanceError, InstanceId,
        InstanceInstallStage, InstanceLocks, InstanceStorage, ModpackInstallParams, PackStorage,
        PlanOrigin, ProviderId, ResolveInstallPlanParams, ResolveTarget,
        app::extensions::InstanceStorageExt,
    },
    shared::capability::domain::CapabilityRegistry,
};

pub struct InstallContentUseCase {
    pack_storage: Arc<dyn PackStorage>,
    provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
    instance_storage: Arc<dyn InstanceStorage>,
    plan_executor: Arc<dyn InstallPlanExecutorPort>,
    instance_locks: Arc<InstanceLocks>,
}

impl InstallContentUseCase {
    pub fn new(
        pack_storage: Arc<dyn PackStorage>,
        provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
        instance_storage: Arc<dyn InstanceStorage>,
        plan_executor: Arc<dyn InstallPlanExecutorPort>,
        instance_locks: Arc<InstanceLocks>,
    ) -> Self {
        Self {
            pack_storage,
            provider_registry,
            instance_storage,
            plan_executor,
            instance_locks,
        }
    }

    /// Content installed into an existing instance is installed under the instance's lock
    /// ([`InstanceLocks`]): after any plan already running there, and never while the instance
    /// is being launched. A modpack makes an instance of its own.
    pub async fn execute(&self, install_params: ContentInstallParams) -> Result<(), InstanceError> {
        let _lock = match &install_params {
            ContentInstallParams::Atomic(params) => {
                Some(self.instance_locks.lock(&params.instance_id).await)
            }
            ContentInstallParams::Modpack(_) => None,
        };
        let provider_id = install_params.provider().to_owned();

        let provider = self
            .provider_registry
            .find_by_plugin_and_capability_id(
                provider_id.plugin_id.as_str(),
                provider_id.capability_id.as_str(),
            )
            .await
            .map_err(|_| InstanceError::ContentProviderNotFound {
                plugin_id: provider_id.plugin_id.to_string(),
                capability_id: provider_id.capability_id.to_string(),
            })?
            .capability;

        if let ContentInstallParams::Atomic(params) = &install_params
            && params.content_type == ContentType::Modpack
        {
            return Err(InstanceError::UnsupportedContentType {
                content_type: params.content_type,
            });
        }

        let (mut resolve_params, content_id) = resolve_params_of(&install_params);
        if let ResolveTarget::Existing { instance_id, .. } = &resolve_params.target {
            resolve_params.installed_version = self
                .installed_version(instance_id, &provider_id, &content_id)
                .await?;
        }
        let requested = resolve_params.target.clone();

        let plan = provider.resolve_install_plan(resolve_params).await?;
        self.execute_plan(&provider, plan, &requested, content_id)
            .await
    }

    /// The version of `content_id` from `provider_id`'s platform recorded in the instance's pack
    /// file, whichever plan or path installed it: the provider picks the next version from it
    /// instead of rolling a beta back to an older release.
    async fn installed_version(
        &self,
        instance_id: &InstanceId,
        provider_id: &ProviderId,
        content_id: &str,
    ) -> Result<Option<String>, InstanceError> {
        Ok(self
            .pack_storage
            .find_by_provider_id(instance_id, provider_id, content_id)
            .await?
            .and_then(|record| record.update?.remove(provider_id))
            .map(|info| info.version))
    }

    async fn execute_plan(
        &self,
        provider: &Arc<dyn ContentProvider>,
        plan: InstallPlan,
        requested: &ResolveTarget,
        content_id: String,
    ) -> Result<(), InstanceError> {
        // The plugin proxy checks this for plugins; a built-in provider is not behind it, and
        // a plan for another instance than the one asked about would install into it.
        if !requested.is_answered_by(&plan.target) {
            return Err(InstanceError::ContentProviderError {
                reason: format!(
                    "{} answered with a plan for another target than the one asked for",
                    provider.provider_id()
                ),
            });
        }

        let provider_id = provider.provider_id();

        // The content owns its files under the id the provider gives it, whichever spelling of
        // it the caller used: a slug resolves to the same project, but would not match on the
        // next update. A pack records its id; a single file names its content in its identity.
        let content_id = match &plan.target {
            InstallTarget::New(instance) => match &instance.pack_info {
                Some(pack_info) => {
                    // The new instance is recorded as the pack of that provider, but its files
                    // are owned by this one: updates would look for them under the wrong one.
                    if pack_info.provider_id != provider_id {
                        return Err(InstanceError::ContentProviderError {
                            reason: format!(
                                "{provider_id} answered with a pack of another provider ({})",
                                pack_info.provider_id
                            ),
                        });
                    }
                    pack_info.modpack_id.clone()
                }
                None => content_id,
            },
            InstallTarget::Existing { .. } => {
                own_identity(provider.as_ref(), &plan).unwrap_or(content_id)
            }
        };

        let origin = PlanOrigin {
            provider_id,
            content_id,
        };
        let instance_id = self.plan_executor.execute(plan, origin.clone()).await?;

        self.finish_pack_install(&instance_id, &origin).await
    }

    /// A pack's instance stays `PackInstalling` until a plan of that very pack has run through:
    /// then it is ready. A plan of anything else leaves the stage alone.
    async fn finish_pack_install(
        &self,
        instance_id: &InstanceId,
        origin: &PlanOrigin,
    ) -> Result<(), InstanceError> {
        let instance = self.instance_storage.get(instance_id).await?;
        if instance.install_stage != InstanceInstallStage::PackInstalling
            || !is_pack_of(&instance, origin)
        {
            return Ok(());
        }

        self.instance_storage
            .upsert_with(instance_id, |instance| {
                instance.install_stage = InstanceInstallStage::Installed;
                Ok(())
            })
            .await?;

        Ok(())
    }
}

/// The id `provider` itself gives the content of a plan of one file: the identity of that file,
/// when it is on the provider's own platform. A plan of several files, or with overrides, is a
/// pack's, whose files are not the content asked for.
fn own_identity(provider: &dyn ContentProvider, plan: &InstallPlan) -> Option<String> {
    let platform = provider.metadata().platform.as_deref()?;

    match plan.files.as_slice() {
        [
            InstallFileEntry {
                source:
                    ContentSource::Direct {
                        identity: Some(identity),
                        ..
                    },
                ..
            },
        ] if plan.overrides.is_empty() && identity.platform == platform => {
            Some(identity.content_id.clone())
        }
        _ => None,
    }
}

/// What to ask the provider for: an atomic install goes into the instance the caller chose, a
/// modpack into a new one; the installed version is looked up by the caller. Also the content id
/// the caller named, which owns the installed files.
fn resolve_params_of(params: &ContentInstallParams) -> (ResolveInstallPlanParams, String) {
    match params {
        ContentInstallParams::Atomic(AtomicInstallParams {
            base,
            instance_id,
            game_version,
            loader,
            ..
        }) => (
            ResolveInstallPlanParams {
                content_id: base.content_id.clone(),
                version: base.content_version.clone(),
                installed_version: None,
                target: ResolveTarget::Existing {
                    instance_id: instance_id.clone(),
                    game_version: game_version.clone(),
                    // A vanilla instance has no loader to match.
                    loader: loader.filter(|loader| *loader != ModLoader::Vanilla),
                },
            },
            base.content_id.clone(),
        ),
        ContentInstallParams::Modpack(ModpackInstallParams { base }) => (
            ResolveInstallPlanParams {
                content_id: base.content_id.clone(),
                version: base.content_version.clone(),
                installed_version: None,
                target: ResolveTarget::New,
            },
            base.content_id.clone(),
        ),
    }
}

#[async_trait]
impl InstallContentUseCasePort for InstallContentUseCase {
    async fn execute(&self, install_params: ContentInstallParams) -> Result<(), InstanceError> {
        self.execute(install_params).await
    }
}
