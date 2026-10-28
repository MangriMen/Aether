use std::sync::Arc;
use uuid::Uuid;

use crate::features::auth::app::{
    CredentialsStorage, SetActiveAccountUseCase, SetActiveAccountUseCasePort,
};
use crate::features::auth::{app::AuthApplicationError, domain::AuthDomainError};

use super::fixtures::{CredentialsBuilder, MockCredentialsStorage};

#[tokio::test]
async fn should_activate_the_account_when_it_exists() {
    let storage = Arc::new(MockCredentialsStorage::new());

    let cred1_id = Uuid::new_v4();
    let cred2_id = Uuid::new_v4();

    storage
        .upsert(CredentialsBuilder::new("User1").with_id(cred1_id).build())
        .await
        .unwrap();

    storage
        .upsert(CredentialsBuilder::new("User2").with_id(cred2_id).build())
        .await
        .unwrap();

    let use_case = SetActiveAccountUseCase::new(storage.clone());

    let active_account = use_case.execute(cred2_id).await.unwrap();
    assert!(active_account.active);
    assert_eq!(active_account.id, cred2_id);

    let currently_active = storage.find_active().await.unwrap().unwrap();
    assert_eq!(currently_active.id(), cred2_id);

    // Verify previous active account was deactivated by ActiveAccountHelper
    let cred1 = storage.get(cred1_id).await.unwrap();
    assert!(!cred1.is_active());
}

#[tokio::test]
async fn should_keep_the_account_active_when_it_is_already_active() {
    let storage = Arc::new(MockCredentialsStorage::new());

    let cred_id = Uuid::new_v4();
    storage
        .upsert(
            CredentialsBuilder::new("User1")
                .with_id(cred_id)
                .with_active(true)
                .build(),
        )
        .await
        .unwrap();

    let use_case = SetActiveAccountUseCase::new(storage.clone());

    let result = use_case.execute(cred_id).await.unwrap();
    assert!(result.active);
    assert_eq!(result.id, cred_id);
}

#[tokio::test]
async fn should_fail_to_activate_when_the_account_does_not_exist() {
    let storage = Arc::new(MockCredentialsStorage::new());

    storage
        .upsert(CredentialsBuilder::new("User1").build())
        .await
        .unwrap();

    let use_case = SetActiveAccountUseCase::new(storage.clone());

    let nonexistent_id = Uuid::new_v4();
    let result = use_case.execute(nonexistent_id).await;

    assert!(
        matches!(
            result,
            Err(AuthApplicationError::Domain(AuthDomainError::CredentialsNotFound { id }))
                if id == nonexistent_id
        ),
        "{result:?}"
    );
}
