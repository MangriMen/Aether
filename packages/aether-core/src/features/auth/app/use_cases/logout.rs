use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::auth::{
    app::{
        ActiveAccountHelper, AuthApplicationError, CredentialsStorage, ports::LogoutUseCasePort,
    },
    domain::AuthDomainError,
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

        // The account is already gone, so having nothing left to promote must
        // not turn a successful logout into an error.
        match ActiveAccountHelper::ensure_active(storage).await {
            Ok(_)
            | Err(AuthApplicationError::Domain(
                AuthDomainError::NoActiveCredentials | AuthDomainError::TokenExpired,
            )) => Ok(()),
            Err(error) => Err(error),
        }
    }
}
