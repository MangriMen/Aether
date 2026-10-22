use std::sync::Arc;
use uuid::Uuid;

use crate::features::auth::app::{CredentialsStorage, LogoutUseCase, LogoutUseCasePort};

use super::fixtures::{CredentialsBuilder, MockCredentialsStorage};

#[tokio::test]
async fn should_activate_another_account_when_the_active_one_logs_out() {
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
async fn should_leave_no_active_account_when_the_last_one_logs_out() {
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
async fn should_fail_to_log_out_when_the_account_does_not_exist() {
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

#[tokio::test]
async fn should_log_out_and_promote_the_next_usable_account_when_another_is_expired() {
    let storage = Arc::new(MockCredentialsStorage::new());

    let active_id = Uuid::new_v4();
    let offline_id = Uuid::new_v4();
    storage
        .upsert(
            CredentialsBuilder::new("Active")
                .with_id(active_id)
                .with_active(true)
                .build(),
        )
        .await
        .unwrap();
    // Inserted before the offline account, so it is the first promotion candidate.
    storage
        .upsert(
            CredentialsBuilder::new("Expired")
                .with_microsoft(chrono::Utc::now() - chrono::Duration::hours(1))
                .build(),
        )
        .await
        .unwrap();
    storage
        .upsert(
            CredentialsBuilder::new("Offline")
                .with_id(offline_id)
                .build(),
        )
        .await
        .unwrap();

    LogoutUseCase::new(storage.clone())
        .execute(active_id)
        .await
        .unwrap();

    assert_eq!(
        storage.find_active().await.unwrap().unwrap().id(),
        offline_id
    );
}

#[tokio::test]
async fn should_log_out_when_every_remaining_account_is_expired() {
    let storage = Arc::new(MockCredentialsStorage::new());

    let active_id = Uuid::new_v4();
    storage
        .upsert(
            CredentialsBuilder::new("Active")
                .with_id(active_id)
                .with_active(true)
                .build(),
        )
        .await
        .unwrap();
    storage
        .upsert(
            CredentialsBuilder::new("Expired")
                .with_microsoft(chrono::Utc::now() - chrono::Duration::hours(1))
                .build(),
        )
        .await
        .unwrap();

    LogoutUseCase::new(storage.clone())
        .execute(active_id)
        .await
        .unwrap();

    assert!(storage.find_active().await.unwrap().is_none());
}
