//! Turns an [`InstallPlan`] into files in an instance.
//!
//! Phases: check the plan as data ([`select_files`], paths, overrides, who may change the game);
//! resolve every [`ContentSource::ProviderRef`](crate::features::instance::ContentSource::ProviderRef)
//! through the provider chosen for its platform ([`resolve::choose_provider`]) against the game
//! the target runs once the plan is applied; diff against the instance
//! ([`diff_against_installed`]); download and verify into staging; create or prepare
//! the target; move the files in and record them under the plan's [`PlanOrigin`]; remove the
//! dropped files; apply the overrides.
//!
//! Invariants:
//!
//! * Nothing is written into an instance, nor is one created or moved to another game, before
//!   every file has been resolved, downloaded and verified: a wrong hash or a refused host leaves
//!   the instance as it was. A new instance that fails after it was created is removed again.
//! * An existing instance that fails after that is rolled back ([`Undo`]): every file written
//!   over or removed is first moved aside, so the files, their records and the game are put back
//!   as they were, and the caller may report the instance in the stage it had. Rolling back
//!   rather than marking the instance for repair keeps it launchable, and a plan that keeps
//!   failing (a file held open) cannot lock it out. Should putting a file back fail too, the
//!   backups are kept in the temp directory and the failure is logged.
//! * A run's directories in the temp directory go with it, even when its future is dropped
//!   halfway ([`PlanDirs`]); what a crash leaves is removed on the next start
//!   ([`remove_plan_leftovers`]). Backups holding files are kept either way: they may be the
//!   only copy of a file the run moved aside.
//! * Dropped files are removed only once the new ones are in and recorded, and a record goes
//!   only with its file: a run killed before it could roll back leaves at worst a stale file
//!   still recorded as the origin's, which the next run of the plan removes.
//! * Only a plan of the pack the instance was installed from may change its game
//!   ([`InstallTarget::Existing::game`](InstallTarget)). A failed update moves the instance back
//!   to its old game, which stays installed.
//!
//! # What an update may touch
//!
//! * **Removed**: a file recorded as installed by the same origin ([`installed_state::is_installed_by`]) whose path
//!   the plan no longer has, and another version of a content the plan installs, whoever
//!   installed it. Files added by hand or by another plan are otherwise never removed.
//! * **Kept**: a planned path already holding the plan's bytes, checked on disk against the plan's
//!   hash (not the possibly stale record). A planned file without a hash is downloaded again.
//! * **Written over**: a planned path holding other bytes; the plan claims its paths.
//! * **Disabled** files stay disabled (`<path>.disabled`), and a new version of a disabled content
//!   under a new path is installed disabled too.
//!
//! An instance never holds two versions of one content: a plan of a content replaces the pack's
//! copy of it and owns the file from then on, while a pack's update leaves out its file of a
//! content the user installed on its own. A file without a recognised content is never taken for
//! another's version.
//!
//! # Overrides
//!
//! Files a pack ships as they are ([`plan_overrides`]). They pass the same path checks as planned
//! files, and an archive entry escaping the archive refuses the whole plan before any download.
//! Server sources are skipped, later sources win over earlier ones, and planned files win over
//! overrides. Their count and bytes are bounded over the whole plan, the bytes counted while
//! staging so an archive declaring less than it holds is still refused. Overrides overwrite their
//! paths on every update and are not recorded under the plan's origin, so no update removes them.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt, future::try_join_all};

mod checks;
mod installed_state;
mod resolve;
mod rollback;
mod staging;

use super::plan_overrides::{self, OverrideFile, StagedOverride};
use crate::shared::limits::ContentInstallLimits;
use crate::{
    features::{
        events::{
            CancellableBar, ProgressBarId, ProgressEventType, ProgressService, ProgressServiceExt,
            ProgressStage,
        },
        instance::{
            ContentDownloader, ContentFileService, ContentFileUpdateInfo, ContentProvider,
            FileSide, InstallPlan, InstallTarget, Instance, InstanceError, InstanceId,
            InstanceStorage, OverrideLocation, PackFile, PackStorage, PlanOrigin,
            PlatformProviderSelection, ProviderId,
            app::{
                DISABLED_SUFFIX,
                ports::{
                    ChangeInstanceGameUseCasePort, ContentMetadataCache, CreateInstanceUseCasePort,
                    InstallPlanExecutorPort, RemoveInstanceUseCasePort,
                },
            },
        },
        minecraft::ModLoader,
        settings::LocationInfo,
    },
    shared::{
        capability::domain::CapabilityRegistry,
        hash::{FileHash, HashAlgorithm, infra::hash_file_hex_async},
    },
};
pub(super) use installed_state::is_pack_of;
use installed_state::{
    diff_against_installed, dropped_paths, select_files, without_separate_installs,
};
pub use staging::remove_plan_leftovers;
use staging::{PlanDirs, remove_staging};

/// How many files on disk are hashed at once while diffing.
const CONCURRENT_HASHES: usize = 8;

