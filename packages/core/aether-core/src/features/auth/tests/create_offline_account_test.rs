use std::sync::Arc;

use crate::features::auth::{CreateOfflineAccountUseCase, CredentialsStorage};

use super::fixtures::MockCredentialsStorage;

#[tokio::test]
async fn test_create_offline_account_sets_active_if_none() {
    let storage = Arc::new(MockCredentialsStorage::new());

    let use_case = CreateOfflineAccountUseCase::new(storage.clone());

    let account = use_case.execute("TestUser".to_string()).await.unwrap();

    assert!(account.active);
    assert_eq!(account.username.as_str(), "TestUser");

    let active = storage.find_active().await.unwrap().unwrap();
    assert_eq!(active.id(), account.id);
}

#[tokio::test]
async fn test_create_offline_account_with_invalid_username() {
    let storage = Arc::new(MockCredentialsStorage::new());
    let use_case = CreateOfflineAccountUseCase::new(storage.clone());

    assert!(use_case.execute("ab".to_string()).await.is_err());
    assert!(use_case
        .execute("verylongusernamethatexceedslimit".to_string())
        .await
        .is_err());
    assert!(use_case.execute("user@name".to_string()).await.is_err());
}
