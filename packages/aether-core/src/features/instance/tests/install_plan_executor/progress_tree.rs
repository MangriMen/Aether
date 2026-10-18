//! The tree of bars a run reports on, with the real progress service: the plan's bar, with one
//! under it per file downloaded and one for the game a new instance installs.

use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::features::{
    events::{
        Event, EventEmitter, EventError, ProgressBarStorage, ProgressEvent, ProgressEventType,
        ProgressServiceImpl, ProgressStatus, infra::InMemoryProgressBarStorage,
    },
    instance::{InstallPlan, InstallTarget, NewInstance},
    minecraft::ModLoader,
};

use super::common::*;

struct ChannelEmitter(mpsc::UnboundedSender<Event>);

#[async_trait]
impl EventEmitter<Event> for ChannelEmitter {
    async fn emit(&self, event: Event) -> Result<(), EventError> {
        let _ = self.0.send(event);
        Ok(())
    }

    fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
}

/// The real progress service, put into `fixture`, and what it stores and sends.
struct Bars {
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
        *fixture.create_instance.game_progress.lock().unwrap() = Some(service);
        Self { storage, events }
    }

    /// The events of every bar, by bar in the order sent, once no bar is left: a bar left
    /// behind fails the test.
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

/// The one bar of `bars` that has no parent.
fn root(bars: &HashMap<Uuid, Vec<ProgressEvent>>) -> &[ProgressEvent] {
    let roots: Vec<_> = bars
        .values()
        .filter(|events| events[0].parent_id.is_none())
        .collect();
    assert_eq!(roots.len(), 1, "{roots:?}");
    roots[0]
}

fn new_instance_plan(files: Vec<crate::features::instance::InstallFileEntry>) -> InstallPlan {
    InstallPlan {
        target: InstallTarget::New(NewInstance {
            name: "My Pack".into(),
            game_version: "1.20.1".into(),
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

#[tokio::test]
async fn should_report_one_tree_when_a_pack_is_installed_into_a_new_instance() {
    let mut fixture = Fixture::new().await;
    let mut bars = Bars::install(&mut fixture);
    let big_url = "https://cdn.example/chunks/big.jar";
    let small_url = "https://cdn.example/pack/small.jar";
    let mut big = direct(big_url, "mods/big.jar");
    big.size = Some(big_url.len() as u64);
    let mut small = direct(small_url, "mods/small.jar");
    small.size = Some(small_url.len() as u64);

    fixture
        .execute(new_instance_plan(vec![big, small]), &plugin("packwiz"))
        .await
        .unwrap();

    let bars = bars.once_all_ended().await;
    let pack = root(&bars);
    let pack_id = pack[0].progress_bar_id;
    assert!(matches!(
        pack[0].event,
        ProgressEventType::PackDownload { .. }
    ));

    let mut files = Vec::new();
    let mut games = 0;
    for events in bars.values().filter(|events| events[0].parent_id.is_some()) {
        assert!(
            events.iter().all(|event| event.parent_id == Some(pack_id)),
            "{events:?}"
        );
        let last = events.last().unwrap();
        assert_eq!((last.fraction, &last.status), (None, &ProgressStatus::Done));
        match &events[0].event {
            ProgressEventType::PackFileDownload { pack_name, .. } => {
                files.push((pack_name.clone(), last.bytes_done, last.bytes_total));
            }
            ProgressEventType::MinecraftDownload { .. } => games += 1,
            other => panic!("unexpected bar {other:?}"),
        }
    }
    files.sort();
    let (big_len, small_len) = (big_url.len() as u64, small_url.len() as u64);
    assert_eq!(
        files,
        [
            ("big.jar".to_owned(), Some(big_len), Some(big_len)),
            ("small.jar".to_owned(), Some(small_len), Some(small_len)),
        ]
    );
    assert_eq!(games, 1, "the game goes under the pack too");

    // The pack's bytes are those of its files.
    let last = pack.last().unwrap();
    assert_eq!((last.fraction, &last.status), (None, &ProgressStatus::Done));
    assert_eq!(
        (last.bytes_done, last.bytes_total),
        (Some(big_len + small_len), Some(big_len + small_len))
    );
}

#[tokio::test]
async fn should_leave_no_bars_and_fail_the_packs_when_a_download_fails() {
    let mut fixture = Fixture::new().await;
    let mut bars = Bars::install(&mut fixture);

    let result = fixture
        .execute(
            existing(vec![
                direct("https://cdn.example/pack/a.jar", "mods/a.jar"),
                direct("https://cdn.example/fail/b.jar", "mods/b.jar"),
            ]),
            &plugin("packwiz"),
        )
        .await;
    assert!(result.is_err());

    let bars = bars.once_all_ended().await;
    let pack = root(&bars).last().unwrap();
    assert!(
        matches!(pack.status, ProgressStatus::Failed { .. }),
        "{pack:?}"
    );
    assert_eq!(pack.fraction, None);

    let failed_file = bars
        .values()
        .map(|events| events.last().unwrap())
        .find(|last| {
            matches!(&last.event, ProgressEventType::PackFileDownload { pack_name, .. } if pack_name == "b.jar")
        })
        .expect("the failed file had a bar");
    assert!(
        matches!(failed_file.status, ProgressStatus::Failed { .. }),
        "{failed_file:?}"
    );
    // Every bar ended, whether it finished, failed or was cut short.
    assert!(
        bars.values()
            .all(|events| events.last().unwrap().fraction.is_none())
    );
}
