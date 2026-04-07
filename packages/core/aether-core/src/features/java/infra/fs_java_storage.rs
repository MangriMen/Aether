use crate::{
    features::java::{Java, JavaStorage, JavaStorageError},
    shared::{JsonEntityStore, UpdateAction},
};
use async_trait::async_trait;
use std::path::Path;

pub struct FsJavaStorage {
    store: JsonEntityStore<Java>,
}

impl FsJavaStorage {
    pub fn new(java_dir: &Path) -> Self {
        Self {
            store: JsonEntityStore::new(java_dir.join("java_versions.json")),
        }
    }
}

#[async_trait]
impl JavaStorage for FsJavaStorage {
    async fn list(&self) -> Result<Vec<Java>, JavaStorageError> {
        Ok(self.store.read_all().await?)
    }

    async fn get(&self, version: u32) -> Result<Option<Java>, JavaStorageError> {
        let list = self.store.read_all().await?;
        Ok(list.into_iter().find(|x| x.major_version() == version))
    }

    async fn upsert(&self, java: Java) -> Result<Java, JavaStorageError> {
        Ok(self
            .store
            .update(|list| {
                if let Some(existing) = list
                    .iter_mut()
                    .find(|c| c.major_version() == java.major_version())
                {
                    if existing == &java {
                        return UpdateAction::NoChanges(java);
                    }
                    *existing = java.clone();
                } else {
                    list.push(java.clone());
                }
                UpdateAction::Save(java)
            })
            .await?)
    }
}