pub struct InstallPlanExecutor {
    provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
    platform_selection: Arc<dyn PlatformProviderSelection>,
    instance_storage: Arc<dyn InstanceStorage>,
    create_instance: Arc<dyn CreateInstanceUseCasePort>,
    change_instance_game: Arc<dyn ChangeInstanceGameUseCasePort>,
    content_downloader: Arc<dyn ContentDownloader>,
    content_file_service: Arc<dyn ContentFileService>,
    pack_storage: Arc<dyn PackStorage>,
    location_info: Arc<LocationInfo>,
    progress_service: Arc<dyn ProgressService>,
    remove_instance: Arc<dyn RemoveInstanceUseCasePort>,
    metadata_cache: Arc<dyn ContentMetadataCache>,
    limits: ContentInstallLimits,
}

/// The progress bar of one run, counted in steps, with a bar under it for each file downloaded.
/// The install of the game of a new instance (or of a new game) goes under it too, with mirrors
/// of the shared downloads it waits on (its Java, its game version). Dropping it completes the
/// bar.
struct PlanProgress<'a> {
    service: &'a dyn ProgressService,
    /// `None` when the bar could not be created: the plan runs without progress.
    bar: Option<ProgressBarId>,
    /// Shared by the bars of the files.
    instance_path: String,
}

impl PlanProgress<'_> {
    async fn step(&self, message: &str) {
        self.advance(1.0, message).await;
    }

    /// Move the bar by part of a step.
    async fn advance(&self, steps: f64, message: &str) {
        if let Some(bar) = &self.bar {
            self.service
                .emit_progress_safe(bar, steps, Some(message))
                .await;
        }
    }

    /// Take the last step, or mark the bar as failed, by the run's `result`. Called once the run
    /// is over (every download stopped, every file's bar gone), so that no update writes over
    /// the failure.
    async fn end(&self, result: &Result<InstanceId, InstanceError>) {
        match (result, &self.bar) {
            (Ok(_), _) => self.step("Complete").await,
            (Err(err), Some(bar)) => bar.fail(err.to_string()).await,
            (Err(_), None) => {}
        }
    }

    /// Start the bar of a file to download, captioned with its file name and expecting its size
    /// when that is known. The plan's bar counts the file's bytes through it.
    async fn start_file(&self, entry: &DownloadEntry) -> FileProgress<'_, '_> {
        let bar = match &self.bar {
            Some(parent) => {
                self.service
                    .init_child_progress_safe(
                        parent.id,
                        ProgressEventType::PackFileDownload {
                            instance_path: self.instance_path.clone(),
                            pack_name: file_name_of(&entry.relative_path).to_owned(),
                            icon: None,
                            pack_version: entry
                                .resolved
                                .as_ref()
                                .map(|resolved| resolved.version.clone())
                                .unwrap_or_default(),
                        },
                        1.0,
                        "Downloading".to_owned(),
                    )
                    .await
            }
            None => None,
        };

        let mut file = FileProgress {
            plan: self,
            bar: CancellableBar::new(bar),
            announced: 0,
        };
        if let Some(size) = entry.size.filter(|size| *size > 0) {
            file.add_bytes_total(size).await;
            file.announced = size;
        }
        file.set_stage(ProgressStage::Downloading).await;
        file
    }
}

/// The bar of one file to download, under the plan's bar. Settled by the result of the file
/// ([`CancellableBar::settle`]); dropped before that — cut short by another file's failure — it
/// ends cancelled.
struct FileProgress<'p, 's> {
    plan: &'p PlanProgress<'s>,
    /// Without a bar (it could not be created) the bytes go to the plan's bar directly.
    bar: CancellableBar,
    /// The bytes the file was expected to have.
    announced: u64,
}

impl FileProgress<'_, '_> {
    /// Where the bytes of the file are counted: its own bar, which passes them on to the plan's.
    fn bytes_bar(&self) -> Option<&ProgressBarId> {
        self.bar.bar().or(self.plan.bar.as_ref())
    }

    /// Count `bytes` more as downloaded, for the bytes and speed of the file and the plan.
    async fn add_bytes(&self, bytes: u64) {
        if let Some(bar) = self.bytes_bar() {
            self.plan.service.add_progress_bytes_safe(bar, bytes).await;
        }
    }

    async fn add_bytes_total(&self, bytes: u64) {
        if let Some(bar) = self.bytes_bar() {
            self.plan
                .service
                .add_progress_bytes_total_safe(bar, bytes)
                .await;
        }
    }

    /// Move the file's bar by `share` of the file, and the plan's by as much of its step.
    async fn advance(&self, share: f64) {
        if let Some(bar) = self.bar.bar() {
            self.plan.service.emit_progress_safe(bar, share, None).await;
        }
        self.plan.advance(share, "Downloading").await;
    }

    async fn set_stage(&self, stage: ProgressStage) {
        if let Some(bar) = self.bar.bar() {
            self.plan
                .service
                .set_progress_stage_safe(bar, Some(stage))
                .await;
        }
    }

