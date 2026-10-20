use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tempfile::TempDir;

use crate::features::{
    events::{
        Event, EventEmitter, EventError, ProgressBarId, ProgressBarStorage, ProgressEventType,
        ProgressService, ProgressServiceImpl, infra::InMemoryProgressBarStorage,
    },
    instance::{
        InstallInstanceUseCase, Instance, InstanceBuilder, InstanceError, InstanceId,
        InstanceInstallStage, InstanceStorage, PackInfo, ProviderId,
    },
    minecraft::{
        InstallMinecraftParams, MinecraftApplicationError, MinecraftInstallService, ModLoader,
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
        provider_id: ProviderId {
            plugin_id: "aether".to_owned(),
            capability_id: "modrinth".to_owned(),
        },
        modpack_id: "pack".to_owned(),
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
        "inst".to_owned(),
        "1.20.1".to_owned(),
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
        "inst".to_owned(),
        "1.20.1".to_owned(),
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
            name.to_owned(),
            "1.20.1".to_owned(),
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
        Err(InstanceError::Storage("boom".to_owned()))
    }

    async fn get(&self, id: &InstanceId) -> Result<Instance, InstanceError> {
        Err(InstanceError::NotFound {
            instance_id: id.to_string(),
        })
    }

    async fn upsert(&self, _instance: &Instance) -> Result<(), InstanceError> {
        Ok(())
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
        "inst".to_owned(),
        "1.20.1".to_owned(),
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

/// Renames the instance while "installing", as an edit made during a long install would.
struct RenameDuringInstall(Arc<MemStorage>);

#[async_trait]
impl MinecraftInstallService for RenameDuringInstall {
    async fn execute(
        &self,
        params: InstallMinecraftParams,
        _loading_bar: Option<&ProgressBarId>,
        _force: bool,
    ) -> Result<(), MinecraftApplicationError> {
        let _ = params;
        let mut instance = self
            .0
            .get(&InstanceId::try_from("inst").unwrap())
            .await
            .unwrap();
        instance.rename("Renamed".to_owned()).unwrap();
        self.0.upsert(&instance).await.unwrap();
        Ok(())
    }
}

#[tokio::test]
async fn should_keep_an_edit_made_during_the_install_when_the_install_finishes() {
    let temp = TempDir::new().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        temp.path().join("settings"),
        temp.path().to_path_buf(),
    ));
    let instance = InstanceBuilder::new(
        InstanceId::try_from("inst").unwrap(),
        "inst".to_owned(),
        "1.20.1".to_owned(),
        ModLoader::Vanilla,
    )
    .build();
    let id = instance.id().clone();
    let storage = Arc::new(MemStorage {
        instances: Mutex::new(vec![instance]),
    });
    let use_case = InstallInstanceUseCase::new(
        storage.clone(),
        Arc::new(RenameDuringInstall(storage.clone())),
        Arc::new(NoProgress),
        location_info,
    );

    use_case.execute(id.clone(), false, None).await.unwrap();

    let stored = storage.get(&id).await.unwrap();
    assert_eq!(stored.name(), "Renamed");
    assert_eq!(stored.install_stage, InstanceInstallStage::Installed);
}
