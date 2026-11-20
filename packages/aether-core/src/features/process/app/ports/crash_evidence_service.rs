use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::features::instance::InstanceId;

/// Where a game that ended without a readable exit code left word of a crash.
#[async_trait]
pub trait CrashEvidenceService: Send + Sync {
    /// Whether the game of the instance, which ran as process `pid`, wrote a crash report, or a
    /// fatal error log of the JVM, at `since` or later.
    async fn found(&self, instance_id: &InstanceId, pid: u32, since: DateTime<Utc>) -> bool;
}
