use std::sync::Arc;

use crate::features::auth::app::ActiveAccountHelper;
use crate::features::auth::app::ports::{
    CreateOfflineAccountUseCasePort, GetAccountsUseCasePort, LogoutUseCasePort,
    SetActiveAccountUseCasePort,
};

/// Extension trait providing access to all auth feature use cases and services.
///
/// Implemented on the core dependency injection container to expose
/// auth-specific functionality in a centralized manner.
pub trait AuthFeature {
    fn create_offline_account_use_case(&self) -> Arc<dyn CreateOfflineAccountUseCasePort>;
    fn get_accounts_use_case(&self) -> Arc<dyn GetAccountsUseCasePort>;
    fn logout_use_case(&self) -> Arc<dyn LogoutUseCasePort>;
    fn set_active_account_use_case(&self) -> Arc<dyn SetActiveAccountUseCasePort>;
    fn active_account_helper(&self) -> Arc<ActiveAccountHelper>;
}
