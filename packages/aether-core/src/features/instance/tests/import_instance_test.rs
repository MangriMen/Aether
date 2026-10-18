//! `ImportInstanceUseCase` finishing the instances a plugin created for a pack.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::{
    features::{
        instance::{
            CapabilityMetadata, ImportInstance, ImportInstanceUseCase, Importer,
            ImporterCapabilityMetadata, Instance, InstanceBuilder, InstanceError, InstanceId,
            InstanceInstallStage, InstanceStorage,
        },
        minecraft::ModLoader,
    },
    shared::capability::{CapabilityRegistry, infra::MemoryCapabilityRegistry},
};

const PLUGIN: &str = "packwiz";
const IMPORTER: &str = "packwiz";

#[derive(Default)]
struct MemStorage {
    instances: Mutex<Vec<Instance>>,
}

impl MemStorage {
    fn stage(&self, id: &str) -> InstanceInstallStage {
        self.instances
            .lock()
            .unwrap()
            .iter()
            .find(|instance| instance.id().as_str() == id)
            .unwrap()
            .install_stage
    }
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
            .find(|instance| instance.id() == id)
            .cloned()
            .ok_or_else(|| InstanceError::NotFound {
                instance_id: id.to_string(),
            })
    }

    async fn upsert(&self, instance: &Instance) -> Result<(), InstanceError> {
        let mut instances = self.instances.lock().unwrap();
        instances.retain(|existing| existing.id() != instance.id());
        instances.push(instance.clone());
        Ok(())
    }

    async fn remove(&self, id: &InstanceId) -> Result<(), InstanceError> {
        self.instances
            .lock()
            .unwrap()
            .retain(|instance| instance.id() != id);
        Ok(())
    }
}

/// An importer that has created `pack` for a pack and answers with it — or fails after creating
/// it.
struct PackImporter {
    metadata: ImporterCapabilityMetadata,
    fail: bool,
}

#[async_trait]
impl Importer for PackImporter {
    fn metadata(&self) -> &ImporterCapabilityMetadata {
        &self.metadata
    }

    async fn import(&self, _: &str) -> Result<Vec<String>, InstanceError> {
        if self.fail {
            Err(InstanceError::ImportFailed {
                plugin_id: PLUGIN.into(),
                capability_id: IMPORTER.into(),
                reason: "the importer failed".into(),
            })
        } else {
            Ok(vec!["pack".into()])
        }
    }
}

async fn import(fail: bool) -> (Result<(), InstanceError>, Arc<MemStorage>) {
    let storage = Arc::new(MemStorage::default());
    let mut instance = InstanceBuilder::new(
        InstanceId::try_from("pack").unwrap(),
        "Pack".into(),
        "1.21.1".into(),
        ModLoader::Fabric,
    )
    .build();
    instance.install_stage = InstanceInstallStage::PackInstalling;
    storage.upsert(&instance).await.unwrap();

    let registry = Arc::new(MemoryCapabilityRegistry::<Arc<dyn Importer>>::default());
    registry
        .add(
            PLUGIN.into(),
            IMPORTER.into(),
            Arc::new(PackImporter {
                metadata: ImporterCapabilityMetadata {
                    base: CapabilityMetadata {
                        id: IMPORTER.into(),
                        name: "Packwiz".into(),
                        description: None,
                        icon: None,
                    },
                    field_label: None,
                    supported_extensions: Vec::new(),
                },
                fail,
            }),
        )
        .await
        .unwrap();

    let result = ImportInstanceUseCase::new(registry, storage.clone())
        .execute(ImportInstance {
            plugin_id: PLUGIN.into(),
            importer_id: IMPORTER.into(),
            path: "pack.toml".into(),
        })
        .await;

    (result, storage)
}

/// The importer's call is over: the pack is in, and its instance can be launched.
#[tokio::test]
async fn should_finish_the_pack_instances_when_an_import_succeeds() {
    let (result, storage) = import(false).await;

    result.unwrap();
    assert_eq!(storage.stage("pack"), InstanceInstallStage::Installed);
}

/// A failed import may have left the pack half deployed: its instance is not launchable.
#[tokio::test]
async fn should_leave_the_pack_instances_unfinished_when_an_import_fails() {
    let (result, storage) = import(true).await;

    result.unwrap_err();
    assert_eq!(storage.stage("pack"), InstanceInstallStage::PackInstalling);
}
