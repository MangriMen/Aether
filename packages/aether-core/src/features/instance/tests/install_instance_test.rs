use crate::features::instance::ContentId;
use crate::features::instance::InstanceName;
use crate::features::minecraft::GameVersion;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tempfile::TempDir;

use crate::features::{
    events::{
        Event, EventEmitter, EventError, ProgressBarId, ProgressBarStorage, ProgressEvent,
        ProgressEventType, ProgressService, ProgressServiceImpl, ProgressStatus,
        infra::InMemoryProgressBarStorage,
        test_support::{ChannelEmitter, sent},
    },
    instance::{
        InstallInstanceUseCase, Instance, InstanceBuilder, InstanceError, InstanceId,
        InstanceInstallStage, InstanceStorage, PackInfo, ProviderId,
    },
    minecraft::{
        GameDownloadParams, InstallMinecraftParams, LoaderVersionId, LoaderVersionPreference,
        LoaderVersionService, MinecraftApplicationError, MinecraftDomainError,
        MinecraftInstallService, ModLoader, modded,
    },
    settings::LocationInfo,
};

struct MemStorage {
    instances: Mutex<Vec<Instance>>,
}

#[async_trait]
impl InstanceStorage for MemStorage {
    async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
        Ok(self.instances.lock().unwrap().clone())
    }

    async fn get(&self, id: &InstanceId) -> Result<Instance, InstanceError> {
        self.instances
            .lock()
            .unwrap()
            .iter()
            .find(|i| i.id() == id)
            .cloned()
            .ok_or_else(|| InstanceError::NotFound {
                instance_id: id.to_string(),
            })
    }

    async fn upsert(&self, instance: &Instance) -> Result<(), InstanceError> {
        let mut guard = self.instances.lock().unwrap();
        match guard.iter().position(|i| i.id() == instance.id()) {
            Some(pos) => guard[pos] = instance.clone(),
            None => guard.push(instance.clone()),
        }
        Ok(())
    }

    async fn update(
        &self,
        id: &InstanceId,
        change: crate::features::instance::InstanceChange<'_>,
    ) -> Result<Instance, InstanceError> {
        let mut instance = self.get(id).await?;
        if change(&mut instance)? {
            self.upsert(&instance).await?;
        }
        Ok(instance)
    }

    async fn remove(&self, id: &InstanceId) -> Result<(), InstanceError> {
        self.instances.lock().unwrap().retain(|i| i.id() != id);
        Ok(())
    }
}

struct StubMinecraftInstall {
    fail: bool,
}

#[async_trait]
impl MinecraftInstallService for StubMinecraftInstall {
    async fn download_game(
        &self,
        _: GameDownloadParams,
        _: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftApplicationError> {
        unimplemented!("installing an instance downloads nothing ahead")
    }

    async fn execute(
        &self,
        _params: InstallMinecraftParams,
        _loading_bar: Option<&ProgressBarId>,
        _force: bool,
    ) -> Result<(), MinecraftApplicationError> {
        if self.fail {
            Err(MinecraftApplicationError::Storage(
                std::io::Error::other("boom").into(),
            ))
        } else {
            Ok(())
        }
    }
}

/// Progress is best-effort in the use case (`*_safe`), so failing every call is enough.
/// A loader service that resolves every preference to the id it holds, or `0.16.10`.
struct NoLoaders;

#[async_trait]
impl LoaderVersionService for NoLoaders {
    async fn resolve(
        &self,
        _: &GameVersion,
        _: &ModLoader,
        preference: Option<&LoaderVersionPreference>,
    ) -> Result<Option<modded::LoaderVersion>, MinecraftDomainError> {
        let id = match preference {
            Some(LoaderVersionPreference::Exact(id)) => id.to_string(),
            _ => "0.16.10".to_string(),
        };
        Ok(Some(modded::LoaderVersion {
            url: format!("https://meta/{id}"),
            id,
            stable: true,
        }))
    }

