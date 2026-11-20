//! Fakes of the ports the taking over of games uses, shared by its tests and those of the
//! launch check.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, SubsecRound, Utc};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::features::{
    events::{Event, ProcessEvent, SharedEventEmitter, test_support::ChannelEmitter},
    game_log::{
        GameLogError, ListGameLogSourcesUseCasePort, LogSource, LogSourceKind, LogSourceRef,
    },
    instance::{
        Instance, InstanceBuilder, InstanceChange, InstanceError, InstanceId, InstanceName,
        InstanceStorage,
    },
    minecraft::{GameVersion, ModLoader},
    process::{
        AdoptedProcess, KillScope, ManageProcessService, OsProcessIdentity, ProcessError,
        ProcessProbe, ProcessRecord, ProcessRecordStorage, ProcessState,
        TakeSkippedHookNoticesUseCase, infra::InMemoryProcessStorage,
    },
};
use crate::shared::error::DatabaseError;

use super::ManageProcessParams;

/// What the OS tells of a process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Look {
    Running,
    Gone,
    /// The OS could not tell.
    Unknown,
}

/// The launcher that runs the tests.
pub(super) fn launcher() -> OsProcessIdentity {
    OsProcessIdentity::new(1, 100, "boot".to_owned())
}

/// A launcher that ran before.
pub(super) fn earlier_launcher() -> OsProcessIdentity {
    OsProcessIdentity::new(2, 50, "boot".to_owned())
}

pub(super) fn instance_id() -> InstanceId {
    InstanceId::try_from("test-instance").unwrap()
}

/// A game started by `owner` ten minutes ago, credited up to then.
pub(super) fn record_of(pid: u32, owner: OsProcessIdentity) -> ProcessRecord {
    // Whole seconds, as a log time read back in milliseconds would cut them.
    let started_at = (Utc::now() - chrono::Duration::minutes(10)).trunc_subsecs(0);
    ProcessRecord {
        id: Uuid::new_v4(),
        instance_id: instance_id(),
        process: OsProcessIdentity::new(pid, 1000 + u64::from(pid), "boot".to_owned()),
        started_at,
        kill_scope: KillScope::Process,
        post_exit_command: "backup.sh".to_owned(),
        playtime_credited_until: started_at,
        owner,
    }
}

/// Keeps the records in memory, with the owner check of the real storage.
#[derive(Default)]
pub(super) struct Records {
    pub(super) saved: Mutex<Vec<ProcessRecord>>,
    pub(super) touches: Mutex<Vec<Uuid>>,
    pub(super) failing_reads: Mutex<bool>,
    /// The next take over finds the record taken over by this launcher first.
    pub(super) taken_first_by: Mutex<Option<OsProcessIdentity>>,
}

impl Records {
    pub(super) fn with(records: &[ProcessRecord]) -> Arc<Self> {
        Arc::new(Self {
            saved: Mutex::new(records.to_vec()),
            ..Self::default()
        })
    }

    pub(super) fn get(&self, id: Uuid) -> Option<ProcessRecord> {
        self.saved
            .lock()
            .unwrap()
            .iter()
            .find(|record| record.id == id)
            .cloned()
    }

    pub(super) fn set_owner(&self, id: Uuid, owner: OsProcessIdentity) {
        let mut saved = self.saved.lock().unwrap();
        let record = saved.iter_mut().find(|record| record.id == id).unwrap();
        record.owner = owner;
    }

    fn fail_if_asked(&self) -> Result<(), DatabaseError> {
        if *self.failing_reads.lock().unwrap() {
            return Err(DatabaseError::new("database is locked"));
        }
        Ok(())
    }
}

#[async_trait]
impl ProcessRecordStorage for Records {
    async fn insert(&self, record: &ProcessRecord) -> Result<(), DatabaseError> {
        self.saved.lock().unwrap().push(record.clone());
        Ok(())
    }

    async fn remove(&self, id: Uuid) -> Result<bool, DatabaseError> {
        let mut saved = self.saved.lock().unwrap();
        let before = saved.len();
        saved.retain(|record| record.id != id);
        Ok(saved.len() < before)
    }

    async fn list(&self) -> Result<Vec<ProcessRecord>, DatabaseError> {
        self.fail_if_asked()?;
        Ok(self.saved.lock().unwrap().clone())
    }