    /// Download one file, spreading its *Downloading* step over the bytes as they arrive, so the
    /// bar moves inside a large file. The step adds up to exactly 1 once the file is done:
    /// bytes beyond the expected size move nothing, and a file of unknown size moves the bar
    /// only at its end.
    ///
    /// The bytes expected grow to the bytes downloaded once the file is in, when it had more
    /// than it was expected to (or no known size at all), so that the plan's bar does not end
    /// with more bytes done than expected.
    ///
    /// Every byte is spent from the plan's `budget` as it arrives; the download is dropped the
    /// moment the plan goes over it.
    async fn download(
        &self,
        downloader: &dyn ContentDownloader,
        entry: &DownloadEntry,
        destination: &Path,
        budget: &PlanBudget,
    ) -> Result<u64, InstanceError> {
        let (sender, mut received) = tokio::sync::mpsc::unbounded_channel::<u64>();
        let on_bytes = move |bytes: u64| {
            let _ = sender.send(bytes);
        };

        let transfer = async {
            let result = downloader
                .download(
                    &entry.origin,
                    &entry.url,
                    entry.hash.as_ref(),
                    destination,
                    Some(&on_bytes),
                )
                .await;
            // Closes the channel, which ends the reporting below.
            drop(on_bytes);
            result
        };
        let reporting = async {
            let mut reported = 0.0_f64;
            let mut pending = 0_u64;
            let mut counted = 0_u64;

            while let Some(bytes) = received.recv().await {
                counted += bytes;
                budget.spend(bytes, &entry.relative_path)?;
                self.add_bytes(bytes).await;

                let Some(expected) = entry.size.filter(|size| *size > 0) else {
                    continue;
                };
                pending += bytes;

                // A tenth of a percent of the file at a time: a bar does not need more, and
                // every report is an event for the frontend.
                #[allow(clippy::cast_precision_loss)] // a progress fraction
                let fraction = pending as f64 / expected as f64;
                if fraction >= 0.001 {
                    let share = fraction.min(1.0 - reported);
                    pending = 0;
                    reported += share;
                    self.advance(share).await;
                }
            }

            Ok((reported, counted))
        };

        // An error of either ends both: a download over the budget is not finished.
        let (size, (reported, counted)) = futures::try_join!(transfer, reporting)?;
        // A downloader that does not report its bytes is counted by the size it returns.
        let uncounted = size.saturating_sub(counted);
        budget.spend(uncounted, &entry.relative_path)?;
        if uncounted > 0 {
            self.add_bytes(uncounted).await;
        }
        let transferred = size.max(counted);
        if transferred > self.announced {
            self.add_bytes_total(transferred - self.announced).await;
        }

        // What the stream did not account for — all of it when the size was unknown.
        let rest = 1.0 - reported;
        if rest > 1e-9 {
            self.advance(rest).await;
        }

        Ok(size)
    }
}

/// The bytes the downloads of one plan may bring in together
/// ([`ContentInstallLimits::max_plan_bytes`]), spent by all of them at once.
struct PlanBudget {
    spent: AtomicU64,
    limit: u64,
}

impl PlanBudget {
    fn new(limit: u64) -> Self {
        Self {
            spent: AtomicU64::new(0),
            limit,
        }
    }

    /// Spend `bytes` of the file at `relative_path`; past the limit, the plan is refused.
    fn spend(&self, bytes: u64, relative_path: &str) -> Result<(), InstanceError> {
        let spent = self
            .spent
            .fetch_add(bytes, Ordering::Relaxed)
            .saturating_add(bytes);
        if spent > self.limit {
            return Err(InstanceError::FileTooLarge {
                path: relative_path.to_owned(),
                size: spent,
                limit: self.limit,
            });
        }
        Ok(())
    }
}

/// What a ref is resolved against: the game version and loader the target runs, or will run.
struct TargetGame {
    game_version: String,
    /// `None` for vanilla.
    loader: Option<ModLoader>,
}

impl TargetGame {
    fn new(game_version: String, loader: ModLoader) -> Self {
        Self {
            game_version,
            loader: (loader != ModLoader::Vanilla).then_some(loader),
        }
    }
}

/// A planned file once its bytes are known: where they come from, and whose word that is.
#[derive(Debug, Clone, PartialEq)]
struct DownloadEntry {
    relative_path: String,
    url: String,
    hash: Option<FileHash>,
    /// The provider that supplied `url`: the plan's for a direct source, the resolving
    /// provider's for a ref.
    origin: ProviderId,
    /// Expected size in bytes, when known: what progress inside the file is measured against.
    size: Option<u64>,
    /// What a ref resolved to, or the identity a direct source carries; `None` for a direct
    /// source without one.
    resolved: Option<ResolvedRef>,
    /// Recorded with the file: the plan entry's name, or the one its resolution gave.
    name: Option<String>,
    side: FileSide,
}

/// The platform's ids of the file a ref resolved to (or a direct source names), under the
/// provider serving that platform: what the file is recorded with as its update info, and how a
/// new version of a disabled file is recognised.
#[derive(Debug, Clone, PartialEq)]
struct ResolvedRef {
    provider_id: ProviderId,
    content_id: String,
    version: String,
}

