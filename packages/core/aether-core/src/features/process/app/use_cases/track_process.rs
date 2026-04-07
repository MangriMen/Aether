use std::{process::ExitStatus, sync::Arc, time};

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::features::{
    instance::{InstanceStorage, InstanceStorageExt},
    process::ProcessStorage,
};

const PROCESS_CHECK_INTERVAL: time::Duration = time::Duration::from_millis(50);
const UPDATE_PLAYTIME_INTERVAL: Duration = Duration::seconds(60);

pub struct TrackProcessParams {
    pub process_uuid: Uuid,
    pub instance_id: String,
}

pub struct TrackProcessUseCase<PS, IS> {
    process_storage: Arc<PS>,
    instance_storage: Arc<IS>,
}

impl<PS: ProcessStorage, IS: InstanceStorage> TrackProcessUseCase<PS, IS> {
    pub fn new(process_storage: Arc<PS>, instance_storage: Arc<IS>) -> Self {
        Self {
            process_storage,
            instance_storage,
        }
    }

    async fn update_playtime(&self, last_updated: &mut DateTime<Utc>, id: &str, force: bool) {
        let elapsed_seconds = Utc::now().signed_duration_since(*last_updated);

        if elapsed_seconds >= UPDATE_PLAYTIME_INTERVAL || force {
            let result = async {
                self.instance_storage
                    .upsert_with(id, |instance| {
                        instance.time_played += elapsed_seconds.num_seconds() as u64;
                        Ok(())
                    })
                    .await
            }
            .await;

            if let Err(e) = result {
                tracing::warn!("Failed to update playtime for profile {}: {}", id, e);
            }

            *last_updated = Utc::now();
        }
    }

    pub async fn execute(&self, params: TrackProcessParams) -> ExitStatus {
        let TrackProcessParams {
            process_uuid,
            instance_id,
        } = params;

        let mut last_updated_playtime = Utc::now();

        loop {
            match self.process_storage.try_wait(process_uuid).await {
                Ok(Some(Some(exit_status))) => {
                    // Process exited successfully
                    self.update_playtime(&mut last_updated_playtime, &instance_id, true)
                        .await;
                    return exit_status;
                }
                Ok(Some(None)) => {} // Still running
                Ok(None) | Err(_) => {
                    self.update_playtime(&mut last_updated_playtime, &instance_id, true)
                        .await;
                    return ExitStatus::default();
                }
            }

            tokio::time::sleep(PROCESS_CHECK_INTERVAL).await;

            self.update_playtime(&mut last_updated_playtime, &instance_id, false)
                .await;
        }
    }
}
