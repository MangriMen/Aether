use uuid::Uuid;

use crate::{
    core::domain::LazyLocator,
    features::auth::{
        AccountData, CreateOfflineAccountUseCase, GetAccountsUseCase, LogoutUseCase,
        SetActiveAccountUseCase,
    },
};

#[tracing::instrument]
pub async fn create_offline_account(username: String) -> crate::Result<AccountData> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        CreateOfflineAccountUseCase::new(lazy_locator.get_credentials_storage().await)
            .execute(username)
            .await?,
    )
}

pub async fn get_accounts() -> crate::Result<Vec<AccountData>> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        GetAccountsUseCase::new(lazy_locator.get_credentials_storage().await)
            .execute()
            .await?,
    )
}

#[tracing::instrument]
pub async fn change_account(account_id: Uuid) -> crate::Result<AccountData> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        SetActiveAccountUseCase::new(lazy_locator.get_credentials_storage().await)
            .execute(account_id)
            .await?,
    )
}

#[tracing::instrument]
pub async fn logout(account_id: Uuid) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        LogoutUseCase::new(lazy_locator.get_credentials_storage().await)
            .execute(account_id)
            .await?,
    )
}
