use crate::{
    features::java::{Java, JavaDomainError, JavaStorage},
    shared::{IoError, JsonEntityStore, UpdateAction},
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
    async fn list(&self) -> Result<Vec<Java>, JavaDomainError> {
        Ok(self.store.read_all().await?)
    }

    async fn get(&self, version: u32) -> Result<Option<Java>, JavaDomainError> {
        let list = self.store.read_all().await?;
        Ok(list.into_iter().find(|x| x.major_version() == version))
    }

    async fn upsert(&self, java: Java) -> Result<Java, JavaDomainError> {
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

    async fn remove(&self, major_version: u32) -> Result<(), JavaDomainError> {
        Ok(self
            .store
            .update(|list| {
                if let Some(index) = list.iter().position(|c| c.major_version() == major_version) {
                    list.remove(index);
                    return UpdateAction::Save(());
                }
                UpdateAction::NoChanges(())
            })
            .await?)
    }
}

impl From<IoError> for JavaDomainError {
    fn from(value: IoError) -> Self {
        JavaDomainError::Storage(value.to_string())
    }
}