impl ResolvedRef {
    /// Whether `record`'s update info names the same content.
    fn is_content_of(&self, record: &PackFile) -> bool {
        record
            .update
            .as_ref()
            .and_then(|update| update.get(&self.provider_id))
            .is_some_and(|info| info.content_id == self.content_id)
    }

    /// Whether `record` is this content as the user installed it on its own: recorded as
    /// installed by a plan of this very content.
    fn is_separate_install(&self, record: &PackFile) -> bool {
        self.is_content_of(record)
            && record
                .installed_by
                .as_ref()
                .is_some_and(|owner| owner.content_id == self.content_id)
    }
}

/// A planned file with where it lands.
#[derive(Debug, Clone, PartialEq)]
struct PlannedFile {
    entry: DownloadEntry,
    /// Kept as `<path>.disabled`: the user has it disabled.
    disabled: bool,
}

impl PlannedFile {
    fn install_path(&self) -> String {
        if self.disabled {
            format!("{}{DISABLED_SUFFIX}", self.entry.relative_path)
        } else {
            self.entry.relative_path.clone()
        }
    }
}

/// A planned file that is already in the instance.
#[derive(Debug, Clone, PartialEq)]
struct UnchangedFile {
    file: PlannedFile,
    /// The file on disk, to record its sha1 from.
    path: PathBuf,
}

/// The plan against what the instance has.
#[derive(Debug, Default, PartialEq)]
struct InstallDiff {
    /// Already in the instance byte for byte: not downloaded again, only recorded.
    unchanged: Vec<UnchangedFile>,
    to_download: Vec<PlannedFile>,
    /// Content paths of the files the plan's origin installed and the plan no longer has, and of
    /// the other versions of the contents it installs.
    to_remove: Vec<String>,
}

/// What an instance has, as far as one plan is concerned.
#[derive(Debug, Default)]
struct InstalledState {
    /// Every recorded file, by content path.
    records: HashMap<String, PackFile>,
    /// The planned paths that hold a file.
    planned: HashMap<String, PlannedOnDisk>,
    /// The files of the plan's origin whose paths the plan no longer has, and the other versions
    /// of the contents it installs ([`dropped_paths`]).
    dropped: Vec<DroppedFile>,
}

/// The file on disk at a planned path.
#[derive(Debug, Clone)]
struct PlannedOnDisk {
    path: PathBuf,
    disabled: bool,
    /// Whether it is the planned file: it hashes to the plan's hash. `false` without one.
    matches: bool,
}

/// A file of the plan's origin the plan no longer has, or another version of a content the plan
/// installs.
#[derive(Debug, Clone)]
struct DroppedFile {
    content_path: String,
    record: PackFile,
    /// Disabled on disk. `false` when the file is gone.
    disabled: bool,
}

/// A downloaded, verified file waiting in the staging directory.
struct StagedFile {
    file: PlannedFile,
    path: PathBuf,
    /// What the pack storage records it under ([`sha1_of`]).
    sha1: String,
}

/// The overrides of a plan for this side, source by source in the order they are applied,
/// without those a planned file or a later source takes the place of.
struct PlanOverrides {
    sources: Vec<(OverrideLocation, Vec<OverrideFile>)>,
}

impl PlanOverrides {
    fn file_count(&self) -> usize {
        self.sources.iter().map(|(_, files)| files.len()).sum()
    }
}

/// A planned file now in the instance, as it is recorded.
struct InstalledFile {
    entry: DownloadEntry,
    sha1: String,
}

/// One change a run made to the files of an instance, by the exact path on disk (`.disabled`
/// included).
enum FileChange {
    /// A file was put where there was none.
    Created { install_path: String },
    /// What was at the path was moved to `backup` before being written over or removed.
    MovedAside {
        install_path: String,
        backup: PathBuf,
    },
}

/// What a run changed in an instance, kept until it is through so that a failure can put an
/// existing instance back as it was.
struct Undo {
    /// Where the files moved aside wait. Kept when they could not all be put back.
    backup_dir: PathBuf,
    /// In the order made; undone in reverse, so a path changed twice ends up as it first was.
    files: Vec<FileChange>,
    /// The content paths whose records the run wrote or removed.
    records: Vec<String>,
    /// The instance as it was before the plan moved it to another game.
    game_before: Option<Instance>,
}

impl FileChange {
    /// What undoing the change is, for the log: a file moved aside names its backup together
    /// with the path it goes back to, so that the only copy of it can be matched with its place
    /// by hand when putting it back failed.
    fn restore_description(&self) -> String {
        match self {
            Self::Created { install_path } => {
                format!("take out the new \"{install_path}\"")
            }
            Self::MovedAside {
                install_path,
                backup,
            } => format!(
                "put \"{install_path}\" back from its backup \"{}\"",
                backup.display()
            ),
        }
    }
}

