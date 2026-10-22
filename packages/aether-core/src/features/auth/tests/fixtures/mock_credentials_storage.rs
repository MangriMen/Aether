use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::features::auth::app::{AuthApplicationError, CredentialsStorage};
use crate::features::auth::domain::{AuthDomainError, Credential};

/// Keeps insertion order so tests can rely on which account comes first.
#[derive(Default)]
pub struct MockCredentialsStorage {
    store: Arc<Mutex<Vec<Credential>>>,
}

impl MockCredentialsStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl CredentialsStorage for MockCredentialsStorage {
    async fn list(&self) -> Result<Vec<Credential>, AuthApplicationError> {
        Ok(self.store.lock().unwrap().clone())
    }

    async fn get(&self, id: Uuid) -> Result<Credential, AuthApplicationError> {
        self.store
            .lock()
            .unwrap()
            .iter()
            .find(|c| c.id() == id)
            .cloned()
            .ok_or(AuthApplicationError::Domain(
                AuthDomainError::CredentialsNotFound { id },
            ))
    }

    async fn upsert(&self, credentials: Credential) -> Result<Credential, AuthApplicationError> {
        let mut store = self.store.lock().unwrap();
        match store.iter_mut().find(|c| c.id() == credentials.id()) {
            Some(existing) => *existing = credentials.clone(),
            None => store.push(credentials.clone()),
        }
        Ok(credentials)
    }

    async fn upsert_all(
        &self,
        credentials_list: Vec<Credential>,
    ) -> Result<(), AuthApplicationError> {
        let mut store = self.store.lock().unwrap();
        for credentials in credentials_list {
            match store.iter_mut().find(|c| c.id() == credentials.id()) {
                Some(existing) => *existing = credentials,
                None => store.push(credentials),
            }
        }
        Ok(())
    }

    async fn remove(&self, id: Uuid) -> Result<(), AuthApplicationError> {
        let mut store = self.store.lock().unwrap();
        let Some(position) = store.iter().position(|c| c.id() == id) else {
            return Err(AuthApplicationError::Domain(
                AuthDomainError::CredentialsNotFound { id },
            ));
        };
        store.remove(position);
        Ok(())
    }

    async fn clear(&self) -> Result<(), AuthApplicationError> {
        self.store.lock().unwrap().clear();
        Ok(())
    }

    async fn find_active(&self) -> Result<Option<Credential>, AuthApplicationError> {
        Ok(self
            .store
            .lock()
            .unwrap()
            .iter()
            .find(|c| c.is_active())
            .cloned())
    }
}
