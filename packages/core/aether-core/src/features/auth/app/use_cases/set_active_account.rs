use std::sync::Arc;

use uuid::Uuid;

use crate::features::auth::{ActiveAccountHelper, AuthApplicationError, CredentialsStorage};

use super::super::AccountData;

pub struct SetActiveAccountUseCase<CS: CredentialsStorage> {
    credentials_storage: Arc<CS>,
}

impl<CS: CredentialsStorage> SetActiveAccountUseCase<CS> {
    pub fn new(credentials_storage: Arc<CS>) -> Self {
        Self {
            credentials_storage,
        }
    }

    pub async fn execute(&self, account_id: Uuid) -> Result<AccountData, AuthApplicationError> {
        let account =
            ActiveAccountHelper::set_active(self.credentials_storage.as_ref(), account_id).await?;

        Ok(AccountData::from(account))
    }
}
