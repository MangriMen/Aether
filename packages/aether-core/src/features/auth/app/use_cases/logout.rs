use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::auth::{
    app::{
        ActiveAccountHelper, AuthApplicationError, CredentialsStorage, ports::LogoutUseCasePort,
    },
    domain::{AuthDomainError, Credential},
};

pub struct LogoutUseCase {
    credentials_storage: Arc<dyn CredentialsStorage>,
}

impl LogoutUseCase {
    pub fn new(credentials_storage: Arc<dyn CredentialsStorage>) -> Self {
        Self {
            credentials_storage,
        }
    }
}

#[async_trait]
impl LogoutUseCasePort for LogoutUseCase {
    async fn execute(&self, account_id: Uuid) -> Result<(), AuthApplicationError> {
        let storage = self.credentials_storage.as_ref();
        storage.remove(account_id).await?;

        let remaining = storage.list().await?;
        if remaining.iter().any(Credential::is_active) {
            return Ok(());
        }

        // The account is already gone, so an expired candidate must not fail the
        // logout: skip it and promote the first account that can be activated.
        for candidate in remaining {
            match ActiveAccountHelper::set_active(storage, candidate.id()).await {
                Err(AuthApplicationError::Domain(AuthDomainError::TokenExpired)) => {}
                result => return result.map(|_| ()),
            }
        }

        Ok(())
    }
}
