use std::sync::Arc;

use crate::features::auth::app::{
    CreateOfflineAccountUseCase, CreateOfflineAccountUseCasePort, CredentialsStorage,
};
use crate::features::auth::{app::AuthApplicationError, domain::AuthDomainError};

use super::fixtures::{CredentialsBuilder, MockCredentialsStorage};

#[tokio::test]
async fn should_set_the_first_account_active_when_none_is_active() {
    let storage = Arc::new(MockCredentialsStorage::new());

    let use_case = CreateOfflineAccountUseCase::new(storage.clone());

    let account = use_case.execute("TestUser".to_string()).await.unwrap();

    assert!(account.active);
    assert_eq!(account.username.as_str(), "TestUser");

    let active = storage.find_active().await.unwrap().unwrap();
    assert_eq!(active.id(), account.id);
}

#[tokio::test]
async fn should_activate_the_new_account_when_an_expired_account_comes_first() {
    let storage = Arc::new(MockCredentialsStorage::new());

    storage
        .upsert(
            CredentialsBuilder::new("Expired")
                .with_expired_token()
                .build(),
        )
        .await
        .unwrap();

    let use_case = CreateOfflineAccountUseCase::new(storage.clone());

    let account = use_case.execute("TestUser".to_string()).await.unwrap();

    assert!(account.active);
    assert_eq!(
        storage.find_active().await.unwrap().unwrap().id(),
        account.id
    );
}

#[tokio::test]
async fn should_fail_to_create_offline_account_when_username_is_invalid() {
    let storage = Arc::new(MockCredentialsStorage::new());
    let use_case = CreateOfflineAccountUseCase::new(storage.clone());

    let too_short = use_case.execute("ab".to_string()).await;
    let too_long = use_case
        .execute("verylongusernamethatexceedslimit".to_string())
        .await;
    let bad_chars = use_case.execute("user@name".to_string()).await;

    assert!(matches!(
        too_short,
        Err(AuthApplicationError::Domain(
            AuthDomainError::InvalidUsernameLength { .. }
        ))
    ));
    assert!(matches!(
        too_long,
        Err(AuthApplicationError::Domain(
            AuthDomainError::InvalidUsernameLength { .. }
        ))
    ));
    assert!(matches!(
        bad_chars,
        Err(AuthApplicationError::Domain(
            AuthDomainError::InvalidUsernameChars
        ))
    ));
}
