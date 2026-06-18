use std::sync::Arc;

use crate::features::update::{UpdateService, UpdateStatus};

pub struct CheckForUpdatesUseCase<US: UpdateService> {
    update_service: Arc<US>,
}

impl<US: UpdateService> CheckForUpdatesUseCase<US> {
    pub fn new(update_service: Arc<US>) -> Self {
        Self { update_service }
    }

    pub async fn execute(&self) -> Result<Option<UpdateStatus>, String> {
        self.update_service.check().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockUpdateService {
        check_result: Result<Option<UpdateStatus>, String>,
    }

    #[async_trait]
    impl UpdateService for MockUpdateService {
        async fn check(&self) -> Result<Option<UpdateStatus>, String> {
            self.check_result.clone()
        }

        async fn install(&self) -> Result<(), String> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn check_returns_update_when_available() {
        let service = Arc::new(MockUpdateService {
            check_result: Ok(Some(UpdateStatus {
                version: Some("1.0.0".to_string()),
                date: None,
                body: Some("Bug fixes".to_string()),
            })),
        });
        let use_case = CheckForUpdatesUseCase::new(service);
        let result = use_case.execute().await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().version.unwrap(), "1.0.0");
    }

    #[tokio::test]
    async fn check_returns_none_when_no_update() {
        let service = Arc::new(MockUpdateService {
            check_result: Ok(None),
        });
        let use_case = CheckForUpdatesUseCase::new(service);
        let result = use_case.execute().await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn check_returns_error() {
        let service = Arc::new(MockUpdateService {
            check_result: Err("network error".to_string()),
        });
        let use_case = CheckForUpdatesUseCase::new(service);
        let result = use_case.execute().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "network error");
    }
}
