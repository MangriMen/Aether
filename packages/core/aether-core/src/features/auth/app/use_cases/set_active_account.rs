use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::auth::app::{
    ActiveAccountHelper, AuthApplicationError, CredentialsStorage,
    ports::SetActiveAccountUseCasePort,
};

use super::super::Account;

pub struct SetActiveAccountUseCase {
    credentials_storage: Arc<dyn CredentialsStorage>,
}

impl SetActiveAccountUseCase {
    pub fn new(credentials_storage: Arc<dyn CredentialsStorage>) -> Self {
        Self {
            credentials_storage,
        }
    }
}

#[async_trait]
impl SetActiveAccountUseCasePort for SetActiveAccountUseCase {
    async fn execute(&self, account_id: Uuid) -> Result<Account, AuthApplicationError> {
        let account =
            ActiveAccountHelper::set_active(self.credentials_storage.as_ref(), account_id).await?;

        Ok(Account::from(account))
    }
}
