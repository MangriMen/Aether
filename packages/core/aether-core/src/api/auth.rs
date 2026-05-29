use uuid::Uuid;

use crate::{
    core::LazyLocator,
    features::auth::{
        AccountData, CreateOfflineAccountUseCase, GetAccountsUseCase, LogoutUseCase,
        SetActiveAccountUseCase,
    },
};

#[tracing::instrument]
pub async fn create_offline_account(username: String) -> crate::Result<AccountData> {
    let locator = LazyLocator::get().await?;

    Ok(
        CreateOfflineAccountUseCase::new(locator.get_credentials_storage().await)
            .execute(username)
            .await?,
    )
}

pub async fn list_accounts() -> crate::Result<Vec<AccountData>> {
    let locator = LazyLocator::get().await?;

    Ok(
        GetAccountsUseCase::new(locator.get_credentials_storage().await)
            .execute()
            .await?,
    )
}

#[tracing::instrument]
pub async fn change_account(account_id: Uuid) -> crate::Result<AccountData> {
    let locator = LazyLocator::get().await?;

    Ok(
        SetActiveAccountUseCase::new(locator.get_credentials_storage().await)
            .execute(account_id)
            .await?,
    )
}

#[tracing::instrument]
pub async fn logout(account_id: Uuid) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    Ok(LogoutUseCase::new(locator.get_credentials_storage().await)
        .execute(account_id)
        .await?)
}
