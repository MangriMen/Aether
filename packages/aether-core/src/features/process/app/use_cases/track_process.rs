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

/// Credits the playtime of a run as it goes. The record of the run holds the moment up to
/// which it is credited, and a later launcher credits from there: every failure on the way
/// leaves time uncredited rather than credited twice.
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

        loop {
            match self.process_storage.try_wait(process_id).await {
                Ok(Some(Some(exit_status))) => {
                    self.finish(process_id, &instance_id, last_updated_playtime)
                        .await;
                    return Some(exit_status);
                }
                Ok(Some(None)) => {} // Still running
                // An unknown exit must not look like a clean one. The game may still run, so
                // its record stays for a later launcher to find.
                Ok(None) | Err(_) => {
                    self.checkpoint(&mut last_updated_playtime, process_id, &instance_id)
                        .await;
                    return None;
                }
            }

            tokio::time::sleep(PROCESS_CHECK_INTERVAL).await;

            if Utc::now().signed_duration_since(last_updated_playtime) >= UPDATE_PLAYTIME_INTERVAL {
                self.checkpoint(&mut last_updated_playtime, process_id, &instance_id)
                    .await;
            }
        }
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

    /// Credits the time since `last_updated` once the record says it is credited. When the
    /// record cannot be moved, nothing is credited and `last_updated` stays: the next
    /// checkpoint tries both again.
    async fn checkpoint(
        &self,
        last_updated: &mut DateTime<Utc>,
        process_id: Uuid,
        instance_id: &InstanceId,
    ) {
        let now = Utc::now();
        if let Err(err) = self.records.touch_playtime(process_id, now).await {
            tracing::warn!(
                %process_id,
                %instance_id,
                error = %error_chain(&err),
                "Could not save the playtime mark of the game"
            );
            return;
        }
        self.credit_playtime(instance_id, now.signed_duration_since(*last_updated))
            .await;
        *last_updated = now;
    }

    /// Credits the rest of an ended run once its record is gone, so that no later launcher
    /// credits it again. A record that cannot be removed is tried once more, then the rest of
    /// the run is left uncredited.
    async fn finish(
        &self,
        process_id: Uuid,
        instance_id: &InstanceId,
        last_updated: DateTime<Utc>,
    ) {
        let removed = match self.records.remove(process_id).await {
            Ok(_) => Ok(()),
            Err(_) => self.records.remove(process_id).await.map(|_| ()),
        };
        if let Err(err) = removed {
            tracing::warn!(
                %process_id,
                %instance_id,
                error = %error_chain(&err),
                "Could not remove the record of the ended game: its last playtime is not credited"
            );
            return;
        }
        self.credit_playtime(instance_id, Utc::now().signed_duration_since(last_updated))
            .await;
    }

    async fn credit_playtime(&self, instance_id: &InstanceId, elapsed: Duration) {
        let result = self
            .instance_storage
            .upsert_with(instance_id, |instance| {
                instance.time_played += elapsed.num_seconds().max(0).cast_unsigned();
                Ok(())
            })
            .await;

        if let Err(err) = result {
            tracing::warn!(
                %instance_id,
                error = %error_chain(&err),
                "Could not credit the playtime"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;
    use process_wrap::tokio::ChildWrapper;
    use rstest::rstest;

    use super::*;
    use crate::features::{
        instance::{Instance, InstanceBuilder, InstanceChange, InstanceError, InstanceName},
        minecraft::{GameVersion, ModLoader},
        process::{MinecraftProcessMetadata, ProcessError, ProcessRecord},
    };
    use crate::shared::error::DatabaseError;

    /// What the fakes were asked to do, in order.
    type Calls = Arc<Mutex<Vec<String>>>;

    fn status(code: i32) -> ExitStatus {
        #[cfg(windows)]
        {
            std::os::windows::process::ExitStatusExt::from_raw(code.cast_unsigned())
        }
        #[cfg(unix)]
        {
            std::os::unix::process::ExitStatusExt::from_raw(code << 8)
        }
    }

    /// How the game looks when it is first looked at.
    #[derive(Clone, Copy, Debug)]
    enum Game {
        Exited,
        /// The storage does not know the game.
        Unknown,
        /// The OS refused the wait.
        Unreadable,
    }

    #[async_trait]
    impl ProcessStorage for Game {
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

        async fn try_wait(&self, id: Uuid) -> Result<Option<Option<ExitStatus>>, ProcessError> {
            match self {
                Game::Exited => Ok(Some(Some(status(0)))),
                Game::Unknown => Ok(None),
                Game::Unreadable => Err(ProcessError::WaitError { id: id.to_string() }),
            }
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

    /// Fails the first `failing_removes` removes, and every touch when `failing_touch`.
    #[derive(Default)]
    struct Records {
        calls: Calls,
        failing_removes: Mutex<u32>,
        failing_touch: bool,
    }

    #[async_trait]
    impl ProcessRecordStorage for Records {
        async fn insert(&self, _record: &ProcessRecord) -> Result<(), DatabaseError> {
            unimplemented!()
        }

        async fn remove(&self, _id: Uuid) -> Result<bool, DatabaseError> {
            self.calls.lock().unwrap().push("remove".to_owned());
            let mut failing = self.failing_removes.lock().unwrap();
            if *failing > 0 {
                *failing -= 1;
                return Err(DatabaseError::new("database is locked"));
            }
            Ok(true)
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
            if self.failing_touch {
                return Err(DatabaseError::new("database is locked"));
            }
            Ok(())
        }
    }

    fn instance_id() -> InstanceId {
        InstanceId::try_from("test-instance").unwrap()
    }

    fn use_case(game: Game, records: Records) -> (TrackProcessUseCase, Calls) {
        let calls = records.calls.clone();
        let instance = InstanceBuilder::new(
            instance_id(),
            InstanceName::parse("Test Instance").unwrap(),
            GameVersion::from_static("1.20.1"),
            ModLoader::Vanilla,
        )
        .build();
        let use_case = TrackProcessUseCase::new(
            Arc::new(game),
            Arc::new(Instances {
                calls: calls.clone(),
                instance: Mutex::new(instance),
            }),
            Arc::new(records),
        );
        (use_case, calls)
    }

    async fn track(use_case: &TrackProcessUseCase) -> Option<ExitStatus> {
        use_case
            .execute(TrackProcessParams {
                process_id: Uuid::new_v4(),
                instance_id: instance_id(),
            })
            .await
    }

    /// The calls with the touch times left out.
    fn kinds(calls: &Calls) -> Vec<String> {
        calls
            .lock()
            .unwrap()
            .iter()
            .map(|call| call.split(' ').next().unwrap().to_owned())
            .collect()
    }

    #[tokio::test]
    async fn should_remove_the_record_before_the_last_credit_when_the_game_exits() {
        let (use_case, calls) = use_case(Game::Exited, Records::default());

        let exit_status = track(&use_case).await;

        assert_eq!(exit_status, Some(status(0)));
        assert_eq!(*calls.lock().unwrap(), ["remove", "credit 0"]);
    }

    #[tokio::test]
    async fn should_credit_after_a_second_try_when_the_record_cannot_be_removed_at_first() {
        let records = Records {
            failing_removes: Mutex::new(1),
            ..Records::default()
        };
        let (use_case, calls) = use_case(Game::Exited, records);

        track(&use_case).await;

        assert_eq!(*calls.lock().unwrap(), ["remove", "remove", "credit 0"]);
    }

    #[tokio::test]
    async fn should_not_credit_the_rest_of_the_run_when_the_record_cannot_be_removed() {
        let records = Records {
            failing_removes: Mutex::new(2),
            ..Records::default()
        };
        let (use_case, calls) = use_case(Game::Exited, records);

        track(&use_case).await;

        assert_eq!(*calls.lock().unwrap(), ["remove", "remove"]);
    }

    #[rstest]
    #[case::unknown_to_the_storage(Game::Unknown)]
    #[case::wait_refused(Game::Unreadable)]
    #[tokio::test]
    async fn should_keep_the_record_and_credit_up_to_now_when_the_exit_is_unknown(
        #[case] game: Game,
    ) {
        let (use_case, calls) = use_case(game, Records::default());

        let exit_status = track(&use_case).await;

        assert_eq!(exit_status, None);
        assert_eq!(kinds(&calls), ["touch", "credit"]);
    }

    #[tokio::test]
    async fn should_move_the_playtime_mark_before_the_credit_when_a_minute_has_passed() {
        let (use_case, calls) = use_case(Game::Exited, Records::default());
        let mut last_updated = Utc::now() - Duration::seconds(61);

        use_case
            .checkpoint(&mut last_updated, Uuid::new_v4(), &instance_id())
            .await;

        let expected = [
            format!("touch {}", last_updated.timestamp_micros()),
            "credit 61".to_owned(),
        ];
        assert_eq!(*calls.lock().unwrap(), expected);
    }

    #[tokio::test]
    async fn should_neither_credit_nor_move_on_when_the_playtime_mark_cannot_be_saved() {
        let records = Records {
            failing_touch: true,
            ..Records::default()
        };
        let (use_case, calls) = use_case(Game::Exited, records);
        let before = Utc::now() - Duration::seconds(61);
        let mut last_updated = before;

        use_case
            .checkpoint(&mut last_updated, Uuid::new_v4(), &instance_id())
            .await;

        assert_eq!(last_updated, before);
        assert_eq!(kinds(&calls), ["touch"]);
    }
}