    async fn try_get_default(
        &self,
        _: &GameVersion,
        _: &ModLoader,
    ) -> Result<Option<LoaderVersionPreference>, MinecraftDomainError> {
        Ok(None)
    }
}

struct NoProgress;

#[async_trait]
impl ProgressService for NoProgress {
    async fn init_progress(
        &self,
        _event_type: ProgressEventType,
        _total: f64,
        _message: String,
    ) -> Result<ProgressBarId, EventError> {
        Err(EventError::NotInitialized)
    }
    async fn init_or_edit_progress(
        &self,
        _progress_bar_id: Option<ProgressBarId>,
        _event_type: ProgressEventType,
        _total: f64,
        _message: String,
    ) -> Result<ProgressBarId, EventError> {
        Err(EventError::NotInitialized)
    }
    async fn emit_progress(
        &self,
        _progress_bar_id: &ProgressBarId,
        _increment_frac: f64,
        _message: Option<&str>,
    ) -> Result<(), EventError> {
        Ok(())
    }
    async fn edit_progress(
        &self,
        _progress_bar_id: &ProgressBarId,
        _event_type: ProgressEventType,
        _total: f64,
        _message: String,
    ) -> Result<(), EventError> {
        Ok(())
    }
}

fn pack_info() -> PackInfo {
    PackInfo {
        provider_id: ProviderId::from_parts("aether", "modrinth").unwrap(),
        modpack_id: ContentId::try_from("pack").unwrap(),
        version_id: "1".to_owned(),
    }
}

async fn run_install(
    pack: Option<PackInfo>,
    stage: InstanceInstallStage,
    fail: bool,
) -> (Instance, bool) {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().to_path_buf(),
    ));

    let mut instance = InstanceBuilder::new(
        InstanceId::try_from("inst").unwrap(),
        InstanceName::parse("inst").unwrap(),
        GameVersion::from_static("1.20.1"),
        ModLoader::Vanilla,
    )
    .with_pack_info(pack)
    .build();
    instance.install_stage = stage;
    let id = instance.id().clone();

    let storage = Arc::new(MemStorage {
        instances: Mutex::new(vec![instance]),
    });
    let use_case = InstallInstanceUseCase::new(
        storage.clone(),
        Arc::new(NoLoaders),
        Arc::new(StubMinecraftInstall { fail }),
        Arc::new(NoProgress),
        location_info,
    );

    let result = use_case.execute(id.clone(), false, None).await;
    (storage.get(&id).await.unwrap(), result.is_ok())
}

#[tokio::test]
async fn should_become_installed_when_the_instance_is_plain() {
    let (instance, ok) = run_install(None, InstanceInstallStage::NotInstalled, false).await;
    assert!(ok);
    assert_eq!(instance.install_stage, InstanceInstallStage::Installed);
}

/// Created for a pack (`CreateInstanceUseCase::execute_for_pack`): installing the game does not
/// make it launchable, deploying the pack does.
#[tokio::test]
async fn should_stay_pack_installing_when_content_is_not_deployed_yet() {
    let (instance, ok) = run_install(
        Some(pack_info()),
        InstanceInstallStage::PackInstalling,
        false,
    )
    .await;
    assert!(ok);
    assert_eq!(instance.install_stage, InstanceInstallStage::PackInstalling);
}

/// The game of an installed pack's instance reinstalled — by the health check before a launch,
/// or by a change of game — leaves it launchable: `pack_info` alone says nothing is pending.
#[tokio::test]
async fn should_stay_installed_when_the_game_of_an_installed_pack_is_reinstalled() {
    let (instance, ok) =
        run_install(Some(pack_info()), InstanceInstallStage::Installed, false).await;
    assert!(ok);
    assert_eq!(instance.install_stage, InstanceInstallStage::Installed);
}

