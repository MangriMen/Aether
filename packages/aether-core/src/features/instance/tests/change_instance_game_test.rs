use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::features::{
    instance::{
        ChangeInstanceGameUseCase, Instance, InstanceBuilder, InstanceError, InstanceGame,
        InstanceId, InstanceInstallService, InstanceInstallStage, InstanceStorage,
    },
    minecraft::{
        LoaderVersionPreference, LoaderVersionService, MinecraftDomainError, ModLoader, modded,
    },
};

const INSTANCE_ID: &str = "instance";

// ─── Mocks ─────────────────────────────────────────────────────

#[derive(Default)]
struct MemInstanceStorage {
    instance: Mutex<Option<Instance>>,
}

impl MemInstanceStorage {
    fn instance(&self) -> Instance {
        self.instance.lock().unwrap().clone().unwrap()
    }
}

#[async_trait]
impl InstanceStorage for MemInstanceStorage {
    async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
        Ok(self.instance.lock().unwrap().iter().cloned().collect())
    }

    async fn get(&self, id: &InstanceId) -> Result<Instance, InstanceError> {
        self.instance
            .lock()
            .unwrap()
            .clone()
            .filter(|instance| instance.id() == id)
            .ok_or_else(|| InstanceError::NotFound {
                instance_id: id.to_string(),
            })
    }

    async fn upsert(&self, instance: &Instance) -> Result<(), InstanceError> {
        *self.instance.lock().unwrap() = Some(instance.clone());
        Ok(())
    }

    async fn remove(&self, _: &InstanceId) -> Result<(), InstanceError> {
        unimplemented!()
    }
}

/// Answers `try_get_default` with `0.16.9`.
struct DefaultLoaderVersion;

#[async_trait]
impl LoaderVersionService for DefaultLoaderVersion {
    async fn resolve(
        &self,
        _: &str,
        _: &ModLoader,
        _: Option<&LoaderVersionPreference>,
    ) -> Result<Option<modded::LoaderVersion>, MinecraftDomainError> {
        Ok(None)
    }

    async fn try_get_default(
        &self,
        _: &str,
        _: &ModLoader,
    ) -> Result<Option<LoaderVersionPreference>, MinecraftDomainError> {
        Ok(Some(LoaderVersionPreference::Exact("0.16.9".into())))
    }
}

/// Records the game the instance had when installing was asked for, and moves its stage as the
/// real service does for an installed instance: `Installed` on success, `NotInstalled` on failure.
struct RecordingInstall {
    storage: Arc<MemInstanceStorage>,
    fail: bool,
    installed: Mutex<Vec<(String, ModLoader, Option<LoaderVersionPreference>)>>,
}

#[async_trait]
impl InstanceInstallService for RecordingInstall {
    async fn execute(&self, instance_id: String, _force: bool) -> Result<(), InstanceError> {
        let mut instance = self
            .storage
            .get(&InstanceId::try_from(instance_id.as_str()).unwrap())
            .await?;
        self.installed.lock().unwrap().push((
            instance.game_version.clone(),
            instance.loader,
            instance.loader_version.clone(),
        ));

        instance.install_stage = if self.fail {
            InstanceInstallStage::NotInstalled
        } else {
            InstanceInstallStage::Installed
        };
        self.storage.upsert(&instance).await?;

        if self.fail {
            Err(InstanceError::Storage("install failed".into()))
        } else {
            Ok(())
        }
    }
}

// ─── Fixture ───────────────────────────────────────────────────

struct Fixture {
    storage: Arc<MemInstanceStorage>,
    install: Arc<RecordingInstall>,
    use_case: ChangeInstanceGameUseCase,
}

