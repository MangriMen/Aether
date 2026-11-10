//! The user calling a run off from its bar: before the run starts on the target nothing is
//! left of it, after that it runs to its end.
use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::features::{
    events::{
        CancelState, Event, EventError, ProgressBarId, ProgressBarStorage, ProgressEvent,
        ProgressEventType, ProgressService, ProgressServiceImpl, ProgressStage, ProgressStatus,
        SharedDownload, SharedProgressWait, infra::InMemoryProgressBarStorage,
        test_support::ChannelEmitter,
    },
    instance::{
        ContentId, InstallFileEntry, InstallPlan, InstallTarget, InstanceError, InstanceId,
        InstanceName, NewInstance, ResolveTarget,
    },
    minecraft::{GameVersion, ModLoader},
};

use super::common::*;
use crate::features::instance::ResolveInstallPlanParams;

/// The real progress service, put into `fixture`, and what it stores and sends.
struct Bars {
    service: Arc<ProgressServiceImpl<InMemoryProgressBarStorage>>,
    storage: Arc<InMemoryProgressBarStorage>,
    events: mpsc::UnboundedReceiver<Event>,
}

impl Bars {
    fn install(fixture: &mut Fixture) -> Self {
        let (tx, events) = mpsc::unbounded_channel();
        let storage = Arc::new(InMemoryProgressBarStorage::default());
        let service = Arc::new(ProgressServiceImpl::new(
            Arc::new(ChannelEmitter(tx)),
            storage.clone(),
        ));
        fixture.progress_service = Some(service.clone());
        *fixture.create_instance.game_progress.lock().unwrap() = Some(service.clone());
        Self {
            service,
            storage,
            events,
        }
    }

    /// The bar of the run: the one at the top.
    async fn plan_bar(&self) -> Uuid {
        let roots: Vec<_> = self
            .storage
            .list()
            .await
            .into_iter()
            .filter(|bar| bar.parent_id.is_none())
            .collect();
        assert_eq!(roots.len(), 1, "{roots:?}");
        roots[0].id
    }

    /// The events sent so far.
    fn sent_so_far(&mut self) -> Vec<ProgressEvent> {
        let mut sent = Vec::new();
        while let Ok(event) = self.events.try_recv() {
            if let Event::Progress(event) = event {
                sent.push(event);
            }
        }
        sent
    }