/// A plugin's old install path creates its instance with `pack_info` through the plain create,
/// and has no way to move it on: it must not be left unlaunchable.
#[tokio::test]
async fn should_become_installed_when_the_pack_info_was_not_created_for_a_pack() {
    let (instance, ok) =
        run_install(Some(pack_info()), InstanceInstallStage::NotInstalled, false).await;
    assert!(ok);
    assert_eq!(instance.install_stage, InstanceInstallStage::Installed);
}

/// A failed reinstall (health check, game change) must not demote a working instance.
#[tokio::test]
async fn should_stay_installed_when_reinstalling_a_working_instance_fails() {
    let (instance, ok) = run_install(None, InstanceInstallStage::Installed, true).await;
    assert!(!ok);
    assert_eq!(instance.install_stage, InstanceInstallStage::Installed);
}

#[tokio::test]
async fn should_become_not_installed_when_the_first_install_fails() {
    let (instance, ok) = run_install(None, InstanceInstallStage::NotInstalled, true).await;
    assert!(!ok);
    assert_eq!(instance.install_stage, InstanceInstallStage::NotInstalled);
}

/// Reads the stage of the instance while the game is being installed.
struct StageProbe {
    storage: Arc<MemStorage>,
    seen: Mutex<Vec<InstanceInstallStage>>,
}

#[async_trait]
impl MinecraftInstallService for StageProbe {
    async fn download_game(
        &self,
        _: GameDownloadParams,
        _: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftApplicationError> {
        unimplemented!("installing an instance downloads nothing ahead")
    }

    async fn execute(
        &self,
        _params: InstallMinecraftParams,
        _loading_bar: Option<&ProgressBarId>,
        _force: bool,
    ) -> Result<(), MinecraftApplicationError> {
        let stage = self.storage.instances.lock().unwrap()[0].install_stage;
        self.seen.lock().unwrap().push(stage);
        Ok(())
    }
}

/// While an installed instance is reinstalled it is `Installing`, which keeps it from being
/// launched or changed under the install; it is `Installed` again once the install is done.
#[tokio::test]
async fn should_be_installing_while_a_working_instance_is_reinstalled() {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().to_path_buf(),
    ));
    let mut instance = InstanceBuilder::new(
        InstanceId::try_from("inst").unwrap(),
        InstanceName::parse("inst").unwrap(),
        GameVersion::from_static("1.20.1"),
        ModLoader::Vanilla,
    )
    .build();
    instance.install_stage = InstanceInstallStage::Installed;
    let id = instance.id().clone();
    let storage = Arc::new(MemStorage {
        instances: Mutex::new(vec![instance]),
    });
    let probe = Arc::new(StageProbe {
        storage: storage.clone(),
        seen: Mutex::default(),
    });
    let use_case = InstallInstanceUseCase::new(
        storage.clone(),
        Arc::new(NoLoaders),
        probe.clone(),
        Arc::new(NoProgress),
        location_info,
    );

    use_case.execute(id.clone(), true, None).await.unwrap();

    assert_eq!(
        *probe.seen.lock().unwrap(),
        [InstanceInstallStage::Installing]
    );
    assert_eq!(
        storage.get(&id).await.unwrap().install_stage,
        InstanceInstallStage::Installed
    );
}

#[tokio::test]
async fn should_reset_only_transient_stages_on_startup_sweep() {
    let make = |name: &str, stage| {
        let mut i = InstanceBuilder::new(
            InstanceId::try_from(name).unwrap(),
            InstanceName::parse(name).unwrap(),
            GameVersion::from_static("1.20.1"),
            ModLoader::Vanilla,
        )
        .build();
        i.install_stage = stage;
        i
    };
    let storage = MemStorage {
        instances: Mutex::new(vec![
            make("a", InstanceInstallStage::Installing),
            make("b", InstanceInstallStage::PackInstalling),
            make("c", InstanceInstallStage::Installed),
            make("d", InstanceInstallStage::NotInstalled),
        ]),
    };

    let reset = crate::features::instance::reset_stuck_install_stages(&storage)
        .await
        .unwrap();

    assert_eq!(reset, 2);
    let stages: Vec<_> = storage
        .instances
        .lock()
        .unwrap()
        .iter()
        .map(|i| i.install_stage)
        .collect();
    assert_eq!(
        stages,
        [
            InstanceInstallStage::NotInstalled,
            InstanceInstallStage::NotInstalled,
            InstanceInstallStage::Installed,
            InstanceInstallStage::NotInstalled,
        ]
    );
}

