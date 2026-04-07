use std::sync::Arc;

use uuid::Uuid;

pub struct CreateOfflineAccountUseCase<CS: CredentialsStorage> {
    credentials_storage: Arc<CS>,
}

use crate::features::auth::{
    ActiveAccountHelper, AuthApplicationError, Credentials, CredentialsStorage, Username,
};

use super::super::AccountData;

impl<CS: CredentialsStorage> CreateOfflineAccountUseCase<CS> {
    pub fn new(credentials_storage: Arc<CS>) -> Self {
        Self {
            credentials_storage,
        }
    }

    pub async fn execute(&self, username: String) -> Result<AccountData, AuthApplicationError> {
        let username = Username::parse(&username)?;
        let credentials = Credentials::new_offline(Uuid::new_v4(), username);

        self.credentials_storage.upsert(credentials).await?;

        let account = ActiveAccountHelper::ensure_active(self.credentials_storage.as_ref()).await?;

        Ok(AccountData::from(account))
    }
}