impl InstallPlanExecutor {
    // One port per collaborator, wired once in the DI container; a parameter struct would only
    // repeat the field list.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
        platform_selection: Arc<dyn PlatformProviderSelection>,
        instance_storage: Arc<dyn InstanceStorage>,
        create_instance: Arc<dyn CreateInstanceUseCasePort>,
        change_instance_game: Arc<dyn ChangeInstanceGameUseCasePort>,
        content_downloader: Arc<dyn ContentDownloader>,
        content_file_service: Arc<dyn ContentFileService>,
        pack_storage: Arc<dyn PackStorage>,
        location_info: Arc<LocationInfo>,
        progress_service: Arc<dyn ProgressService>,
        remove_instance: Arc<dyn RemoveInstanceUseCasePort>,
        metadata_cache: Arc<dyn ContentMetadataCache>,
        limits: ContentInstallLimits,
    ) -> Self {
        Self {
            provider_registry,
            platform_selection,
            instance_storage,
            create_instance,
            change_instance_game,
            content_downloader,
            content_file_service,
            pack_storage,
            location_info,
            progress_service,
            remove_instance,
            metadata_cache,
            limits,
        }
    }

    pub async fn execute(
        &self,
        plan: InstallPlan,
        origin: PlanOrigin,
    ) -> Result<InstanceId, InstanceError> {
        let InstallPlan {
            target,
            files,
            overrides,
        } = plan;

        self.check_file_count(files.len())?;
        let files = select_files(files);
        self.check_paths(&files)?;
        let game = self.target_game(&target, &origin).await?;

        let overrides = self.check_overrides(overrides, &files).await?;

        let entries = self
            .resolve_all_refs(files, &origin.provider_id, &game)
            .await?;

        let records = self.records_of(&target).await?;
        let entries = without_separate_installs(entries, &records, &origin);
        let installed = self
            .installed_state(&target, &origin, records, &entries)
            .await?;
        let diff = diff_against_installed(entries, &installed);

        let progress = self
            .start_progress(&target, &origin, &diff.to_download, &overrides)
            .await;

        let dirs = PlanDirs::new(&self.location_info.temp_dir());
        let staging = dirs.staging.clone();
        let mut undo = Undo {
            backup_dir: dirs.backup_dir.clone(),
            files: Vec::new(),
            records: Vec::new(),
            game_before: None,
        };
        let mut keep_backups = false;

        let result = async {
            let staged = self
                .download_all(diff.to_download, &staging, &progress)
                .await?;
            let staged_overrides = plan_overrides::stage_overrides(
                overrides.sources,
                &staging.join("overrides"),
                self.limits.max_override_bytes,
            )
            .await?;
            let created = matches!(target, InstallTarget::New(_));
            let (instance_id, game_before) = self.prepare_target(target, &progress).await?;
            undo.game_before = game_before;

            let filled = async {
                crate::shared::io::infra::create_dir_all(&undo.backup_dir)
                    .await
                    .map_err(|err| InstanceError::Storage(err.to_string()))?;

                let mut installed_files = self
                    .install_all(&instance_id, staged, &progress, &mut undo)
                    .await?;
                for unchanged in diff.unchanged {
                    let sha1 = sha1_of(&unchanged.path, unchanged.file.entry.hash.as_ref()).await?;
                    installed_files.push(InstalledFile {
                        entry: unchanged.file.entry,
                        sha1,
                    });
                }

                self.record_installed(
                    &instance_id,
                    &origin,
                    &installed.records,
                    installed_files,
                    &mut undo,
                )
                .await?;
                self.remove_dropped(&instance_id, &diff.to_remove, &mut undo)
                    .await?;
                self.apply_overrides(&instance_id, staged_overrides, &progress, &mut undo)
                    .await
            }
            .await;

            if let Err(err) = filled {
                if created {
                    self.discard_created(&instance_id, &err).await;
                } else {
                    keep_backups = !self
                        .roll_back(&instance_id, &installed.records, &undo, &err)
                        .await;
                }
                return Err(err);
            }

            Ok(instance_id)
        }
        .await;
        progress.end(&result).await;

        // Removed here, retried, rather than by the guard.
        dirs.disarm();
        // Whatever is left there was never moved into the instance.
        remove_staging(&staging).await;
        if keep_backups {
            tracing::error!(
                "Files that could not be put back are kept in \"{}\"; the failures logged above \
                 name the backup of each",
                undo.backup_dir.display()
            );
        } else {
            remove_staging(&undo.backup_dir).await;
        }

        result
    }

    /// Start the bar: three steps per file to download (download, hash, install), one per
    /// override, one to finish. Its bytes are those of the bars under it
    /// ([`PlanProgress::start_file`]).
    async fn start_progress<'a>(
        &'a self,
        target: &InstallTarget,
        origin: &PlanOrigin,
        to_download: &[PlannedFile],
        overrides: &PlanOverrides,
    ) -> PlanProgress<'a> {
        let override_count = overrides.file_count();
        #[allow(clippy::cast_precision_loss)] // a count of files
        let total = (3 * to_download.len() + override_count + 1) as f64;
        let instance_path = match target {
            InstallTarget::Existing { instance_id, .. } => instance_id.to_string(),
            InstallTarget::New(_) => String::new(),
        };
        // What the bar is captioned with: the instance the plan fills.
        let pack_name = match target {
            InstallTarget::Existing { instance_id, .. } => {
                self.instance_storage.get(instance_id).await.map_or_else(
                    |_: InstanceError| origin.content_id.clone(),
                    |instance| instance.name().to_owned(),
                )
            }
            InstallTarget::New(new_instance) => new_instance.name.clone(),
        };

        let bar = self
            .progress_service
            .init_progress_safe(
                ProgressEventType::PackDownload {
                    instance_path: instance_path.clone(),
                    pack_name,
                    icon: None,
                    pack_id: Some(origin.content_id.clone()),
                    pack_version: None,
                },
                total,
                "Downloading".to_owned(),
            )
            .await;

        PlanProgress {
            service: self.progress_service.as_ref(),
            bar,
            instance_path,
        }
    }

    /// Every recorded file of the target, by content path. A new instance has none.
    async fn records_of(
        &self,
        target: &InstallTarget,
    ) -> Result<HashMap<String, PackFile>, InstanceError> {
        match target {
            InstallTarget::Existing { instance_id, .. } => {
                self.pack_storage.get_pack_files(instance_id).await
            }
            InstallTarget::New(_) => Ok(HashMap::new()),
        }
    }

    /// Read what the target instance has that the plan cares about: besides its `records`, the
    /// files at the planned paths (hashed against the plan) and the recorded files the plan
    /// drops. A new instance has nothing.
    async fn installed_state(
        &self,
        target: &InstallTarget,
        origin: &PlanOrigin,
        records: HashMap<String, PackFile>,
        entries: &[DownloadEntry],
    ) -> Result<InstalledState, InstanceError> {
        let InstallTarget::Existing { instance_id, .. } = target else {
            return Ok(InstalledState::default());
        };

        // The futures are collected first: a closure kept inside the stream would make the whole
        // `execute` future fail to be `Send` (rust-lang/rust#102211).
        let lookups: Vec<_> = entries
            .iter()
            .map(|entry| self.planned_on_disk(instance_id, entry))
            .collect();
        let found: Vec<_> = futures::stream::iter(lookups)
            .buffer_unordered(CONCURRENT_HASHES)
            .try_collect()
            .await?;
        let planned = found.into_iter().flatten().collect();

        let mut dropped = Vec::new();
        for content_path in dropped_paths(&records, origin, entries) {
            // A record is the storage's word: one at a path no file of the instance can have is
            // left alone, rather than failing every update of the pack.
            if let Err(err) = self.content_file_service.check_content_path(&content_path) {
                tracing::warn!(
                    "Not removing \"{content_path}\" of \"{instance_id}\": {err}. Its record is kept"
                );
                continue;
            }
            dropped.push(DroppedFile {
                disabled: self.is_disabled(instance_id, &content_path).await,
                record: records[&content_path].clone(),
                content_path,
            });
        }

        Ok(InstalledState {
            records,
            planned,
            dropped,
        })
    }

    async fn planned_on_disk(
        &self,
        instance_id: &InstanceId,
        entry: &DownloadEntry,
    ) -> Result<Option<(String, PlannedOnDisk)>, InstanceError> {
        let Some(located) = self
            .content_file_service
            .locate_content_file(instance_id, &entry.relative_path)
            .await?
        else {
            return Ok(None);
        };

        let matches = match &entry.hash {
            Some(hash) => hash.matches(
                &hash_file_hex_async(hash.algorithm(), &located.path)
                    .await
                    .map_err(|err| InstanceError::Storage(err.to_string()))?,
            ),
            None => false,
        };

        Ok(Some((
            entry.relative_path.clone(),
            PlannedOnDisk {
                path: located.path,
                disabled: located.disabled,
                matches,
            },
        )))
    }

    /// Whether the file of a dropped record is disabled on disk. A path that cannot be looked at
    /// counts as enabled: all this decides is whether a new version starts disabled.
    async fn is_disabled(&self, instance_id: &InstanceId, content_path: &str) -> bool {
        match self
            .content_file_service
            .locate_content_file(instance_id, content_path)
            .await
        {
            Ok(located) => located.is_some_and(|located| located.disabled),
            Err(err) => {
                tracing::warn!("Failed to look at \"{content_path}\" of \"{instance_id}\": {err}");
                false
            }
        }
    }

    /// Download every file into `staging`, concurrently; the shared download semaphore inside
    /// the request client is what bounds it. A file is hashed for its record right after it
    /// lands, in the same task, while the next ones download.
    ///
    /// The first failure cancels the rest: a pack of hundreds of files is not downloaded to the
    /// end only to be refused for one of them.
    ///
    /// The files share one budget of bytes ([`PlanBudget`]). Their declared sizes refuse a plan
    /// too large by its own account before anything is downloaded; the bytes that arrive are
    /// what holds it to the limit.
    async fn download_all(
        &self,
        files: Vec<PlannedFile>,
        staging: &Path,
        progress: &PlanProgress<'_>,
    ) -> Result<Vec<StagedFile>, InstanceError> {
        let mut declared: u64 = 0;
        for file in &files {
            declared = declared.saturating_add(file.entry.size.unwrap_or(0));
            if declared > self.limits.max_plan_bytes {
                return Err(InstanceError::FileTooLarge {
                    path: file.entry.relative_path.clone(),
                    size: declared,
                    limit: self.limits.max_plan_bytes,
                });
            }
        }
        let budget = &PlanBudget::new(self.limits.max_plan_bytes);

        crate::shared::io::infra::create_dir_all(staging)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))?;

        try_join_all(files.into_iter().enumerate().map(|(index, file)| {
            // Named by position: the plan's paths are not flat file names, and two of them may
            // share a file name.
            let path = staging.join(index.to_string());

            async move {
                let entry = &file.entry;
                let file_progress = progress.start_file(entry).await;
                let staged = async {
                    file_progress
                        .download(self.content_downloader.as_ref(), entry, &path, budget)
                        .await?;
                    file_progress.set_stage(ProgressStage::Verifying).await;
                    sha1_of(&path, entry.hash.as_ref()).await
                }
                .await;
                // Once its download and hashing are over.
                file_progress.bar.settle(&staged).await;
                let sha1 = staged?;
                progress.step("Verifying").await;

                Ok(StagedFile { file, path, sha1 })
            }
        }))
        .await
    }

    /// The instance to install into: the new one created, or the existing one moved to the game
    /// the plan names (a no-op when it already runs it) — with the instance as it was before
    /// that move, to move it back.
    ///
    /// The game is installed under the plan's bar, with mirrors of the shared downloads it
    /// waits on (its Java, its game version) under the game's bar.
    async fn prepare_target(
        &self,
        target: InstallTarget,
        progress: &PlanProgress<'_>,
    ) -> Result<(InstanceId, Option<Instance>), InstanceError> {
        let parent_bar = progress.bar.as_ref();
        match target {
            InstallTarget::Existing { instance_id, game } => {
                let mut before = None;
                if let Some(game) = game {
                    before = Some(self.instance_storage.get(&instance_id).await?);
                    self.change_instance_game
                        .execute(instance_id.clone(), game, parent_bar)
                        .await?;
                }
                Ok((instance_id, before))
            }
            // A pack's instance is not launchable until the pack is in: the caller moves it
            // to `Installed` after the plan, and a failure removes it (`discard_created`).
            InstallTarget::New(spec) if spec.pack_info.is_some() => {
                let instance_id = self
                    .create_instance
                    .execute_for_pack(spec, parent_bar)
                    .await?;
                Ok((instance_id, None))
            }
            InstallTarget::New(spec) => {
                let instance_id = self.create_instance.execute(spec, parent_bar).await?;
                Ok((instance_id, None))
            }
        }
    }

    /// Remove an instance the plan created and could not fill. Half of a new instance is worth
    /// nothing to keep, and a pack's would stay `PackInstalling`, unlaunchable, for good; the
    /// user installs the plan again instead.
    async fn discard_created(&self, instance_id: &InstanceId, err: &InstanceError) {
        tracing::info!("Failed to fill the new instance \"{instance_id}\": {err}. Rolling back");

        if let Err(remove_err) = self.remove_instance.execute(instance_id.clone()).await {
            tracing::error!(
                "Failed to remove instance \"{instance_id}\" during rollback: {remove_err}"
            );
        }
    }

    async fn install_all(
        &self,
        instance_id: &InstanceId,
        staged: Vec<StagedFile>,
        progress: &PlanProgress<'_>,
        undo: &mut Undo,
    ) -> Result<Vec<InstalledFile>, InstanceError> {
        let mut installed = Vec::with_capacity(staged.len());

        for StagedFile { file, path, sha1 } in staged {
            self.put_file(instance_id, &file.install_path(), &path, undo)
                .await?;
            progress.step("Installing").await;

            installed.push(InstalledFile {
                entry: file.entry,
                sha1,
            });
        }

        Ok(installed)
    }

    /// Record every planned file as `origin`'s, under its content path (never the `.disabled`
    /// one, as `list_content` keys them).
    ///
    /// The side is the plan's, and so is the name when the plan has one. Update info, and a name
    /// the plan lacks, are kept from the old record while the bytes are the same, and dropped
    /// with them otherwise; a ref's file gets the update info of what it resolved to.
    async fn record_installed(
        &self,
        instance_id: &InstanceId,
        origin: &PlanOrigin,
        records: &HashMap<String, PackFile>,
        installed: Vec<InstalledFile>,
        undo: &mut Undo,
    ) -> Result<(), InstanceError> {
        let (paths, files): (Vec<_>, Vec<_>) = installed
            .into_iter()
            .map(|InstalledFile { entry, sha1 }| {
                let same_bytes = records
                    .get(&entry.relative_path)
                    .filter(|record| record.hash.eq_ignore_ascii_case(&sha1));

                let mut pack_file = PackFile {
                    file_name: file_name_of(&entry.relative_path).to_owned(),
                    name: entry
                        .name
                        .or_else(|| same_bytes.and_then(|record| record.name.clone())),
                    hash: sha1,
                    download: None,
                    option: None,
                    side: Some(entry.side),
                    update_provider_id: same_bytes
                        .and_then(|record| record.update_provider_id.clone()),
                    update: same_bytes.and_then(|record| record.update.clone()),
                    installed_by: Some(origin.clone()),
                };

                if let Some(resolved) = entry.resolved {
                    pack_file.update_provider_id = Some(resolved.provider_id.clone());
                    pack_file.update = Some(HashMap::from([(
                        resolved.provider_id,
                        ContentFileUpdateInfo {
                            content_id: resolved.content_id,
                            version: resolved.version,
                        },
                    )]));
                }

                (entry.relative_path, pack_file)
            })
            .unzip();

        if paths.is_empty() {
            return Ok(());
        }

        undo.records.extend(paths.iter().cloned());
        self.pack_storage
            .update_pack_file_many(instance_id, &paths, &files)
            .await
    }

    /// Move the staged overrides into the instance, over whatever is at their paths; a path the
    /// user disabled stays disabled.
    async fn apply_overrides(
        &self,
        instance_id: &InstanceId,
        staged: Vec<StagedOverride>,
        progress: &PlanProgress<'_>,
        undo: &mut Undo,
    ) -> Result<(), InstanceError> {
        for StagedOverride {
            relative_path,
            path,
        } in staged
        {
            let disabled = self
                .content_file_service
                .locate_content_file(instance_id, &relative_path)
                .await?
                .is_some_and(|located| located.disabled);
            let install_path = if disabled {
                format!("{relative_path}{DISABLED_SUFFIX}")
            } else {
                relative_path
            };

            self.put_file(instance_id, &install_path, &path, undo)
                .await?;
            progress.step("Installing").await;
        }

        Ok(())
    }

    /// Remove the dropped files — moved aside, enabled or disabled — then their records. A file
    /// that cannot be removed fails the plan before any record goes: dropping its record would
    /// leave an unrecorded copy next to the new version.
    async fn remove_dropped(
        &self,
        instance_id: &InstanceId,
        content_paths: &[String],
        undo: &mut Undo,
    ) -> Result<(), InstanceError> {
        if content_paths.is_empty() {
            return Ok(());
        }

        for content_path in content_paths {
            // A file that is gone already counts as removed.
            if !self.move_aside(instance_id, content_path, undo).await? {
                self.move_aside(
                    instance_id,
                    &format!("{content_path}{DISABLED_SUFFIX}"),
                    undo,
                )
                .await?;
            }
        }

        undo.records.extend(content_paths.iter().cloned());
        self.pack_storage
            .remove_pack_file_many(instance_id, content_paths)
            .await
    }
}

