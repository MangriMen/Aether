use std::sync::Arc;

use crate::features::auth::app::{
    CreateOfflineAccountUseCase, CreateOfflineAccountUseCasePort, CredentialsStorage,
};

use super::fixtures::MockCredentialsStorage;

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
async fn should_fail_to_create_offline_account_when_username_is_invalid() {
    let storage = Arc::new(MockCredentialsStorage::new());
    let use_case = CreateOfflineAccountUseCase::new(storage.clone());

    assert!(use_case.execute("ab".to_string()).await.is_err());
    assert!(
        use_case
            .execute("verylongusernamethatexceedslimit".to_string())
            .await
            .is_err()
    );
    assert!(use_case.execute("user@name".to_string()).await.is_err());
}
