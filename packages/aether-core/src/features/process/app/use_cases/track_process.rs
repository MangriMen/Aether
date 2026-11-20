use std::{process::ExitStatus, sync::Arc, time};

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::core::domain::error_chain;
use crate::features::{
    instance::{InstanceId, InstanceStorage, InstanceStorageExt},
    process::{ProcessRecordStorage, ProcessStorage, TrackProcessService},
};

const PROCESS_CHECK_INTERVAL: time::Duration = time::Duration::from_millis(50);
const UPDATE_PLAYTIME_INTERVAL: Duration = Duration::seconds(60);

pub struct TrackProcessParams {
    pub process_id: Uuid,
    pub instance_id: InstanceId,
}

pub struct TrackProcessUseCase {
    process_storage: Arc<dyn ProcessStorage>,
    instance_storage: Arc<dyn InstanceStorage>,
    records: Arc<dyn ProcessRecordStorage>,
}

#[async_trait::async_trait]
impl TrackProcessService for TrackProcessUseCase {
    async fn execute(&self, params: TrackProcessParams) -> Option<ExitStatus> {
        let TrackProcessParams {
            process_id,
            instance_id,
        } = params;

        let mut last_updated_playtime = Utc::now();

        let exit_status = loop {
            match self.process_storage.try_wait(process_id).await {
                Ok(Some(Some(exit_status))) => break Some(exit_status),
                Ok(Some(None)) => {} // Still running
                // An unknown exit must not look like a clean one.
                Ok(None) | Err(_) => break None,
            }

            tokio::time::sleep(PROCESS_CHECK_INTERVAL).await;

            self.update_playtime(&mut last_updated_playtime, process_id, &instance_id)
                .await;
        };

        // The record goes before the last credit: a later launcher credits the run again from
        // a record it finds, and adding too little is better than adding the run twice.
        if let Err(err) = self.records.remove(process_id).await {
            tracing::warn!(
                %process_id,
                %instance_id,
                error = %error_chain(&err),
                "Could not remove the record of the ended game"
            );
        }
        let elapsed = Utc::now().signed_duration_since(last_updated_playtime);
        self.credit_playtime(&instance_id, elapsed).await;

        exit_status
    }
}

impl TrackProcessUseCase {
    pub fn new(
        process_storage: Arc<dyn ProcessStorage>,
        instance_storage: Arc<dyn InstanceStorage>,
        records: Arc<dyn ProcessRecordStorage>,
    ) -> Self {
        Self {
            process_storage,
            instance_storage,
            records,
        }
    }

    async fn update_playtime(
        &self,
        last_updated: &mut DateTime<Utc>,
        process_id: Uuid,
        instance_id: &InstanceId,
    ) {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(*last_updated);
        if elapsed < UPDATE_PLAYTIME_INTERVAL {
            return;
        }

        // The mark moves before the credit, for the same reason the record goes before the
        // last one: a launcher that dies in between loses the minute instead of adding it twice.
        if let Err(err) = self.records.touch_playtime(process_id, now).await {
            tracing::warn!(
                %process_id,
                %instance_id,
                error = %error_chain(&err),
                "Could not save the playtime mark of the game"
            );
        }
        self.credit_playtime(instance_id, elapsed).await;

        *last_updated = now;
    }

