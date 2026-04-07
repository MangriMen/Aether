use std::sync::Arc;

use crate::features::auth::{AuthApplicationError, CredentialsStorage};

use super::super::AccountData;

pub struct GetAccountsUseCase<CS: CredentialsStorage> {
    credentials_storage: Arc<CS>,
}

impl<CS: CredentialsStorage> GetAccountsUseCase<CS> {
    pub fn new(credentials_storage: Arc<CS>) -> Self {
        Self {
            credentials_storage,
        }
    }

    pub async fn execute(&self) -> Result<Vec<AccountData>, AuthApplicationError> {
        let credentials = self.credentials_storage.list().await?;
        Ok(credentials.iter().map(AccountData::from).collect())
    }
}
