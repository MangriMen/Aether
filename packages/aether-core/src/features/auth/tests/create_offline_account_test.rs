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

#[tokio::test]
async fn should_derive_the_id_from_the_name_like_the_game_does() {
    for (name, expected) in [
        ("Steve", "5627dd98-e6be-3c21-b8a8-e92344183641"),
        ("steve", "53909932-f794-33c0-9329-948045a4c1ce"),
        ("Notch", "b50ad385-829d-3141-a216-7e7d7539ba7f"),
    ] {
        // A fresh storage per name: `steve` is a duplicate of `Steve`.
        let use_case = CreateOfflineAccountUseCase::new(Arc::new(MockCredentialsStorage::new()));
        let account = use_case.execute(name.to_string()).await.unwrap();

        assert_eq!(account.id.to_string(), expected, "{name}");
    }
}

#[tokio::test]
async fn should_refuse_a_second_offline_account_with_the_same_name_ignoring_case() {
    let storage = Arc::new(MockCredentialsStorage::new());
    let use_case = CreateOfflineAccountUseCase::new(storage.clone());

    use_case.execute("Steve".to_string()).await.unwrap();
    let same = use_case.execute("Steve".to_string()).await;
    let other_case = use_case.execute("steve".to_string()).await;

    for result in [same, other_case] {
        assert!(matches!(
            result,
            Err(AuthApplicationError::Domain(
                AuthDomainError::AccountAlreadyExists
            ))
        ));
    }
    assert_eq!(storage.list().await.unwrap().len(), 1);
}

#[tokio::test]
async fn should_refuse_a_name_taken_by_an_account_with_a_random_id() {
    let storage = Arc::new(MockCredentialsStorage::new());

    storage
        .upsert(CredentialsBuilder::new("Steve").build())
        .await
        .unwrap();

    let result = CreateOfflineAccountUseCase::new(storage)
        .execute("Steve".to_string())
        .await;

    assert!(matches!(
        result,
        Err(AuthApplicationError::Domain(
            AuthDomainError::AccountAlreadyExists
        ))
    ));
}

#[tokio::test]
async fn should_not_treat_a_microsoft_account_of_the_same_name_as_a_duplicate() {
    let storage = Arc::new(MockCredentialsStorage::new());

    storage
        .upsert(
            CredentialsBuilder::new("Steve")
                .with_expired_token()
                .build(),
        )
        .await
        .unwrap();

    let result = CreateOfflineAccountUseCase::new(storage)
        .execute("Steve".to_string())
        .await;

    assert!(result.is_ok());
}
