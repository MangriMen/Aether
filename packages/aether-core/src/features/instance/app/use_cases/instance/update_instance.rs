use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use async_trait::async_trait;

use crate::features::instance::app::ports::{InstallPlanExecutorPort, UpdateInstanceUseCasePort};
use crate::{
    features::{
        instance::{
            ContentProvider, InstallFileEntry, InstallPlan, Instance, InstanceError,
            InstanceInstallStage, InstanceLocks, InstanceStorage, InstanceStorageExt, PackFile,
            PackInfo, PackSource, PackStorage, PlanOrigin, ResolveInstallPlanParams, ResolveTarget,
            Updater,
        },
        minecraft::ModLoader,
    },
    shared::capability::domain::CapabilityRegistry,
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
    pack_storage: Arc<dyn PackStorage>,
    plan_executor: Arc<dyn InstallPlanExecutorPort>,
    instance_locks: Arc<InstanceLocks>,
}

/// How an update went through.
enum Updated {
    /// By a plan; the version it installed, when the provider named it.
    Plan { version_id: Option<String> },
    /// By the plugin's updater, which keeps the pack info itself.
    Updater,
}

impl UpdateInstanceUseCase {
    pub fn new(
        instance_storage: Arc<dyn InstanceStorage>,
        updaters_registry: Arc<dyn CapabilityRegistry<Arc<dyn Updater>>>,
        provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
        pack_storage: Arc<dyn PackStorage>,
        plan_executor: Arc<dyn InstallPlanExecutorPort>,
        instance_locks: Arc<InstanceLocks>,
    ) -> Self {
        Self {
            instance_storage,
            updaters_registry,
            provider_registry,
            pack_storage,
            plan_executor,
            instance_locks,
        }
    }

    pub async fn update_by_plugin(
        &self,
        instance_id: &str,
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
                modpack_id: pack_info.modpack_id.clone(),
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
            instance_id: instance.id().to_owned(),
            game_version: instance.game_version.clone(),
            loader: (instance.loader != ModLoader::Vanilla)
                .then(|| instance.loader.as_str().to_owned()),
        };
        let plan = match provider
            .resolve_install_plan(ResolveInstallPlanParams {
                content_id: pack_info.modpack_id.clone(),
                version: version_id.clone(),
                installed_version: Some(pack_info.version_id.clone()),
                target: requested.clone(),
            })
            .await
        {
            Ok(plan) => plan,
            // A plugin that deploys its packs itself has no plans to give.
            Err(InstanceError::UnsupportedOperation {
                operation: "resolve_install_plan",
                ..
            }) => {
                self.update_by_plugin(instance.id(), pack_info).await?;
                return Ok(Updated::Updater);
            }
            Err(err) => return Err(err),
        };

        // A built-in provider is not behind the plugin proxy that checks this.
        if !requested.is_answered_by(&plan.target) {
            return Err(InstanceError::ContentProviderError {
                reason: format!(
                    "{} answered with a plan for another target than the one asked for",
                    provider.provider_id()
                ),
            });
        }

        let origin = PlanOrigin {
            provider_id: pack_info.provider_id.clone(),
            content_id: pack_info.modpack_id.clone(),
        };
        let adopted = self
            .adopt_ownerless_files(
                provider.as_ref(),
                instance.id(),
                &requested,
                pack_info,
                &origin,
            )
            .await?;

        // The executor rolls a failed plan back to the records it found, the adopted ones; the
        // adoption is part of the update and goes back with it.
        if let Err(err) = self.plan_executor.execute(plan, origin).await {
            self.disown(instance.id(), adopted).await;
            return Err(err);
        }

