use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tempfile::TempDir;

use crate::features::{
    events::{EventError, ProgressBarId, ProgressEventType, ProgressService},
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

    let result = use_case.execute(id.to_string(), false).await;
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

#[tokio::test]
async fn should_reset_to_not_installed_when_the_install_fails() {
    let (instance, ok) = run_install(
        Some(pack_info()),
        InstanceInstallStage::PackInstalling,
        true,
    )
    .await;
    assert!(!ok);
    assert_eq!(instance.install_stage, InstanceInstallStage::NotInstalled);
}
