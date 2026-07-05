use std::sync::Arc;

use uuid::Uuid;

use crate::features::auth::{
    app::{Account, ActiveAccountHelper, AuthApplicationError, CredentialsStorage},
    domain::{Credential, Username},
};

pub struct CreateOfflineAccountUseCase {
    credentials_storage: Arc<dyn CredentialsStorage>,
}

impl CreateOfflineAccountUseCase {
    pub fn new(credentials_storage: Arc<dyn CredentialsStorage>) -> Self {
        Self {
            credentials_storage,
        }
    }

    pub async fn execute(&self, username: String) -> Result<Account, AuthApplicationError> {
        let username = Username::parse(&username)?;
        let credentials = Credential::new_offline(Uuid::new_v4(), username);

        self.credentials_storage.upsert(credentials).await?;

        let account = ActiveAccountHelper::ensure_active(self.credentials_storage.as_ref()).await?;

        Ok(Account::from(account))
    }
}
