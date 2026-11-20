use async_trait::async_trait;

use crate::features::instance::InstanceId;

#[async_trait]
pub trait SkippedHookNoticeService: Send + Sync {
    /// Tells the user that the post-exit hook of the instance's game was not run.
    async fn notify(&self, instance_id: &InstanceId);
}
