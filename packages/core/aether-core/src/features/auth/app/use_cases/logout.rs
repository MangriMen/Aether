use std::sync::Arc;

use uuid::Uuid;

use crate::features::auth::{
    app::{ActiveAccountHelper, AuthApplicationError, CredentialsStorage},
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

    pub async fn execute(&self, account_id: Uuid) -> Result<(), AuthApplicationError> {
        self.credentials_storage.remove(account_id).await?;

        match ActiveAccountHelper::ensure_active(self.credentials_storage.as_ref()).await {
            Ok(_) | Err(AuthApplicationError::Domain(AuthDomainError::NoActiveCredentials)) => {
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