    async fn list_for_instance(
        &self,
        instance_id: &InstanceId,
    ) -> Result<Vec<ProcessRecord>, DatabaseError> {
        self.fail_if_asked()?;
        Ok(self
            .saved
            .lock()
            .unwrap()
            .iter()
            .filter(|record| &record.instance_id == instance_id)
            .cloned()
            .collect())
    }

    async fn touch_playtime(&self, id: Uuid, until: DateTime<Utc>) -> Result<(), DatabaseError> {
        self.touches.lock().unwrap().push(id);
        if let Some(record) = self
            .saved
            .lock()
            .unwrap()
            .iter_mut()
            .find(|record| record.id == id)
        {
            record.playtime_credited_until = until;
        }
        Ok(())
    }

    async fn take_over(
        &self,
        id: Uuid,
        from: &OsProcessIdentity,
        to: &OsProcessIdentity,
    ) -> Result<bool, DatabaseError> {
        if let Some(first) = self.taken_first_by.lock().unwrap().take() {
            self.set_owner(id, first);
        }
        let mut saved = self.saved.lock().unwrap();
        match saved
            .iter_mut()
            .find(|record| record.id == id && record.owner.matches(from))
        {
            Some(record) => {
                record.owner = to.clone();
                Ok(true)
            }
            None => Ok(false),
        }
    }
}

/// Tells of each pid what it was told to; a pid it was not told of is gone.
pub(super) struct Probe {
    own: Option<OsProcessIdentity>,
    looks: Mutex<HashMap<u32, Look>>,
    /// The pids looked at, in order.
    pub(super) asked: Mutex<Vec<u32>>,
}

impl Probe {
    pub(super) fn new(own: Option<OsProcessIdentity>) -> Arc<Self> {
        Arc::new(Self {
            own,
            looks: Mutex::default(),
            asked: Mutex::default(),
        })
    }

    pub(super) fn set(&self, pid: u32, look: Look) {
        self.looks.lock().unwrap().insert(pid, look);
    }

    pub(super) fn times_asked(&self, pid: u32) -> usize {
        self.asked
            .lock()
            .unwrap()
            .iter()
            .filter(|asked| **asked == pid)
            .count()
    }

    fn look(&self, pid: u32) -> Look {
        self.asked.lock().unwrap().push(pid);
        self.looks
            .lock()
            .unwrap()
            .get(&pid)
            .copied()
            .unwrap_or(Look::Gone)
    }
}

#[async_trait]
impl ProcessProbe for Probe {
    async fn identity_of(&self, _pid: u32) -> std::io::Result<OsProcessIdentity> {
        unimplemented!()
    }

    fn own_identity(&self) -> Option<&OsProcessIdentity> {
        self.own.as_ref()
    }

    async fn adopt(
        &self,
        record: &ProcessRecord,
    ) -> std::io::Result<Option<Box<dyn AdoptedProcess>>> {
        match self.look(record.process.pid()) {
            Look::Running => Ok(Some(Box::new(Game::default()))),
            Look::Gone => Ok(None),
            Look::Unknown => Err(std::io::Error::other("the process table cannot be read")),
        }
    }

    async fn is_alive(&self, process: &OsProcessIdentity) -> std::io::Result<bool> {
        match self.look(process.pid()) {
            Look::Running => Ok(true),
            Look::Gone => Ok(false),
            Look::Unknown => Err(std::io::Error::other("the process table cannot be read")),
        }
    }
}

/// A taken over game that runs until it is stopped.
#[derive(Debug, Default)]
pub(super) struct Game {
    stopped: bool,
}

impl AdoptedProcess for Game {
    fn try_wait(&mut self) -> std::io::Result<ProcessState> {
        Ok(if self.stopped {
            ProcessState::Exited(None)
        } else {
            ProcessState::Running
        })
    }

    fn start_kill(&mut self) -> std::io::Result<()> {
        self.stopped = true;
        Ok(())
    }
}

/// What the following of a taken over game was started with.
#[derive(Debug, Clone)]
pub(super) struct Followed {
    pub(super) process_id: Uuid,
    pub(super) post_exit_command: String,
    pub(super) captured: bool,
    pub(super) origin: crate::features::process::ProcessOrigin,
    pub(super) crash_watch: Option<super::CrashWatch>,
}

