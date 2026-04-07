use std::sync::Arc;
use uuid::Uuid;

use crate::features::auth::{CredentialsStorage, LogoutUseCase};

use super::fixtures::{CredentialsBuilder, MockCredentialsStorage};

#[tokio::test]
async fn test_logout_sets_new_active() {
    let storage = Arc::new(MockCredentialsStorage::new());

    let cred1_id = Uuid::new_v4();
    let cred2_id = Uuid::new_v4();

    storage
        .upsert(
            CredentialsBuilder::new("User1")
                .with_id(cred1_id)
                .with_active(true)
                .build(),
        )
        .await
        .unwrap();

    storage
        .upsert(CredentialsBuilder::new("User2").with_id(cred2_id).build())
        .await
        .unwrap();

    let use_case = LogoutUseCase::new(storage.clone());
    use_case.execute(cred1_id).await.unwrap();

    let active = storage.find_active().await.unwrap().unwrap();
    assert_eq!(active.id(), cred2_id);
}

#[tokio::test]
async fn test_logout_last_account() {
    let storage = Arc::new(MockCredentialsStorage::new());

    let cred_id = Uuid::new_v4();
    storage
        .upsert(
            CredentialsBuilder::new("OnlyUser")
                .with_id(cred_id)
                .with_active(true)
                .build(),
        )
        .await
        .unwrap();

    let use_case = LogoutUseCase::new(storage.clone());

    let result = use_case.execute(cred_id).await;
    assert!(
        result.is_ok(),
        "Logout should succeed even for the last account"
    );

    let list = storage.list().await.unwrap();
    assert!(list.is_empty());

    // Account should be removed since logout succeeded in removing it
    let result = storage.get(cred_id).await;
    assert!(result.is_err());

    let active = storage.find_active().await.unwrap();
    assert!(active.is_none());
}

#[tokio::test]
async fn test_logout_nonexistent_account() {
    let storage = Arc::new(MockCredentialsStorage::new());

    storage
        .upsert(CredentialsBuilder::new("User1").build())
        .await
        .unwrap();

    let use_case = LogoutUseCase::new(storage.clone());

    let nonexistent_id = Uuid::new_v4();
    let result = use_case.execute(nonexistent_id).await;
    assert!(result.is_err());
}