    /// The events of every bar, by bar in the order sent, once no bar is left.
    async fn once_all_ended(&mut self) -> HashMap<Uuid, Vec<ProgressEvent>> {
        tokio::time::timeout(Duration::from_secs(5), async {
            while !self.storage.list().await.is_empty() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("bars were left behind");
        for _ in 0..10 {
            tokio::task::yield_now().await;
        }

        let mut by_bar: HashMap<Uuid, Vec<ProgressEvent>> = HashMap::new();
        while let Ok(event) = self.events.try_recv() {
            let Event::Progress(event) = event else {
                continue;
            };
            by_bar.entry(event.progress_bar_id).or_default().push(event);
        }
        by_bar
    }
}

/// The real progress service, with the user calling the run off right before its point of no
/// return, or right after it.
struct CallOffAtClose {
    inner: Arc<dyn ProgressService>,
    before: bool,
}

#[async_trait]
impl ProgressService for CallOffAtClose {
    async fn init_progress(
        &self,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<ProgressBarId, EventError> {
        self.inner.init_progress(event_type, total, message).await
    }
    async fn init_cancellable_progress(
        &self,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<(ProgressBarId, CancellationToken), EventError> {
        self.inner
            .init_cancellable_progress(event_type, total, message)
            .await
    }
    async fn cancel_progress(&self, progress_bar_id: Uuid) -> Result<(), EventError> {
        self.inner.cancel_progress(progress_bar_id).await
    }
    async fn close_cancellation(&self, progress_bar_id: &ProgressBarId) -> Result<(), EventError> {
        if self.before {
            self.inner.cancel_progress(progress_bar_id.id).await?;
            self.inner.close_cancellation(progress_bar_id).await
        } else {
            self.inner.close_cancellation(progress_bar_id).await?;
            self.inner.cancel_progress(progress_bar_id.id).await
        }
    }
    async fn init_child_progress(
        &self,
        parent_id: Uuid,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<ProgressBarId, EventError> {
        self.inner
            .init_child_progress(parent_id, event_type, total, message)
            .await
    }
    async fn init_shared_progress(
        &self,
        download: SharedDownload,
        total: f64,
        message: String,
    ) -> Result<ProgressBarId, EventError> {
        self.inner
            .init_shared_progress(download, total, message)
            .await
    }
    async fn wait_on_shared(
        &self,
        waiting: Option<&ProgressBarId>,
        download: SharedDownload,
    ) -> SharedProgressWait {
        self.inner.wait_on_shared(waiting, download).await
    }
    async fn init_or_edit_progress(
        &self,
        progress_bar_id: Option<ProgressBarId>,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<ProgressBarId, EventError> {
        self.inner
            .init_or_edit_progress(progress_bar_id, event_type, total, message)
            .await
    }
    async fn emit_progress(
        &self,
        progress_bar_id: &ProgressBarId,
        increment_frac: f64,
        message: Option<&str>,
    ) -> Result<(), EventError> {
        self.inner
            .emit_progress(progress_bar_id, increment_frac, message)
            .await
    }
    async fn edit_progress(
        &self,
        progress_bar_id: &ProgressBarId,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<(), EventError> {
        self.inner
            .edit_progress(progress_bar_id, event_type, total, message)
            .await
    }
    async fn set_progress_stage(
        &self,
        progress_bar_id: &ProgressBarId,
        stage: Option<ProgressStage>,
    ) -> Result<(), EventError> {
        self.inner.set_progress_stage(progress_bar_id, stage).await
    }
    async fn add_progress_bytes(
        &self,
        progress_bar_id: &ProgressBarId,
        bytes: u64,
    ) -> Result<(), EventError> {
        self.inner.add_progress_bytes(progress_bar_id, bytes).await
    }
    async fn add_progress_bytes_total(
        &self,
        progress_bar_id: &ProgressBarId,
        bytes: u64,
    ) -> Result<(), EventError> {
        self.inner
            .add_progress_bytes_total(progress_bar_id, bytes)
            .await
    }
}

impl Bars {
    /// Put the service into `fixture` behind a user calling the run off at its point of no
    /// return: right `before` it, or right after.
    fn call_off_at_close(&self, fixture: &mut Fixture, before: bool) {
        fixture.progress_service = Some(Arc::new(CallOffAtClose {
            inner: self.service.clone(),
            before,
        }));
    }
}

fn new_instance_plan(files: Vec<InstallFileEntry>) -> InstallPlan {
    InstallPlan {
        target: InstallTarget::New(NewInstance {
            name: InstanceName::parse("My Pack").unwrap(),
            game_version: GameVersion::from_static("1.20.1"),
            mod_loader: ModLoader::Vanilla,
            loader_version: None,
            icon_path: None,
            skip_install_instance: Some(true),
            pack_info: None,
        }),
        files,
        overrides: Vec::new(),
    }
}

/// The last event of the run's bar.
fn plan_end(bars: &HashMap<Uuid, Vec<ProgressEvent>>) -> &ProgressEvent {
    let roots: Vec<_> = bars
        .values()
        .filter(|events| events[0].parent_id.is_none())
        .collect();
    assert_eq!(roots.len(), 1, "{roots:?}");
    roots[0].last().unwrap()
}

#[tokio::test]
async fn should_create_no_instance_and_leave_no_files_when_cancelled_while_downloading() {
    let mut fixture = Fixture::new().await;
    let mut bars = Bars::install(&mut fixture);
    let packwiz = plugin("packwiz");
    let run = fixture.execute(
        // One file: a concurrent download's write could land after staging is removed.
        new_instance_plan(vec![direct(
            "https://cdn.example/hang.jar",
            "mods/hang.jar",
        )]),
        &packwiz,
    );
    tokio::pin!(run);
    tokio::select! {
        result = &mut run => panic!("the run cannot end by itself: {result:?}"),
        () = fixture.downloader.hanging.notified() => {}
    }

    let plan_bar = bars.plan_bar().await;
    bars.service.cancel_progress(plan_bar).await.unwrap();
    // A second press, before the run is through, changes nothing.
    bars.service.cancel_progress(plan_bar).await.unwrap();
    let err = tokio::time::timeout(Duration::from_secs(5), run)
        .await
        .expect("the run did not stop")
        .unwrap_err();

    assert!(matches!(err, InstanceError::Cancelled), "{err:?}");
    assert!(fixture.create_instance.created.lock().unwrap().is_empty());
    assert!(fixture.remove.removed.lock().unwrap().is_empty());
    assert_eq!(
        fixture.instance_files(NEW_INSTANCE_ID),
        Vec::<String>::new()
    );
    assert!(fixture.staging_is_clean());
    let bars = bars.once_all_ended().await;
    assert_eq!(plan_end(&bars).status, ProgressStatus::Cancelled);
    assert!(
        bars.values()
            .all(|events| events.last().unwrap().status == ProgressStatus::Cancelled),
        "{bars:?}"
    );
}

#[tokio::test]
async fn should_leave_the_instance_as_it_was_when_an_update_is_cancelled_while_downloading() {
    let mut fixture = Fixture::new().await;
    let mut bars = Bars::install(&mut fixture);
    let packwiz = plugin("packwiz");
    fixture
        .put_installed("mods/a.jar", "old", Some(pack_of(&packwiz)))
        .await;
    let records_before = format!("{:?}", fixture.records(INSTANCE_ID).await);
    let run = fixture.execute(
        existing(vec![direct("https://cdn.example/hang.jar", "mods/a.jar")]),
        &packwiz,
    );
    tokio::pin!(run);
    tokio::select! {
        result = &mut run => panic!("the run cannot end by itself: {result:?}"),
        () = fixture.downloader.hanging.notified() => {}
    }

    bars.service
        .cancel_progress(bars.plan_bar().await)
        .await
        .unwrap();
    let err = tokio::time::timeout(Duration::from_secs(5), run)
        .await
        .expect("the run did not stop")
        .unwrap_err();

    assert!(matches!(err, InstanceError::Cancelled), "{err:?}");
    assert_eq!(fixture.instance_files(INSTANCE_ID), ["mods/a.jar"]);
    assert_eq!(
        fixture.instance_file(INSTANCE_ID, "mods/a.jar").as_deref(),
        Some("old")
    );
    assert_eq!(
        format!("{:?}", fixture.records(INSTANCE_ID).await),
        records_before
    );
    assert!(fixture.backup_dirs().is_empty());
    assert!(fixture.staging_is_clean());
    assert_eq!(
        plan_end(&bars.once_all_ended().await).status,
        ProgressStatus::Cancelled
    );
}

#[tokio::test]
async fn should_create_no_instance_when_called_off_right_before_the_point_of_no_return() {
    let mut fixture = Fixture::new().await;
    let mut bars = Bars::install(&mut fixture);
    bars.call_off_at_close(&mut fixture, true);

    let err = fixture
        .execute(
            new_instance_plan(vec![direct("https://cdn.example/a.jar", "mods/a.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap_err();

    assert!(matches!(err, InstanceError::Cancelled), "{err:?}");
    assert!(fixture.create_instance.created.lock().unwrap().is_empty());
    assert_eq!(
        fixture.instance_files(NEW_INSTANCE_ID),
        Vec::<String>::new()
    );
    assert!(fixture.staging_is_clean());
    assert_eq!(
        plan_end(&bars.once_all_ended().await).status,
        ProgressStatus::Cancelled
    );
}

#[tokio::test]
async fn should_install_everything_when_called_off_past_the_point_of_no_return() {
    let mut fixture = Fixture::new().await;
    let mut bars = Bars::install(&mut fixture);
    bars.call_off_at_close(&mut fixture, false);

    fixture
        .execute(
            new_instance_plan(vec![direct("https://cdn.example/a.jar", "mods/a.jar")]),
            &plugin("packwiz"),
        )
        .await
        .unwrap();

    assert_eq!(fixture.create_instance.created.lock().unwrap().len(), 1);
    assert_eq!(fixture.instance_files(NEW_INSTANCE_ID), ["mods/a.jar"]);
    assert!(fixture.staging_is_clean());
    let bars = bars.once_all_ended().await;
    let end = plan_end(&bars);
    assert_eq!(end.status, ProgressStatus::Done);
    assert_eq!(end.cancel, Some(CancelState::Closed));
}

#[tokio::test]
async fn should_download_nothing_when_cancelled_while_resolving_refs() {
    let mut fixture = Fixture::new().await;
    let mut bars = Bars::install(&mut fixture);
    let modrinth = MockProvider::builtin_modrinth();
    fixture.register(&modrinth).await;
    let packwiz = plugin("packwiz");
    let run = fixture.execute(
        existing(vec![
            provider_ref("modrinth", "hang", "mods/hang.jar"),
            direct("https://cdn.example/a.jar", "mods/a.jar"),
        ]),
        &packwiz,
    );
    tokio::pin!(run);
    tokio::select! {
        result = &mut run => panic!("the run cannot end by itself: {result:?}"),
        () = modrinth.hanging.notified() => {}
    }
    // Listeners see the bar, and its cancel button, while the refs are resolved.
    let shown = bars.sent_so_far();
    assert!(
        shown.iter().any(|event| event.parent_id.is_none()
            && event.stage == Some(ProgressStage::Preparing)
            && event.cancel == Some(CancelState::Open)),
        "{shown:?}"
    );

    bars.service
        .cancel_progress(bars.plan_bar().await)
        .await
        .unwrap();
    let err = tokio::time::timeout(Duration::from_secs(5), run)
        .await
        .expect("the run did not stop")
        .unwrap_err();

    assert!(matches!(err, InstanceError::Cancelled), "{err:?}");
    assert!(fixture.downloaded_urls().is_empty());
    assert_eq!(fixture.instance_files(INSTANCE_ID), Vec::<String>::new());
    assert!(fixture.staging_is_clean());
    assert_eq!(
        plan_end(&bars.once_all_ended().await).status,
        ProgressStatus::Cancelled
    );
}

#[tokio::test]
async fn should_end_the_bar_cancelled_when_cancelled_while_the_plan_is_resolved() {
    let mut fixture = Fixture::new().await;
    let mut bars = Bars::install(&mut fixture);
    let executor = fixture.executor();
    let progress = executor
        .begin(&ResolveTarget::New, &ContentId::try_from("pack").unwrap())
        .await;
    // The archive of a pack is downloaded about now: listeners see the bar and its cancel button.
    let shown = bars.sent_so_far();
    assert!(
        shown.iter().any(|event| event.parent_id.is_none()
            && event.stage == Some(ProgressStage::Preparing)
            && event.cancel == Some(CancelState::Open)),
        "{shown:?}"
    );

    let result = {
        let resolving =
            progress.unless_cancelled(std::future::pending::<Result<(), InstanceError>>());
        tokio::pin!(resolving);
        tokio::select! {
            result = &mut resolving => panic!("nothing ends it: {result:?}"),
            () = tokio::task::yield_now() => {}
        }
        bars.service
            .cancel_progress(bars.plan_bar().await)
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(5), resolving)
            .await
            .expect("the resolving did not stop")
    };
    assert!(
        matches!(result, Err(InstanceError::Cancelled)),
        "{result:?}"
    );
    progress.end(&result).await;
    drop(progress);

    assert_eq!(
        plan_end(&bars.once_all_ended().await).status,
        ProgressStatus::Cancelled
    );
}

#[tokio::test]
async fn should_end_the_bar_once_when_the_run_and_its_caller_both_end_it() {
    let mut fixture = Fixture::new().await;
    let mut bars = Bars::install(&mut fixture);
    fixture.register(&MockProvider::builtin_modrinth()).await;
    let packwiz = plugin("packwiz");
    let executor = fixture.executor();
    let progress = executor
        .begin(
            &ResolveTarget::Existing {
                instance_id: InstanceId::try_from(INSTANCE_ID).unwrap(),
                game_version: GameVersion::from_static("1.21.1"),
                loader: Some(ModLoader::Fabric),
            },
            &ContentId::try_from("pack").unwrap(),
        )
        .await;

    let result = executor
        .execute_in(
            existing(vec![direct("https://cdn.example/a.jar", "mods/a.jar")]),
            pack_of(&packwiz),
            &progress,
        )
        .await;
    // What the caller does with whatever the run answered.
    progress.end(&Err::<(), _>(InstanceError::Cancelled)).await;
    drop(progress);

    result.unwrap();
    assert_eq!(
        plan_end(&bars.once_all_ended().await).status,
        ProgressStatus::Done
    );
}

#[tokio::test]
async fn should_count_the_bytes_of_the_bar_when_the_plan_is_resolved_with_a_download() {
    let mut fixture = Fixture::new().await;
    let bars = Bars::install(&mut fixture);
    let provider = MockProvider::builtin_modrinth();
    let executor = fixture.executor();
    let progress = executor
        .begin(&ResolveTarget::New, &ContentId::try_from("pack").unwrap())
        .await;

    progress
        .resolve_plan(
            provider.as_ref(),
            ResolveInstallPlanParams {
                content_id: ContentId::try_from("pack").unwrap(),
                version: None,
                installed_version: None,
                target: ResolveTarget::New,
            },
        )
        .await
        .unwrap();

    let bar = bars.storage.get(bars.plan_bar().await).await.unwrap();
    assert_eq!((bar.bytes_done, bar.bytes_total), (Some(100), Some(100)));
}

#[tokio::test]
async fn should_say_a_plugin_does_the_work_and_stop_the_cancel_when_the_run_is_handed_over() {
    let mut fixture = Fixture::new().await;
    let mut bars = Bars::install(&mut fixture);
    let executor = fixture.executor();
    let progress = executor
        .begin(&ResolveTarget::New, &ContentId::try_from("pack").unwrap())
        .await;

    progress.hand_over_to_plugin().await;

    let shown = bars.sent_so_far();
    let last = shown.last().expect("the bar said nothing");
    assert_eq!(last.stage, Some(ProgressStage::ByPlugin), "{shown:?}");
    assert_eq!(last.cancel, Some(CancelState::Closed), "{shown:?}");
}
