use std::sync::Arc;

use crate::features::auth::app::{Account, AuthApplicationError, CredentialsStorage};

pub struct GetAccountsUseCase {
    credentials_storage: Arc<dyn CredentialsStorage>,
}

impl GetAccountsUseCase {
    pub fn new(credentials_storage: Arc<dyn CredentialsStorage>) -> Self {
        Self {
            credentials_storage,
        }
    }

    pub async fn execute(&self) -> Result<Vec<Account>, AuthApplicationError> {
        let credentials = self.credentials_storage.list().await?;
        Ok(credentials.iter().map(Account::from).collect())
    }
}