/// A storage that cannot even be listed.
struct UnlistableStorage;

#[async_trait]
impl InstanceStorage for UnlistableStorage {
    async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
        Err(InstanceError::storage(
            crate::features::instance::StorageError::other("boom"),
        ))
    }

    async fn get(&self, id: &InstanceId) -> Result<Instance, InstanceError> {
        Err(InstanceError::NotFound {
            instance_id: id.to_string(),
        })
    }

    async fn upsert(&self, _instance: &Instance) -> Result<(), InstanceError> {
        Ok(())
    }

    async fn update(
        &self,
        id: &InstanceId,
        _change: crate::features::instance::InstanceChange<'_>,
    ) -> Result<Instance, InstanceError> {
        self.get(id).await
    }

    async fn remove(&self, _id: &InstanceId) -> Result<(), InstanceError> {
        Ok(())
    }
}

#[tokio::test]
async fn should_fail_the_startup_sweep_when_the_instances_cannot_be_listed() {
    let result = crate::features::instance::reset_stuck_install_stages(&UnlistableStorage).await;

    assert!(matches!(result, Err(InstanceError::Storage(_))));
}

struct NoEmitter;

#[async_trait]
impl EventEmitter<Event> for NoEmitter {
    async fn emit(&self, _event: Event) -> Result<(), EventError> {
        Ok(())
    }

    fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
}

/// Records the parent of the game's bar, every time the game installs.
struct ParentOfBar(
    Arc<InMemoryProgressBarStorage>,
    Mutex<Vec<Option<uuid::Uuid>>>,
);

#[async_trait]
impl MinecraftInstallService for ParentOfBar {
    async fn download_game(
        &self,
        _: GameDownloadParams,
        _: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftApplicationError> {
        unimplemented!("installing an instance downloads nothing ahead")
    }

    async fn execute(
        &self,
        _params: InstallMinecraftParams,
        loading_bar: Option<&ProgressBarId>,
        _force: bool,
    ) -> Result<(), MinecraftApplicationError> {
        let bar = self.0.get(loading_bar.unwrap().id).await.unwrap().clone();
        self.1.lock().unwrap().push(bar.parent_id);
        Ok(())
    }
}

/// Installing the game of a pack's new instance reports under the pack's bar; on its own, at
/// the top.
#[tokio::test]
async fn should_put_the_bar_of_the_game_under_the_parent_bar_when_there_is_one() {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().to_path_buf(),
    ));
    let instance = InstanceBuilder::new(
        InstanceId::try_from("inst").unwrap(),
        InstanceName::parse("inst").unwrap(),
        GameVersion::from_static("1.20.1"),
        ModLoader::Vanilla,
    )
    .build();
    let id = instance.id().clone();
    let bars = Arc::new(InMemoryProgressBarStorage::default());
    let progress = Arc::new(ProgressServiceImpl::new(Arc::new(NoEmitter), bars.clone()));
    let minecraft = Arc::new(ParentOfBar(bars, Mutex::default()));
    let use_case = InstallInstanceUseCase::new(
        Arc::new(MemStorage {
            instances: Mutex::new(vec![instance]),
        }),
        Arc::new(NoLoaders),
        minecraft.clone(),
        progress.clone(),
        location_info,
    );
    let pack = progress
        .init_progress(
            ProgressEventType::PluginDownload {
                plugin_name: "pack".to_owned(),
            },
            1.0,
            String::new(),
        )
        .await
        .unwrap();

    use_case
        .execute(id.clone(), false, Some(&pack))
        .await
        .unwrap();
    use_case.execute(id, false, None).await.unwrap();

    assert_eq!(*minecraft.1.lock().unwrap(), [Some(pack.id), None]);
}