/// The sha1 the pack storage records a file under: `known` when it is one (the file was checked
/// against it), else read from the file.
async fn sha1_of(path: &Path, known: Option<&FileHash>) -> Result<String, InstanceError> {
    match known {
        Some(hash) if hash.algorithm() == HashAlgorithm::Sha1 => Ok(hash.hex().to_owned()),
        _ => hash_file_hex_async(HashAlgorithm::Sha1, path)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string())),
    }
}

/// A path as an override is matched against the planned ones: with either separator, in any
/// case — on a case-insensitive file system these are one file.
fn path_key(relative_path: &str) -> String {
    relative_path.replace('\\', "/").to_lowercase()
}

/// The last segment of a content path.
fn file_name_of(relative_path: &str) -> &str {
    relative_path.rsplit('/').next().unwrap_or(relative_path)
}

#[async_trait]
impl InstallPlanExecutorPort for InstallPlanExecutor {
    async fn execute(
        &self,
        plan: InstallPlan,
        origin: PlanOrigin,
    ) -> Result<InstanceId, InstanceError> {
        self.execute(plan, origin).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_name_the_backup_and_the_path_when_describing_a_file_moved_aside() {
        let change = FileChange::MovedAside {
            install_path: "mods/a.jar".into(),
            backup: PathBuf::from("temp")
                .join("install-plan-1-backup")
                .join("3"),
        };

        let description = change.restore_description();

        assert!(description.contains("\"mods/a.jar\""), "{description}");
        assert!(
            description.contains(
                &PathBuf::from("temp")
                    .join("install-plan-1-backup")
                    .join("3")
                    .display()
                    .to_string()
            ),
            "{description}"
        );
    }

    #[test]
    fn should_name_only_the_path_when_describing_a_created_file() {
        let change = FileChange::Created {
            install_path: "mods/b.jar".into(),
        };

        assert_eq!(
            change.restore_description(),
            "take out the new \"mods/b.jar\""
        );
    }
}