#[derive(Default)]
pub(super) struct Manage {
    pub(super) followed: Mutex<Vec<Followed>>,
}

#[async_trait]
impl ManageProcessService for Manage {
    async fn execute(&self, params: ManageProcessParams) -> Result<(), ProcessError> {
        self.followed.lock().unwrap().push(Followed {
            process_id: params.process_id,
            post_exit_command: params.post_exit_command,
            captured: params.capture.is_some(),
            origin: params.origin,
            crash_watch: params.crash_watch,
        });
        Ok(())
    }
}

/// One instance, whose playtime it keeps.
pub(super) struct Instances(Mutex<Instance>);

impl Instances {
    pub(super) fn new() -> Arc<Self> {
        let instance = InstanceBuilder::new(
            instance_id(),
            InstanceName::parse("Test Instance").unwrap(),
            GameVersion::from_static("1.20.1"),
            ModLoader::Vanilla,
        )
        .build();
        Arc::new(Self(Mutex::new(instance)))
    }

    pub(super) fn time_played(&self) -> u64 {
        self.0.lock().unwrap().time_played
    }
}

#[async_trait]
impl InstanceStorage for Instances {
    async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
        unimplemented!()
    }

    async fn get(&self, _id: &InstanceId) -> Result<Instance, InstanceError> {
        Ok(self.0.lock().unwrap().clone())
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
        let mut instance = self.0.lock().unwrap();
        change(&mut instance)?;
        Ok(instance.clone())
    }
}

/// The logs of the instance: a `latest.log` last written at the time it holds, if any.
#[derive(Default)]
pub(super) struct GameLogs(pub(super) Mutex<Option<DateTime<Utc>>>);

#[async_trait]
impl ListGameLogSourcesUseCasePort for GameLogs {
    async fn execute(&self, _instance_id: InstanceId) -> Result<Vec<LogSource>, GameLogError> {
        let Some(modified) = *self.0.lock().unwrap() else {
            return Ok(Vec::new());
        };
        Ok(vec![LogSource {
            source: LogSourceRef::new(LogSourceKind::Game, "latest.log")?,
            size: 1,
            modified: Some(modified.timestamp_millis()),
        }])
    }
}

/// The collaborators of the use cases, as the tests look at them.
pub(super) struct World {
    pub(super) storage: Arc<InMemoryProcessStorage>,
    pub(super) records: Arc<Records>,
    pub(super) probe: Arc<Probe>,
    pub(super) manage: Arc<Manage>,
    pub(super) instances: Arc<Instances>,
    pub(super) game_logs: Arc<GameLogs>,
    pub(super) emitter: SharedEventEmitter,
    pub(super) events: Mutex<mpsc::UnboundedReceiver<Event>>,
    pub(super) notices: Arc<TakeSkippedHookNoticesUseCase>,
}

impl World {
    pub(super) fn new(records: &[ProcessRecord], own: Option<OsProcessIdentity>) -> Self {
        let (sender, events) = mpsc::unbounded_channel();
        let emitter: SharedEventEmitter = Arc::new(ChannelEmitter(sender));
        let instances = Instances::new();
        let notices = Arc::new(TakeSkippedHookNoticesUseCase::new(
            emitter.clone(),
            instances.clone(),
        ));
        Self {
            storage: Arc::new(InMemoryProcessStorage::default()),
            records: Records::with(records),
            probe: Probe::new(own),
            manage: Arc::new(Manage::default()),
            instances,
            game_logs: Arc::new(GameLogs::default()),
            emitter,
            events: Mutex::new(events),
            notices,
        }
    }

    /// The process events sent so far.
    pub(super) fn process_events(&self) -> Vec<ProcessEvent> {
        let mut events = self.events.lock().unwrap();
        std::iter::from_fn(|| events.try_recv().ok())
            .filter_map(|event| match event {
                Event::Process(event) => Some(event),
                _ => None,
            })
            .collect()
    }

    pub(super) fn followed(&self) -> Vec<Followed> {
        self.manage.followed.lock().unwrap().clone()
    }
}

/// Waits until `done` holds; fails the test after a few seconds.
pub(super) async fn eventually(what: &str, mut done: impl FnMut() -> bool) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while !done() {
        assert!(tokio::time::Instant::now() < deadline, "{what}");
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}
