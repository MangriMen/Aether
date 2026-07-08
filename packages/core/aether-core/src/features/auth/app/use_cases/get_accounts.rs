use std::sync::Arc;

use async_trait::async_trait;

use crate::features::auth::app::{
    Account, AuthApplicationError, CredentialsStorage, ports::GetAccountsUseCasePort,
};

pub struct GetAccountsUseCase {
    credentials_storage: Arc<dyn CredentialsStorage>,
}

impl GetAccountsUseCase {
    pub fn new(credentials_storage: Arc<dyn CredentialsStorage>) -> Self {
        Self {
            credentials_storage,
        }
    }
}

#[async_trait]
impl GetAccountsUseCasePort for GetAccountsUseCase {
    async fn execute(&self) -> Result<Vec<Account>, AuthApplicationError> {
        let credentials = self.credentials_storage.list().await?;
        Ok(credentials.iter().map(Account::from).collect())
    }
}