#[tokio::test]
async fn should_stay_pack_installing_when_the_install_fails() {
    let (instance, ok) = run_install(
        Some(pack_info()),
        InstanceInstallStage::PackInstalling,
        true,
    )
    .await;
    assert!(!ok);
    assert_eq!(instance.install_stage, InstanceInstallStage::PackInstalling);
}

struct RenameDuringInstall(Arc<MemStorage>);

#[async_trait]
impl MinecraftInstallService for RenameDuringInstall {
    async fn download_game(
        &self,
        _: GameDownloadParams,
        _: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftApplicationError> {
        unimplemented!("installing an instance downloads nothing ahead")
    }

    async fn execute(
        &self,
        _params: InstallMinecraftParams,
        _loading_bar: Option<&ProgressBarId>,
        _force: bool,
    ) -> Result<(), MinecraftApplicationError> {
        let id = InstanceId::try_from("inst").unwrap();
        self.0
            .update(
                &id,
                Box::new(|instance| {
                    instance.rename(InstanceName::parse("renamed").unwrap());
                    Ok(true)
                }),
            )
            .await
            .unwrap();
        Ok(())
    }
}

#[tokio::test]
async fn should_keep_an_edit_made_while_the_game_installs() {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().to_path_buf(),
    ));
    let instance = InstanceBuilder::new(
        InstanceId::try_from("inst").unwrap(),
        InstanceName::parse("inst").unwrap(),
        GameVersion::from_static("1.20.1"),
        ModLoader::Vanilla,
    )
    .build();
    let id = instance.id().clone();
    let storage = Arc::new(MemStorage {
        instances: Mutex::new(vec![instance]),
    });
    let use_case = InstallInstanceUseCase::new(
        storage.clone(),
        Arc::new(NoLoaders),
        Arc::new(RenameDuringInstall(storage.clone())),
        Arc::new(NoProgress),
        location_info,
    );

    use_case.execute(id.clone(), false, None).await.unwrap();

    let stored = storage.get(&id).await.unwrap();
    assert_eq!(stored.name().as_str(), "renamed");
    assert_eq!(stored.install_stage, InstanceInstallStage::Installed);
}

async fn loader_version_after_install(
    loader_version: LoaderVersionPreference,
) -> Option<LoaderVersionPreference> {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().to_path_buf(),
    ));
    let instance = InstanceBuilder::new(
        InstanceId::try_from("inst").unwrap(),
        InstanceName::parse("inst").unwrap(),
        GameVersion::from_static("1.20.1"),
        ModLoader::Fabric,
    )
    .with_loader_version(Some(loader_version))
    .build();
    let id = instance.id().clone();
    let storage = Arc::new(MemStorage {
        instances: Mutex::new(vec![instance]),
    });
    let use_case = InstallInstanceUseCase::new(
        storage.clone(),
        Arc::new(NoLoaders),
        Arc::new(StubMinecraftInstall { fail: false }),
        Arc::new(NoProgress),
        location_info,
    );

    use_case.execute(id.clone(), false, None).await.unwrap();
    storage.get(&id).await.unwrap().loader_version
}

#[tokio::test]
async fn should_pin_the_loader_version_when_an_instance_holds_stable_or_latest() {
    for preference in [
        LoaderVersionPreference::Stable,
        LoaderVersionPreference::Latest,
    ] {
        assert_eq!(
            loader_version_after_install(preference).await,
            Some(LoaderVersionPreference::Exact(
                LoaderVersionId::from_static("0.16.10")
            ))
        );
    }
}

