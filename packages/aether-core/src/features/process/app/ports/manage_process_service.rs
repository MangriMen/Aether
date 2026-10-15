use async_trait::async_trait;

use crate::features::process::app::use_cases::ManageProcessParams;
use crate::features::process::domain::ProcessError;

#[async_trait]
pub trait ManageProcessService: Send + Sync {
    async fn execute(&self, params: ManageProcessParams) -> Result<(), ProcessError>;
}
