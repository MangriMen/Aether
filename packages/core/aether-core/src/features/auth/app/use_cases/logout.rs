use std::sync::Arc;

use uuid::Uuid;

use crate::features::auth::{
    ActiveAccountHelper, AuthApplicationError, AuthDomainError, CredentialsStorage,
};

pub struct LogoutUseCase<CS: CredentialsStorage> {
    credentials_storage: Arc<CS>,
}

impl<CS: CredentialsStorage> LogoutUseCase<CS> {
    pub fn new(credentials_storage: Arc<CS>) -> Self {
        Self {
            credentials_storage,
        }
    }

    pub async fn execute(&self, account_id: Uuid) -> Result<(), AuthApplicationError> {
        self.credentials_storage.remove(account_id).await?;

        match ActiveAccountHelper::ensure_active(self.credentials_storage.as_ref()).await {
            Ok(_) => Ok(()),
            Err(AuthApplicationError::Domain(AuthDomainError::NoActiveCredentials)) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