#[tokio::test]
async fn should_keep_the_loader_version_when_it_is_exact() {
    let exact = LoaderVersionPreference::Exact(LoaderVersionId::from_static("0.15.0"));

    assert_eq!(
        loader_version_after_install(exact.clone()).await,
        Some(exact)
    );
}

/// Starts, then waits for ever: an install the user calls off.
struct HangingInstall(tokio::sync::Notify);

#[async_trait]
impl MinecraftInstallService for HangingInstall {
    async fn download_game(
        &self,
        _: GameDownloadParams,
        _: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftApplicationError> {
        unimplemented!("installing an instance downloads nothing ahead")
    }

    async fn execute(
        &self,
        _: InstallMinecraftParams,
        _: Option<&ProgressBarId>,
        _: bool,
    ) -> Result<(), MinecraftApplicationError> {
        self.0.notify_one();
        std::future::pending().await
    }
}

/// Calls the install of an instance in `stage` off from its bar, as the button does.
async fn call_off_install(
    stage: InstanceInstallStage,
) -> (Instance, Result<(), InstanceError>, Vec<ProgressEvent>) {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().to_path_buf(),
    ));
    let mut instance = InstanceBuilder::new(
        InstanceId::try_from("inst").unwrap(),
        InstanceName::parse("inst").unwrap(),
        GameVersion::from_static("1.20.1"),
        ModLoader::Vanilla,
    )
    .build();
    instance.install_stage = stage;
    let id = instance.id().clone();
    let storage = Arc::new(MemStorage {
        instances: Mutex::new(vec![instance]),
    });
    let (tx, mut events) = tokio::sync::mpsc::unbounded_channel();
    let bars = Arc::new(InMemoryProgressBarStorage::default());
    let progress = Arc::new(ProgressServiceImpl::new(
        Arc::new(ChannelEmitter(tx)),
        bars.clone(),
    ));
    let minecraft = Arc::new(HangingInstall(tokio::sync::Notify::new()));
    let use_case = InstallInstanceUseCase::new(
        storage.clone(),
        Arc::new(NoLoaders),
        minecraft.clone(),
        progress.clone(),
        location_info,
    );

    let run = use_case.execute(id.clone(), false, None);
    tokio::pin!(run);
    tokio::select! {
        result = &mut run => panic!("the install cannot end by itself: {result:?}"),
        () = minecraft.0.notified() => {}
    }
    let game_bar = bars.list().await.into_iter().next().unwrap();
    progress.cancel_progress(game_bar.id).await.unwrap();
    let result = tokio::time::timeout(std::time::Duration::from_secs(5), run)
        .await
        .expect("the install did not stop");
    // The bar ends on a task of its own.
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while !bars.list().await.is_empty() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("the bar was left behind");
    (storage.get(&id).await.unwrap(), result, sent(&mut events))
}

#[tokio::test]
async fn should_end_cancelled_and_not_installed_when_the_first_install_is_called_off() {
    let (instance, result, events) = call_off_install(InstanceInstallStage::NotInstalled).await;

    assert!(
        matches!(result, Err(InstanceError::Cancelled)),
        "{result:?}"
    );
    assert_eq!(instance.install_stage, InstanceInstallStage::NotInstalled);
    assert_eq!(
        events.last().map(|event| event.status.clone()),
        Some(ProgressStatus::Cancelled)
    );
}

#[tokio::test]
async fn should_stay_installed_when_the_reinstall_of_a_working_instance_is_called_off() {
    let (instance, result, _) = call_off_install(InstanceInstallStage::Installed).await;

    assert!(
        matches!(result, Err(InstanceError::Cancelled)),
        "{result:?}"
    );
    assert_eq!(instance.install_stage, InstanceInstallStage::Installed);
}