    async fn credit_playtime(&self, id: &InstanceId, elapsed: Duration) {
        let result = self
            .instance_storage
            .upsert_with(id, |instance| {
                instance.time_played += elapsed.num_seconds().max(0).cast_unsigned();
                Ok(())
            })
            .await;

        if let Err(e) = result {
            tracing::warn!("Failed to update playtime for profile {}: {}", id, e);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;
    use process_wrap::tokio::ChildWrapper;

    use super::*;
    use crate::features::{
        instance::{Instance, InstanceBuilder, InstanceChange, InstanceError, InstanceName},
        minecraft::{GameVersion, ModLoader},
        process::{MinecraftProcessMetadata, ProcessError, ProcessRecord},
    };
    use crate::shared::error::DatabaseError;

    /// What the fakes were asked to do, in order.
    type Calls = Arc<Mutex<Vec<String>>>;

    /// A game that has ended by the time it is first looked at.
    struct Ended;

    #[async_trait]
    impl ProcessStorage for Ended {
        async fn insert(
            &self,
            _process: MinecraftProcessMetadata,
            _child: Box<dyn ChildWrapper>,
        ) -> Result<(), ProcessError> {
            unimplemented!()
        }

        async fn remove(&self, _id: Uuid) -> Result<(), ProcessError> {
            unimplemented!()
        }

        async fn list_metadata(&self) -> Result<Vec<MinecraftProcessMetadata>, ProcessError> {
            unimplemented!()
        }

        async fn get_metadata(
            &self,
            _id: Uuid,
        ) -> Result<Option<MinecraftProcessMetadata>, ProcessError> {
            unimplemented!()
        }

        async fn try_wait(&self, _id: Uuid) -> Result<Option<Option<ExitStatus>>, ProcessError> {
            Ok(None)
        }

        async fn kill(&self, _id: Uuid) -> Result<(), ProcessError> {
            unimplemented!()
        }

        async fn was_killed(&self, _id: Uuid) -> bool {
            unimplemented!()
        }
    }

    struct Instances {
        calls: Calls,
        instance: Mutex<Instance>,
    }

    #[async_trait]
    impl InstanceStorage for Instances {
        async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
            unimplemented!()
        }

        async fn get(&self, _id: &InstanceId) -> Result<Instance, InstanceError> {
            unimplemented!()
        }

        async fn upsert(&self, _instance: &Instance) -> Result<(), InstanceError> {
            unimplemented!()
        }

        async fn remove(&self, _id: &InstanceId) -> Result<(), InstanceError> {
            unimplemented!()
        }

        async fn update(
            &self,
            _id: &InstanceId,
            change: InstanceChange<'_>,
        ) -> Result<Instance, InstanceError> {
            let mut instance = self.instance.lock().unwrap();
            change(&mut instance)?;
            self.calls
                .lock()
                .unwrap()
                .push(format!("credit {}", instance.time_played));
            Ok(instance.clone())
        }
    }

    struct Records {
        calls: Calls,
    }

    #[async_trait]
    impl ProcessRecordStorage for Records {
        async fn insert(&self, _record: &ProcessRecord) -> Result<(), DatabaseError> {
            unimplemented!()
        }

        async fn remove(&self, _id: Uuid) -> Result<(), DatabaseError> {
            self.calls.lock().unwrap().push("remove".to_owned());
            Ok(())
        }

        async fn list(&self) -> Result<Vec<ProcessRecord>, DatabaseError> {
            unimplemented!()
        }

        async fn list_for_instance(
            &self,
            _instance_id: &InstanceId,
        ) -> Result<Vec<ProcessRecord>, DatabaseError> {
            unimplemented!()
        }

        async fn touch_playtime(
            &self,
            _id: Uuid,
            until: DateTime<Utc>,
        ) -> Result<(), DatabaseError> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("touch {}", until.timestamp_micros()));
            Ok(())
        }
    }

    fn instance_id() -> InstanceId {
        InstanceId::try_from("test-instance").unwrap()
    }

    fn use_case(calls: &Calls) -> TrackProcessUseCase {
        let instance = InstanceBuilder::new(
            instance_id(),
            InstanceName::parse("Test Instance").unwrap(),
            GameVersion::from_static("1.20.1"),
            ModLoader::Vanilla,
        )
        .build();
        TrackProcessUseCase::new(
            Arc::new(Ended),
            Arc::new(Instances {
                calls: calls.clone(),
                instance: Mutex::new(instance),
            }),
            Arc::new(Records {
                calls: calls.clone(),
            }),
        )
    }

    #[tokio::test]
    async fn should_remove_the_record_before_the_last_credit_when_the_game_ends() {
        let calls = Calls::default();

        use_case(&calls)
            .execute(TrackProcessParams {
                process_id: Uuid::new_v4(),
                instance_id: instance_id(),
            })
            .await;

        assert_eq!(*calls.lock().unwrap(), ["remove", "credit 0"]);
    }

    #[tokio::test]
    async fn should_move_the_playtime_mark_before_the_credit_when_a_minute_has_passed() {
        let calls = Calls::default();
        let mut last_updated = Utc::now() - Duration::seconds(61);

        use_case(&calls)
            .update_playtime(&mut last_updated, Uuid::new_v4(), &instance_id())
            .await;

        let expected = [
            format!("touch {}", last_updated.timestamp_micros()),
            "credit 61".to_owned(),
        ];
        assert_eq!(*calls.lock().unwrap(), expected);
    }

    #[tokio::test]
    async fn should_leave_the_playtime_mark_when_less_than_a_minute_has_passed() {
        let calls = Calls::default();
        let mut last_updated = Utc::now() - Duration::seconds(30);

        use_case(&calls)
            .update_playtime(&mut last_updated, Uuid::new_v4(), &instance_id())
            .await;

        assert!(calls.lock().unwrap().is_empty());
    }
}
