use std::path::{Path, PathBuf};

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    features::auth::{
        app::{AuthApplicationError, CredentialsStorage},
        domain::{AuthDomainError, Credential},
        infra::fs::CredentialV1,
    },
    shared::{
        io::domain::IoError,
        json_store::{domain::UpdateAction, infra::JsonEntityStore},
    },
};

pub struct FsCredentialsStorage {
    store: JsonEntityStore<CredentialV1>,
    path: PathBuf,
}

impl FsCredentialsStorage {
    pub fn new(settings_dir: &Path) -> Self {
        let path = settings_dir.join("credentials.json");
        Self {
            store: JsonEntityStore::new(path.clone()),
            path,
        }
    }

    pub fn get_file_path(&self) -> PathBuf {
        self.path.clone()
    }
}

#[async_trait]
impl CredentialsStorage for FsCredentialsStorage {
    async fn list(&self) -> Result<Vec<Credential>, AuthApplicationError> {
        Ok(self
            .store
            .read_all()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    async fn get(&self, id: Uuid) -> Result<Credential, AuthApplicationError> {
        let list = self.store.read_all().await?;

        list.into_iter()
            .find(|x| x.id == id)
            .map(Into::into)
            .ok_or(AuthApplicationError::Domain(
                AuthDomainError::CredentialsNotFound { id },
            ))
    }

    async fn upsert(&self, credentials: Credential) -> Result<Credential, AuthApplicationError> {
        let dto: CredentialV1 = credentials.into();

        let updated_dto = self
            .store
            .update(|list| {
                if let Some(existing) = list.iter_mut().find(|c| c.id == dto.id) {
                    if existing == &dto {
                        return UpdateAction::NoChanges(dto);
                    }
                    *existing = dto.clone();
                } else {
                    list.push(dto.clone());
                }
                UpdateAction::Save(dto)
            })
            .await?;

        Ok(updated_dto.into())
    }

    async fn upsert_all(
        &self,
        credentials_list: Vec<Credential>,
    ) -> Result<(), AuthApplicationError> {
        let dtos: Vec<CredentialV1> = credentials_list.into_iter().map(Into::into).collect();

        self.store
            .update(|current| {
                let mut changed = false;

                for new_dto in dtos {
                    if let Some(existing) = current.iter_mut().find(|c| c.id == new_dto.id) {
                        if existing != &new_dto {
                            *existing = new_dto;
                            changed = true;
                        }
                    } else {
                        current.push(new_dto);
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
                list.retain(|c| c.id != id);

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

    async fn find_active(&self) -> Result<Option<Credential>, AuthApplicationError> {
        let list = self.store.read_all().await?;
        Ok(list.into_iter().find(|c| c.is_active).map(Into::into))
    }
}

impl From<IoError> for AuthApplicationError {
    fn from(err: IoError) -> Self {
        AuthApplicationError::Storage(err.to_string())
    }
}