        Ok(Updated::Plan { version_id })
    }

    /// The version an update takes, as a check for updates finds it: by the release channel of
    /// the installed one, never older. `None` — the provider cannot tell without the plan, which
    /// then picks it the same way.
    async fn version_to_install(
        &self,
        provider: &dyn ContentProvider,
        pack_info: &PackInfo,
    ) -> Result<Option<String>, InstanceError> {
        match provider
            .resolve_pack_metadata(PackSource::Content {
                content_id: pack_info.modpack_id.clone(),
                version: None,
                installed_version: Some(pack_info.version_id.clone()),
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

    /// A pack installed before plans recorded owners has its files recorded without one, so an
    /// update would leave the files the pack dropped next to their replacements. The plan of the
    /// installed version names the pack's paths: the ownerless records at them become the pack's.
    ///
    /// Without that plan (the version is gone, no network) nothing is adopted and the update
    /// goes on: an ownerless file may be the user's own, and only an owned one is ever removed.
    ///
    /// Returns the adopted records as they were, for a failed update to put back ([`disown`]).
    ///
    /// [`disown`]: Self::disown
    async fn adopt_ownerless_files(
        &self,
        provider: &dyn ContentProvider,
        instance_id: &str,
        requested: &ResolveTarget,
        pack_info: &PackInfo,
        origin: &PlanOrigin,
    ) -> Result<Vec<(String, PackFile)>, InstanceError> {
        let records = self.pack_storage.get_pack_files(instance_id).await?;
        let owned = records
            .values()
            .any(|record| record.installed_by.as_ref() == Some(origin));
        if owned || records.values().all(|record| record.installed_by.is_some()) {
            return Ok(Vec::new());
        }

        let installed_plan = provider
            .resolve_install_plan(ResolveInstallPlanParams {
                content_id: pack_info.modpack_id.clone(),
                version: Some(pack_info.version_id.clone()),
                installed_version: Some(pack_info.version_id.clone()),
                target: requested.clone(),
            })
            .await;
        let installed_plan = match installed_plan {
            Ok(plan) => plan,
            Err(err) => {
                tracing::warn!(
                    "Cannot tell the files of pack \"{}\" version \"{}\" in \"{instance_id}\" \
                     from the user's: {err}. Updating without removing the files the pack drops",
                    pack_info.modpack_id,
                    pack_info.version_id
                );
                return Ok(Vec::new());
            }
        };

        let ownerless = ownerless_pack_files(records, &installed_plan);
        if ownerless.is_empty() {
            return Ok(Vec::new());
        }
        let (paths, files): (Vec<String>, Vec<PackFile>) = ownerless
            .iter()
            .map(|(path, record)| {
                let mut record = record.clone();
                record.installed_by = Some(origin.clone());
                (path.clone(), record)
            })
            .unzip();

        tracing::info!(
            "Adopting {} files of \"{instance_id}\" as files of pack \"{}\"",
            paths.len(),
            pack_info.modpack_id
        );
        self.pack_storage
            .update_pack_file_many(instance_id, &paths, &files)
            .await?;

        Ok(ownerless)
    }

    /// Put the records adopted for an update that failed back as they were, without an owner.
    /// A failure is only logged: the update has failed already, and an adopted record is still
    /// the pack's file.
    async fn disown(&self, instance_id: &str, adopted: Vec<(String, PackFile)>) {
        if adopted.is_empty() {
            return;
        }

        let (paths, files): (Vec<String>, Vec<PackFile>) = adopted.into_iter().unzip();
        if let Err(err) = self
            .pack_storage
            .update_pack_file_many(instance_id, &paths, &files)
            .await
        {
            tracing::error!(
                "Failed to put back the {} files of \"{instance_id}\" adopted for the failed \
                 update: {err}",
                paths.len()
            );
        }
    }
}

/// The ownerless records at a path of `plan`, compared as the executor compares paths: with
/// either separator, and without case where the file system ignores it.
fn ownerless_pack_files(
    records: HashMap<String, PackFile>,
    plan: &InstallPlan,
) -> Vec<(String, PackFile)> {
    ownerless_pack_files_by(records, plan, cfg!(any(windows, target_os = "macos")))
}

fn ownerless_pack_files_by(
    records: HashMap<String, PackFile>,
    plan: &InstallPlan,
    ignores_case: bool,
) -> Vec<(String, PackFile)> {
    let key = |path: &str| {
        InstallFileEntry::path_key(
            &InstallFileEntry::normalize_relative_path(path),
            ignores_case,
        )
    };
    let planned: HashSet<String> = plan
        .files
        .iter()
        .map(|file| key(&file.relative_path))
        .collect();

    records
        .into_iter()
        .filter(|(path, record)| record.installed_by.is_none() && planned.contains(&key(path)))
        .collect()
}

#[async_trait]
impl UpdateInstanceUseCasePort for UpdateInstanceUseCase {
    async fn execute(&self, instance_id: String) -> Result<(), InstanceError> {
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
                    current.version_id = version_id;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::instance::{ContentSource, ContentType};

    fn plan_of(path: &str) -> InstallPlan {
        InstallPlan {
            target: crate::features::instance::InstallTarget::existing("instance"),
            files: vec![InstallFileEntry::new(
                ContentSource::direct("https://example.com/a.jar".into(), None),
                path,
                ContentType::Mod,
            )],
            overrides: Vec::new(),
        }
    }

    fn record_at(path: &str) -> HashMap<String, PackFile> {
        HashMap::from([(
            path.to_owned(),
            PackFile::from_hash("a.jar".into(), "hash".into()),
        )])
    }

    #[test]
    fn should_not_adopt_a_record_when_it_differs_from_the_planned_path_in_case_on_a_case_sensitive_file_system()
     {
        let adopted =
            ownerless_pack_files_by(record_at("mods/A.jar"), &plan_of("mods/a.jar"), false);

        assert!(adopted.is_empty());
    }

    #[test]
    fn should_adopt_a_record_when_it_differs_from_the_planned_path_in_case_on_a_case_insensitive_file_system()
     {
        let adopted =
            ownerless_pack_files_by(record_at("mods/A.jar"), &plan_of("mods/a.jar"), true);

        assert_eq!(adopted.len(), 1);
    }

    #[test]
    fn should_adopt_a_record_when_it_differs_from_the_planned_path_in_the_separator() {
        let adopted =
            ownerless_pack_files_by(record_at("mods\\a.jar"), &plan_of("mods/a.jar"), false);

        assert_eq!(adopted.len(), 1);
    }
}
