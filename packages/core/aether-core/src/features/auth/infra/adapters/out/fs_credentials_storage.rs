use std::path::Path;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    features::auth::{AuthApplicationError, AuthDomainError, Credentials, CredentialsStorage},
    shared::{JsonEntityStore, UpdateAction},
};

pub struct FsCredentialsStorage {
    store: JsonEntityStore<Credentials>,
}

impl FsCredentialsStorage {
    pub fn new(settings_dir: &Path) -> Self {
        Self {
            store: JsonEntityStore::new(settings_dir.join("credentials.json")),
        }
    }
}

#[async_trait]
impl CredentialsStorage for FsCredentialsStorage {
    async fn list(&self) -> Result<Vec<Credentials>, AuthApplicationError> {
        Ok(self.store.read_all().await?)
    }

    async fn get(&self, id: Uuid) -> Result<Credentials, AuthApplicationError> {
        let list = self.store.read_all().await?;

        list.into_iter()
            .find(|x| x.id() == id)
            .ok_or(AuthApplicationError::Domain(
                AuthDomainError::CredentialsNotFound { id },
            ))
    }

    async fn upsert(&self, credentials: Credentials) -> Result<Credentials, AuthApplicationError> {
        Ok(self
            .store
            .update(|list| {
                if let Some(existing) = list.iter_mut().find(|c| c.id() == credentials.id()) {
                    if existing == &credentials {
                        return UpdateAction::NoChanges(credentials);
                    }
                    *existing = credentials.clone();
                } else {
                    list.push(credentials.clone());
                }
                UpdateAction::Save(credentials)
            })
            .await?)
    }

    async fn upsert_all(
        &self,
        credentials_list: Vec<Credentials>,
    ) -> Result<(), AuthApplicationError> {
        self.store
            .update(|current| {
                let mut changed = false;

                for new_item in credentials_list {
                    if let Some(existing) = current.iter_mut().find(|c| c.id() == new_item.id()) {
                        if existing != &new_item {
                            *existing = new_item;
                            changed = true;
                        }
                    } else {
                        current.push(new_item);
                        changed = true;
                    }
                }

                if changed {
                    UpdateAction::Save(())
                } else {
                    UpdateAction::NoChanges(())
                }
            })
            .await
            .map_err(AuthApplicationError::from)
    }

    async fn remove(&self, id: Uuid) -> Result<(), AuthApplicationError> {
        let found = self
            .store
            .update(|list| {
                let prev_len = list.len();
                list.retain(|c| c.id() != id);

                if list.len() < prev_len {
                    UpdateAction::Save(true)
                } else {
                    UpdateAction::NoChanges(false)
                }
            })
            .await
            .map_err(AuthApplicationError::from)?;

        if !found {
            return Err(AuthApplicationError::Domain(
                AuthDomainError::CredentialsNotFound { id },
            ));
        }
        Ok(())
    }

    async fn clear(&self) -> Result<(), AuthApplicationError> {
        Ok(self.store.write_all(&[]).await?)
    }

    async fn find_active(&self) -> Result<Option<Credentials>, AuthApplicationError> {
        let list = self.store.read_all().await?;
        Ok(list.into_iter().find(|x| x.is_active()))
    }
}
