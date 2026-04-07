use std::sync::Arc;

use crate::features::auth::{CredentialsStorage, GetAccountsUseCase};

use super::fixtures::{CredentialsBuilder, MockCredentialsStorage};

#[tokio::test]
async fn test_get_accounts_returns_all() {
    let storage = Arc::new(MockCredentialsStorage::new());

    storage
        .upsert(CredentialsBuilder::new("User1").build())
        .await
        .unwrap();

    storage
        .upsert(CredentialsBuilder::new("User2").build())
        .await
        .unwrap();

    storage
        .upsert(CredentialsBuilder::new("User3").build())
        .await
        .unwrap();

    let use_case = GetAccountsUseCase::new(storage.clone());

    let accounts = use_case.execute().await.unwrap();

    assert_eq!(accounts.len(), 3);
    assert!(accounts.iter().any(|a| a.username.as_str() == "User1"));
    assert!(accounts.iter().any(|a| a.username.as_str() == "User2"));
    assert!(accounts.iter().any(|a| a.username.as_str() == "User3"));
}

#[tokio::test]
async fn test_get_accounts_empty() {
    let storage = Arc::new(MockCredentialsStorage::new());

    let use_case = GetAccountsUseCase::new(storage.clone());

    let accounts = use_case.execute().await.unwrap();

    assert_eq!(accounts.len(), 0);
}