#[tokio::test]
async fn should_end_cancelled_when_the_instance_is_removed_while_the_game_installs() {
    let temp = TempDir::new().unwrap();
    let instance = InstanceBuilder::new(
        InstanceId::try_from("inst").unwrap(),
        InstanceName::parse("inst").unwrap(),
        GameVersion::from_static("1.20.1"),
        ModLoader::Vanilla,
    )
    .build();
    let id = instance.id().clone();
    let minecraft = Arc::new(HangingInstall(tokio::sync::Notify::new()));
    let removals = Arc::new(crate::features::instance::InstallCancellations::default());
    let use_case = InstallInstanceUseCase::new(
        Arc::new(MemStorage {
            instances: Mutex::new(vec![instance]),
        }),
        Arc::new(NoLoaders),
        minecraft.clone(),
        Arc::new(NoProgress),
        Arc::new(LocationInfo::new(
            temp.path().join("settings"),
            temp.path().to_path_buf(),
        )),
    )
    .with_cancellations(removals.clone());

    let run = use_case.execute(id.clone(), false, None);
    tokio::pin!(run);
    tokio::select! {
        result = &mut run => panic!("the install cannot end by itself: {result:?}"),
        () = minecraft.0.notified() => {}
    }
    // The removal waits for the install to leave, which it does once it is called off.
    let removal = {
        let (removals, id) = (removals.clone(), id.clone());
        tokio::spawn(async move {
            let _removal = removals.begin_removal(&id).await;
        })
    };
    let result = tokio::time::timeout(std::time::Duration::from_secs(5), run)
        .await
        .expect("the install did not stop");
    removal.await.unwrap();

    assert!(
        matches!(result, Err(InstanceError::Cancelled)),
        "{result:?}"
    );
}

/// A removal that came first has taken the instance by the time the install stops.
#[tokio::test]
async fn should_end_cancelled_when_the_instance_is_gone_by_the_time_the_install_stops() {
    let temp = TempDir::new().unwrap();
    let instance = InstanceBuilder::new(
        InstanceId::try_from("inst").unwrap(),
        InstanceName::parse("inst").unwrap(),
        GameVersion::from_static("1.20.1"),
        ModLoader::Vanilla,
    )
    .build();
    let id = instance.id().clone();
    let storage = Arc::new(MemStorage {
        instances: Mutex::new(vec![instance]),
    });
    let minecraft = Arc::new(HangingInstall(tokio::sync::Notify::new()));
    let removals = Arc::new(crate::features::instance::InstallCancellations::default());
    let progress = Arc::new(ProgressServiceImpl::new(
        Arc::new(NoEmitter),
        Arc::new(InMemoryProgressBarStorage::default()),
    ));
    let use_case = InstallInstanceUseCase::new(
        storage.clone(),
        Arc::new(NoLoaders),
        minecraft.clone(),
        progress.clone(),
        Arc::new(LocationInfo::new(
            temp.path().join("settings"),
            temp.path().to_path_buf(),
        )),
    )
    .with_cancellations(removals.clone());
    // Under the bar of a pack, as when a pack installs the game.
    let pack = progress
        .init_progress(
            ProgressEventType::PluginDownload {
                plugin_name: "pack".to_owned(),
            },
            1.0,
            String::new(),
        )
        .await
        .unwrap();

    let run = use_case.execute(id.clone(), false, Some(&pack));
    tokio::pin!(run);
    tokio::select! {
        result = &mut run => panic!("the install cannot end by itself: {result:?}"),
        () = minecraft.0.notified() => {}
    }
    storage.remove(&id).await.unwrap();
    // The removal waits for the install to leave, which it does once it is called off.
    let removal = {
        let (removals, id) = (removals.clone(), id.clone());
        tokio::spawn(async move {
            let _removal = removals.begin_removal(&id).await;
        })
    };
    let result = tokio::time::timeout(std::time::Duration::from_secs(5), run)
        .await
        .expect("the install did not stop");
    removal.await.unwrap();

    assert!(
        matches!(result, Err(InstanceError::Cancelled)),
        "{result:?}"
    );
}