impl Fixture {
    /// An installed Fabric 1.21.1 instance, loader 0.16.5.
    fn new(fail_install: bool) -> Self {
        let storage = Arc::new(MemInstanceStorage::default());
        let mut instance = InstanceBuilder::new(
            InstanceId::try_from(INSTANCE_ID).unwrap(),
            "Instance".into(),
            "1.21.1".into(),
            ModLoader::Fabric,
        )
        .with_loader_version(Some(LoaderVersionPreference::Exact("0.16.5".into())))
        .build();
        instance.install_stage = InstanceInstallStage::Installed;
        *storage.instance.lock().unwrap() = Some(instance);

        let install = Arc::new(RecordingInstall {
            storage: storage.clone(),
            fail: fail_install,
            installed: Mutex::default(),
        });

        Self {
            use_case: ChangeInstanceGameUseCase::new(
                storage.clone(),
                Arc::new(DefaultLoaderVersion),
                install.clone(),
            ),
            storage,
            install,
        }
    }

    async fn change(&self, game: InstanceGame) -> Result<(), InstanceError> {
        self.use_case.execute(INSTANCE_ID.into(), game).await
    }

    fn installed(&self) -> Vec<(String, ModLoader, Option<LoaderVersionPreference>)> {
        self.install.installed.lock().unwrap().clone()
    }
}

fn game(game_version: &str, mod_loader: ModLoader, loader_version: Option<&str>) -> InstanceGame {
    InstanceGame {
        game_version: game_version.into(),
        mod_loader,
        loader_version: loader_version
            .map(|version| LoaderVersionPreference::Exact(version.into())),
    }
}

// ─── Tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn should_do_nothing_when_the_instance_already_runs_the_game() {
    let fixture = Fixture::new(false);
    let before = fixture.storage.instance();

    fixture
        .change(game("1.21.1", ModLoader::Fabric, Some("0.16.5")))
        .await
        .unwrap();
    // A pack that does not pin its loader version keeps the instance's.
    fixture
        .change(game("1.21.1", ModLoader::Fabric, None))
        .await
        .unwrap();

    assert!(fixture.installed().is_empty());
    assert_eq!(fixture.storage.instance().modified, before.modified);
}

#[tokio::test]
async fn should_move_the_instance_to_the_new_game_and_install_it_when_the_game_changes() {
    let fixture = Fixture::new(false);

    fixture
        .change(game("1.21.4", ModLoader::NeoForge, Some("21.4.1")))
        .await
        .unwrap();

    let expected = (
        "1.21.4".to_owned(),
        ModLoader::NeoForge,
        Some(LoaderVersionPreference::Exact("21.4.1".into())),
    );
    // The new game is what gets installed, and what the instance keeps.
    assert_eq!(fixture.installed(), std::slice::from_ref(&expected));
    let instance = fixture.storage.instance();
    assert_eq!(
        (
            instance.game_version,
            instance.loader,
            instance.loader_version
        ),
        expected
    );
}

#[tokio::test]
async fn should_install_a_new_loader_version_when_the_game_stays_the_same() {
    let fixture = Fixture::new(false);

    fixture
        .change(game("1.21.1", ModLoader::Fabric, Some("0.16.9")))
        .await
        .unwrap();

    assert_eq!(fixture.installed().len(), 1);
    assert_eq!(
        fixture.storage.instance().loader_version,
        Some(LoaderVersionPreference::Exact("0.16.9".into()))
    );
}

#[tokio::test]
async fn should_take_the_default_loader_version_when_the_new_game_pins_none() {
    let fixture = Fixture::new(false);

    fixture
        .change(game("1.21.4", ModLoader::Fabric, None))
        .await
        .unwrap();

    assert_eq!(
        fixture.storage.instance().loader_version,
        Some(LoaderVersionPreference::Exact("0.16.9".into()))
    );
}

/// The old game is still installed, so putting it back leaves an instance that runs as before.
#[tokio::test]
async fn should_put_the_old_game_back_when_installing_fails() {
    let fixture = Fixture::new(true);
    let before = fixture.storage.instance();

    let result = fixture
        .change(game("1.21.4", ModLoader::NeoForge, Some("21.4.1")))
        .await;

    assert!(result.is_err());
    assert_eq!(fixture.installed().len(), 1);
    let instance = fixture.storage.instance();
    assert_eq!(instance.game_version, before.game_version);
    assert_eq!(instance.loader, before.loader);
    assert_eq!(instance.loader_version, before.loader_version);
    assert_eq!(instance.install_stage, InstanceInstallStage::Installed);
}
