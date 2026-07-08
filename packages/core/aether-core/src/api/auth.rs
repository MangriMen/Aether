use uuid::Uuid;

use crate::{
    core::app::AetherContainer,
    features::auth::{Account, AuthFeature},
};

#[tracing::instrument]
pub async fn create_offline_account(username: String) -> crate::Result<Account> {
    let container = AetherContainer::get();
    Ok(container
        .create_offline_account_use_case()
        .execute(username)
        .await?)
}

pub async fn list_accounts() -> crate::Result<Vec<Account>> {
    let container = AetherContainer::get();
    Ok(container.get_accounts_use_case().execute().await?)
}

#[tracing::instrument]
pub async fn change_account(account_id: Uuid) -> crate::Result<Account> {
    let container = AetherContainer::get();
    Ok(container
        .set_active_account_use_case()
        .execute(account_id)
        .await?)
}

#[tracing::instrument]
pub async fn logout(account_id: Uuid) -> crate::Result<()> {
    let container = AetherContainer::get();
    Ok(container.logout_use_case().execute(account_id).await?)
}
