use std::sync::Arc;

use async_trait::async_trait;
use md5::{Digest, Md5};
use uuid::Uuid;

use crate::features::auth::{
    app::{
        Account, ActiveAccountHelper, AuthApplicationError, CredentialsStorage,
        ports::CreateOfflineAccountUseCasePort,
    },
    domain::{AccountType, AuthDomainError, Credential, Username},
};

/// The UUID the game derives for an offline player: a name-based (version 3) UUID of
/// `OfflinePlayer:<name>`, the same as the vanilla server and the other launchers get.
/// The name is used as typed, case-sensitive.
fn offline_uuid(username: &Username) -> Uuid {
    let hash = Md5::digest(format!("OfflinePlayer:{}", username.as_str()).as_bytes());

    uuid::Builder::from_md5_bytes(hash.into()).into_uuid()
}

pub struct CreateOfflineAccountUseCase {
    credentials_storage: Arc<dyn CredentialsStorage>,
}

impl CreateOfflineAccountUseCase {
    pub fn new(credentials_storage: Arc<dyn CredentialsStorage>) -> Self {
        Self {
            credentials_storage,
        }
    }
}

#[async_trait]
impl CreateOfflineAccountUseCasePort for CreateOfflineAccountUseCase {
    async fn execute(&self, username: String) -> Result<Account, AuthApplicationError> {
        let username = Username::parse(&username)?;

        // Minecraft names are case-insensitive, and the id is the key of the account, so a
        // second account of the same name would overwrite the first one. Names that differ
        // only in case get different ids; they are refused too, as the same player.
        let taken = self.credentials_storage.list().await?.iter().any(|c| {
            c.account_type() == AccountType::Offline
                && c.username()
                    .as_str()
                    .eq_ignore_ascii_case(username.as_str())
        });

        if taken {
            return Err(AuthDomainError::AccountAlreadyExists.into());
        }

        let credentials = Credential::new_offline(offline_uuid(&username), username);

        self.credentials_storage.upsert(credentials).await?;

        let account = ActiveAccountHelper::ensure_active(self.credentials_storage.as_ref()).await?;

        Ok(Account::from(account))
    }
}
