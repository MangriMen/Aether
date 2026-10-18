use std::{
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::SystemTime,
};

use async_trait::async_trait;
use tempfile::TempDir;

use crate::{
    features::{
        events::{
            EventError, ProgressBarId, ProgressEventType, ProgressService, ProgressServiceExt,
        },
        instance::{
            CachedContentAuthor, CachedContentMetadata, CachedContentVersionMetadata,
            CapabilityMetadata, ChangeInstanceGameUseCasePort, ContentAuthor,
            ContentCompatibilityCheckParams, ContentCompatibilityResult, ContentDownloader,
            ContentFileUpdateInfo, ContentIdentity, ContentItem, ContentMetadata,
            ContentMetadataCache, ContentProvider, ContentProviderCapabilityMetadata,
            ContentSearchParams, ContentSearchResult, ContentSource, ContentType, ContentVersion,
            ContentVersionMetadata, CreateInstanceUseCasePort, FileSide, InstallFileEntry,
            InstallPlan, InstallPlanExecutor, InstallTarget, Instance, InstanceBuilder,
            InstanceError, InstanceGame, InstanceId, InstanceStorage, NewInstance,
            OverrideLocation, OverrideSource, PackFile, PackInfo, PackStorage, PlanOrigin,
            PlatformProviderSelection, ProviderId, RemoveInstanceUseCasePort, ResolvedFile,
            app::{ContentFileService, LocatedContentFile},
            infra::{FsContentFileService, SqlitePackStorage},
        },
        minecraft::{LoaderVersionPreference, ModLoader},
        settings::LocationInfo,
    },
    shared::{
        capability::{CapabilityRegistry, infra::MemoryCapabilityRegistry},
        hash::FileHash,
        request_client::BytesProgress,
    },
};

pub(super) const INSTANCE_ID: &str = "instance";
pub(super) const NEW_INSTANCE_ID: &str = "new-instance";
pub(super) const PACK_ID: &str = "pack";
pub(super) const SHA1_ABC: &str = "a9993e364706816aba3e25717850c26c9cd0d89d";

// ─── Mocks ─────────────────────────────────────────────────────

pub(super) type ResolveCall = (String, Option<String>, String, Option<String>);

/// A provider of one platform whose `resolve_file` answers from a fixed URL per content id and
/// records how it was asked.
pub(super) struct MockProvider {
    pub(super) id: ProviderId,
    pub(super) metadata: ContentProviderCapabilityMetadata,
    pub(super) calls: Mutex<Vec<ResolveCall>>,
}

impl MockProvider {
    pub(super) fn new(plugin_id: &str, capability_id: &str, platform: Option<&str>) -> Arc<Self> {
        Arc::new(Self {
            id: ProviderId {
                plugin_id: plugin_id.into(),
                capability_id: capability_id.into(),
            },
            metadata: ContentProviderCapabilityMetadata {
                base: CapabilityMetadata {
                    id: capability_id.into(),
                    name: capability_id.into(),
                    description: None,
                    icon: None,
                },
                platform: platform.map(Into::into),
            },
            calls: Mutex::default(),
        })
    }

    pub(super) fn builtin_modrinth() -> Arc<Self> {
        Self::new(ProviderId::BUILTIN_PLUGIN_ID, "modrinth", Some("modrinth"))
    }

    /// Where this provider says the file of `content_id` is.
    pub(super) fn url_for(&self, content_id: &str) -> String {
        format!("https://{}.example/{content_id}.jar", self.id.plugin_id)
    }

    pub(super) fn calls(&self) -> Vec<ResolveCall> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl ContentProvider for MockProvider {
    fn metadata(&self) -> &ContentProviderCapabilityMetadata {
        &self.metadata
    }

    fn provider_id(&self) -> ProviderId {
        self.id.clone()
    }

    async fn search(&self, _: ContentSearchParams) -> Result<ContentSearchResult, InstanceError> {
        unimplemented!()
    }

    async fn get_content(&self, _: String) -> Result<ContentItem, InstanceError> {
        unimplemented!()
    }

    async fn list_versions(&self, _: String) -> Result<Vec<ContentVersion>, InstanceError> {
        unimplemented!()
    }

    async fn resolve_file(
        &self,
        content_id: &str,
        version: Option<&str>,
        game_version: &str,
        loader: Option<&str>,
    ) -> Result<ResolvedFile, InstanceError> {
        self.calls.lock().unwrap().push((
            content_id.into(),
            version.map(Into::into),
            game_version.into(),
            loader.map(Into::into),
        ));

        if content_id == "unsupported" {
            return Err(self.unsupported("resolve_file"));
        }

        Ok(ResolvedFile {
            version: version.unwrap_or("newest").into(),
            url: self.url_for(content_id),
            hash: Some(FileHash::Sha1(SHA1_ABC.into())),
            file_name: format!("{content_id}.jar"),
            size: None,
            name: Some(format!("{content_id} {}", version.unwrap_or("newest"))),
        })
    }

    async fn check_compatibility(
        &self,
        _: &[Instance],
        _: &ContentCompatibilityCheckParams,
    ) -> Result<HashMap<InstanceId, ContentCompatibilityResult>, InstanceError> {
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Download {
    pub(super) origin: String,
    pub(super) url: String,
    pub(super) hash: Option<FileHash>,
}

/// Writes the URL as the file's body, records every call, and fails for a URL containing
/// `fail`. A URL containing `hang` writes part of its file, signals `hanging`, and never ends.
#[derive(Default)]
pub(super) struct RecordingDownloader {
    pub(super) downloads: Mutex<Vec<Download>>,
    pub(super) hanging: tokio::sync::Notify,
}

impl RecordingDownloader {
    pub(super) fn downloads(&self) -> Vec<Download> {
        let mut downloads = self.downloads.lock().unwrap().clone();
        // Downloads run concurrently; their order is not part of the contract.
        downloads.sort_by(|a, b| a.url.cmp(&b.url));
        downloads
    }
}

#[async_trait]
impl ContentDownloader for RecordingDownloader {
    async fn download(
        &self,
        origin: &ProviderId,
        url: &str,
        hash: Option<&FileHash>,
        destination: &Path,
        on_bytes: Option<BytesProgress<'_>>,
    ) -> Result<u64, InstanceError> {
        self.downloads.lock().unwrap().push(Download {
            origin: origin.to_string(),
            url: url.into(),
            hash: hash.cloned(),
        });

        if url.contains("fail") {
            return Err(InstanceError::ContentDownloadError(url.into()));
        }
        if url.contains("hang") {
            tokio::fs::write(destination, "part").await.unwrap();
            self.hanging.notify_one();
            std::future::pending::<()>().await;
        }

        // A URL containing `chunks` arrives in four pieces, the way a large file does.
        if let (true, Some(on_bytes)) = (url.contains("chunks"), on_bytes) {
            let piece = url.len() as u64 / 4;
            on_bytes(piece);
            on_bytes(piece);
            on_bytes(piece);
            on_bytes(url.len() as u64 - 3 * piece);
        }

        tokio::fs::write(destination, url).await.unwrap();
        Ok(url.len() as u64)
    }
}

/// Creates the instance directory the way the real use case does, and records the spec — under
/// `created_for_pack` when it was created for a pack to be deployed into.
pub(super) struct RecordingCreateInstance {
    pub(super) location_info: Arc<LocationInfo>,
    pub(super) created: Mutex<Vec<NewInstance>>,
    pub(super) created_for_pack: Mutex<Vec<NewInstance>>,
    /// The bar each instance was created under.
    pub(super) parent_bars: Mutex<Vec<Option<uuid::Uuid>>>,
    /// When set, a bar is started on it the way installing the game does.
    pub(super) game_progress: Mutex<Option<Arc<dyn ProgressService>>>,
}

impl RecordingCreateInstance {
    pub(super) async fn create_dir(&self, parent_bar: Option<&ProgressBarId>) {
        self.parent_bars
            .lock()
            .unwrap()
            .push(parent_bar.map(|bar| bar.id));
        let game_progress = self.game_progress.lock().unwrap().clone();
        if let Some(progress) = game_progress {
            let bar = progress
                .init_progress_under_safe(
                    parent_bar,
                    ProgressEventType::MinecraftDownload {
                        instance_id: InstanceId::try_from(NEW_INSTANCE_ID).unwrap(),
                        instance_name: NEW_INSTANCE_ID.into(),
                    },
                    100.0,
                    "Downloading Minecraft".into(),
                )
                .await;
            drop(bar);
        }

        tokio::fs::create_dir_all(
            self.location_info
                .instance_dir(&InstanceId::try_from(NEW_INSTANCE_ID).unwrap()),
        )
        .await
        .unwrap();
    }
}

#[async_trait]
impl CreateInstanceUseCasePort for RecordingCreateInstance {
    async fn execute(
        &self,
        new_instance: NewInstance,
        parent_bar: Option<&ProgressBarId>,
    ) -> Result<InstanceId, InstanceError> {
        self.create_dir(parent_bar).await;
        self.created.lock().unwrap().push(new_instance);
        Ok(InstanceId::try_from(NEW_INSTANCE_ID).unwrap())
    }

    async fn execute_for_pack(
        &self,
        new_instance: NewInstance,
        parent_bar: Option<&ProgressBarId>,
    ) -> Result<InstanceId, InstanceError> {
        self.create_dir(parent_bar).await;
        self.created_for_pack.lock().unwrap().push(new_instance);
        Ok(InstanceId::try_from(NEW_INSTANCE_ID).unwrap())
    }
}

/// Records every game an instance is moved to, and fails for a game version containing `fail`.
#[derive(Default)]
pub(super) struct RecordingChangeGame {
    pub(super) changes: Mutex<Vec<(String, InstanceGame)>>,
    /// The bar each move was made under.
    pub(super) parent_bars: Mutex<Vec<Option<uuid::Uuid>>>,
}

impl RecordingChangeGame {
    pub(super) fn changes(&self) -> Vec<(String, InstanceGame)> {
        self.changes.lock().unwrap().clone()
    }
}

#[async_trait]
impl ChangeInstanceGameUseCasePort for RecordingChangeGame {
    async fn execute(
        &self,
        instance_id: InstanceId,
        game: InstanceGame,
        parent_bar: Option<&ProgressBarId>,
    ) -> Result<(), InstanceError> {
        self.parent_bars
            .lock()
            .unwrap()
            .push(parent_bar.map(|bar| bar.id));
        let fail = game.game_version.contains("fail");
        self.changes
            .lock()
            .unwrap()
            .push((instance_id.into(), game));

        if fail {
            Err(InstanceError::Storage("install failed".into()))
        } else {
            Ok(())
        }
    }
}

/// Records every instance removed.
#[derive(Default)]
pub(super) struct RecordingRemove {
    pub(super) removed: Mutex<Vec<String>>,
}

#[async_trait]
impl RemoveInstanceUseCasePort for RecordingRemove {
    async fn execute(&self, instance_id: InstanceId) -> Result<(), InstanceError> {
        self.removed.lock().unwrap().push(instance_id.into());
        Ok(())
    }
}

/// The one instance of a fixture; records every instance written.
pub(super) struct OneInstanceStorage {
    pub(super) instance: Instance,
    pub(super) upserted: Arc<Mutex<Vec<Instance>>>,
}

#[async_trait]
impl InstanceStorage for OneInstanceStorage {
    async fn list(&self) -> Result<Vec<Instance>, InstanceError> {
        Ok(vec![self.instance.clone()])
    }

    async fn get(&self, id: &InstanceId) -> Result<Instance, InstanceError> {
        if id == self.instance.id() {
            Ok(self.instance.clone())
        } else {
            Err(InstanceError::NotFound {
                instance_id: id.to_string(),
            })
        }
    }

    async fn upsert(&self, instance: &Instance) -> Result<(), InstanceError> {
        self.upserted.lock().unwrap().push(instance.clone());
        Ok(())
    }

    async fn remove(&self, _: &InstanceId) -> Result<(), InstanceError> {
        unimplemented!()
    }
}

pub(super) struct FixedSelection(Option<ProviderId>);

#[async_trait]
impl PlatformProviderSelection for FixedSelection {
    async fn selected_provider(&self, _: &str) -> Result<Option<ProviderId>, InstanceError> {
        Ok(self.0.clone())
    }
}

pub(super) type RecordedBar = (f64, Vec<(f64, Option<String>)>);

/// Records the bar the executor starts and every step it takes on it.
#[derive(Default)]
pub(super) struct RecordingProgress {
    /// `(total, steps)`: each step is `(increment, message)`.
    pub(super) bar: Mutex<Option<RecordedBar>>,
    /// The id of the bar.
    pub(super) bar_id: Mutex<Option<uuid::Uuid>>,
    /// The bars started under it, as `(parent, event)`.
    pub(super) children: Mutex<Vec<(uuid::Uuid, ProgressEventType)>>,
    /// The caption (`pack_name`) of the started bar.
    pub(super) caption: Mutex<Option<String>>,
    /// A step message to stall on: reaching it signals `stalled` and never returns.
    pub(super) stall_on: Mutex<Option<String>>,
    pub(super) stalled: tokio::sync::Notify,
    /// Every count of bytes reported as downloaded, to any of the bars.
    pub(super) bytes: Mutex<Vec<u64>>,
    /// The bytes the bars were told to expect together, if they were.
    pub(super) bytes_total: Mutex<Option<u64>>,
}

impl RecordingProgress {
    pub(super) fn bar(&self) -> RecordedBar {
        self.bar
            .lock()
            .unwrap()
            .clone()
            .expect("no bar was started")
    }
}

#[async_trait]
impl ProgressService for RecordingProgress {
    async fn init_progress(
        &self,
        event_type: ProgressEventType,
        total: f64,
        _message: String,
    ) -> Result<ProgressBarId, EventError> {
        if let ProgressEventType::PackDownload { pack_name, .. } = event_type {
            *self.caption.lock().unwrap() = Some(pack_name);
        }
        *self.bar.lock().unwrap() = Some((total, Vec::new()));
        let id = uuid::Uuid::new_v4();
        *self.bar_id.lock().unwrap() = Some(id);
        Ok(ProgressBarId {
            id,
            progress_storage: None,
            event_emitter: None,
        })
    }
    async fn init_child_progress(
        &self,
        parent_id: uuid::Uuid,
        event_type: ProgressEventType,
        _total: f64,
        _message: String,
    ) -> Result<ProgressBarId, EventError> {
        self.children.lock().unwrap().push((parent_id, event_type));
        Ok(ProgressBarId {
            id: uuid::Uuid::new_v4(),
            progress_storage: None,
            event_emitter: None,
        })
    }
    async fn init_or_edit_progress(
        &self,
        _progress_bar_id: Option<ProgressBarId>,
        _event_type: ProgressEventType,
        _total: f64,
        _message: String,
    ) -> Result<ProgressBarId, EventError> {
        Err(EventError::NotInitialized)
    }
    async fn emit_progress(
        &self,
        progress_bar_id: &ProgressBarId,
        increment_frac: f64,
        message: Option<&str>,
    ) -> Result<(), EventError> {
        // Only the steps of the plan's bar; the files' bars move with them.
        if *self.bar_id.lock().unwrap() != Some(progress_bar_id.id) {
            return Ok(());
        }
        self.bar
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .1
            .push((increment_frac, message.map(Into::into)));
        let stall = message
            .is_some_and(|message| self.stall_on.lock().unwrap().as_deref() == Some(message));
        if stall {
            self.stalled.notify_one();
            std::future::pending::<()>().await;
        }
        Ok(())
    }
    async fn edit_progress(
        &self,
        _progress_bar_id: &ProgressBarId,
        _event_type: ProgressEventType,
        _total: f64,
        _message: String,
    ) -> Result<(), EventError> {
        Err(EventError::NotInitialized)
    }
    async fn add_progress_bytes(
        &self,
        _progress_bar_id: &ProgressBarId,
        bytes: u64,
    ) -> Result<(), EventError> {
        self.bytes.lock().unwrap().push(bytes);
        Ok(())
    }
    async fn add_progress_bytes_total(
        &self,
        _progress_bar_id: &ProgressBarId,
        bytes: u64,
    ) -> Result<(), EventError> {
        let mut total = self.bytes_total.lock().unwrap();
        *total = Some(total.unwrap_or(0) + bytes);
        Ok(())
    }
}

/// The real file service, but for putting a file back from a backup directory, which fails.
pub(super) struct FailingRestore(FsContentFileService);

#[async_trait]
impl ContentFileService for FailingRestore {
    async fn remove_content_files(
        &self,
        instance_id: &InstanceId,
        content_paths: &[String],
    ) -> Result<(), InstanceError> {
        self.0
            .remove_content_files(instance_id, content_paths)
            .await
    }

    async fn enable_content_files(
        &self,
        instance_id: &InstanceId,
        content_paths: &[String],
    ) -> Result<(), InstanceError> {
        self.0
            .enable_content_files(instance_id, content_paths)
            .await
    }

    async fn disable_content_files(
        &self,
        instance_id: &InstanceId,
        content_paths: &[String],
    ) -> Result<(), InstanceError> {
        self.0
            .disable_content_files(instance_id, content_paths)
            .await
    }

    async fn locate_content_file(
        &self,
        instance_id: &InstanceId,
        content_path: &str,
    ) -> Result<Option<LocatedContentFile>, InstanceError> {
        self.0.locate_content_file(instance_id, content_path).await
    }

    fn check_content_path(&self, content_path: &str) -> Result<(), InstanceError> {
        self.0.check_content_path(content_path)
    }

    async fn move_content_file_out(
        &self,
        instance_id: &InstanceId,
        content_path: &str,
        destination: &Path,
    ) -> Result<bool, InstanceError> {
        self.0
            .move_content_file_out(instance_id, content_path, destination)
            .await
    }

    async fn install_content_file(
        &self,
        instance_id: &InstanceId,
        content_path: &str,
        temp_path: &Path,
    ) -> Result<(), InstanceError> {
        let from_backup = temp_path
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with("-backup"));
        if from_backup {
            return Err(InstanceError::Storage("cannot put the file back".into()));
        }
        self.0
            .install_content_file(instance_id, content_path, temp_path)
            .await
    }
}

/// Keeps what is put in it, in memory, with when it was put as fetched.
#[derive(Default)]
pub(super) struct MemoryMetadataCache {
    pub(super) items: Mutex<HashMap<(ProviderId, String), CachedContentMetadata>>,
    pub(super) versions: Mutex<HashMap<(ProviderId, String), CachedContentVersionMetadata>>,
}

#[async_trait]
impl ContentMetadataCache for MemoryMetadataCache {
    async fn get_many(
        &self,
        provider_id: &ProviderId,
        content_ids: &[String],
    ) -> HashMap<String, CachedContentMetadata> {
        let items = self.items.lock().unwrap();
        content_ids
            .iter()
            .filter_map(|id| {
                let item = items.get(&(provider_id.clone(), id.clone()))?;
                Some((id.clone(), item.clone()))
            })
            .collect()
    }

    async fn put_keyed_at(
        &self,
        provider_id: &ProviderId,
        items: &[(String, ContentMetadata)],
        fetched_at: SystemTime,
    ) -> bool {
        let mut cached = self.items.lock().unwrap();
        for (content_id, item) in items {
            cached.insert(
                (provider_id.clone(), content_id.clone()),
                CachedContentMetadata {
                    metadata: item.clone(),
                    fetched_at,
                },
            );
        }
        true
    }

    async fn get_versions(
        &self,
        provider_id: &ProviderId,
        version_ids: &[String],
    ) -> HashMap<String, CachedContentVersionMetadata> {
        let versions = self.versions.lock().unwrap();
        version_ids
            .iter()
            .filter_map(|id| {
                let item = versions.get(&(provider_id.clone(), id.clone()))?;
                Some((id.clone(), item.clone()))
            })
            .collect()
    }

    async fn put_versions(
        &self,
        provider_id: &ProviderId,
        items: &[ContentVersionMetadata],
    ) -> bool {
        let mut cached = self.versions.lock().unwrap();
        for item in items {
            cached.insert(
                (provider_id.clone(), item.version_id.clone()),
                CachedContentVersionMetadata {
                    metadata: item.clone(),
                    fetched_at: SystemTime::now(),
                },
            );
        }
        true
    }

    async fn get_authors(
        &self,
        _: &ProviderId,
        _: &[String],
    ) -> HashMap<String, CachedContentAuthor> {
        HashMap::new()
    }

    async fn put_authors(&self, _: &ProviderId, _: &[ContentAuthor]) -> bool {
        true
    }
}

// ─── Fixture ───────────────────────────────────────────────────

pub(super) struct Fixture {
    pub(super) _temp_dir: TempDir,
    pub(super) location_info: Arc<LocationInfo>,
    pub(super) registry: Arc<MemoryCapabilityRegistry<Arc<dyn ContentProvider>>>,
    pub(super) downloader: Arc<RecordingDownloader>,
    pub(super) create_instance: Arc<RecordingCreateInstance>,
    pub(super) change_game: Arc<RecordingChangeGame>,
    pub(super) pack_storage: Arc<SqlitePackStorage>,
    pub(super) selected: Option<ProviderId>,
    /// The pack the existing instance was installed from.
    pub(super) pack_info: Option<PackInfo>,
    pub(super) progress: Arc<RecordingProgress>,
    /// A progress service used in place of `progress`.
    pub(super) progress_service: Option<Arc<dyn ProgressService>>,
    pub(super) remove: Arc<RecordingRemove>,
    pub(super) metadata_cache: Arc<MemoryMetadataCache>,
    /// Limits on the overrides as `(files, bytes)`, in place of the executor's own.
    pub(super) override_limits: Option<(usize, u64)>,
    /// Limits on the planned files as `(files, bytes)`, in place of the executor's own.
    pub(super) plan_limits: Option<(usize, u64)>,
    /// Every instance the executor wrote.
    pub(super) upserted: Arc<Mutex<Vec<Instance>>>,
    /// Putting a file back from a backup fails.
    pub(super) fail_restore: bool,
}

impl Fixture {
    pub(super) async fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let location_info = Arc::new(LocationInfo::new(
            temp_dir.path().join("settings"),
            temp_dir.path().to_path_buf(),
        ));
        tokio::fs::create_dir_all(
            location_info.instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap()),
        )
        .await
        .unwrap();

        Self {
            _temp_dir: temp_dir,
            create_instance: Arc::new(RecordingCreateInstance {
                location_info: location_info.clone(),
                created: Mutex::default(),
                created_for_pack: Mutex::default(),
                parent_bars: Mutex::default(),
                game_progress: Mutex::default(),
            }),
            location_info,
            registry: Arc::default(),
            downloader: Arc::default(),
            change_game: Arc::default(),
            pack_storage: Arc::new(SqlitePackStorage::in_memory().await),
            selected: None,
            pack_info: None,
            progress: Arc::default(),
            progress_service: None,
            remove: Arc::default(),
            metadata_cache: Arc::default(),
            override_limits: None,
            plan_limits: None,
            upserted: Arc::default(),
            fail_restore: false,
        }
    }

    pub(super) async fn register(&self, provider: &Arc<MockProvider>) {
        self.registry
            .add(
                provider.id.plugin_id.clone(),
                provider.id.capability_id.clone(),
                provider.clone(),
            )
            .await
            .unwrap();
    }

    /// Execute `plan` as the plan of `origin` for the content `PACK_ID`.
    pub(super) async fn execute(
        &self,
        plan: InstallPlan,
        origin: &ProviderId,
    ) -> Result<InstanceId, InstanceError> {
        self.execute_as(plan, pack_of(origin)).await
    }

    pub(super) async fn execute_as(
        &self,
        plan: InstallPlan,
        origin: PlanOrigin,
    ) -> Result<InstanceId, InstanceError> {
        let instance = InstanceBuilder::new(
            InstanceId::try_from(INSTANCE_ID).unwrap(),
            "Instance".into(),
            "1.21.1".into(),
            ModLoader::Fabric,
        )
        .with_pack_info(self.pack_info.clone())
        .build();

        let executor = InstallPlanExecutor::new(
            self.registry.clone(),
            Arc::new(FixedSelection(self.selected.clone())),
            Arc::new(OneInstanceStorage {
                instance,
                upserted: self.upserted.clone(),
            }),
            self.create_instance.clone(),
            self.change_game.clone(),
            self.downloader.clone(),
            self.content_file_service(),
            self.pack_storage.clone(),
            self.location_info.clone(),
            self.progress_service
                .clone()
                .unwrap_or_else(|| self.progress.clone() as Arc<dyn ProgressService>),
            self.remove.clone(),
            self.metadata_cache.clone(),
            self.limits(),
        );

        executor.execute(plan, origin).await
    }

    pub(super) fn content_file_service(&self) -> Arc<dyn ContentFileService> {
        let files = FsContentFileService::new(self.location_info.clone());
        if self.fail_restore {
            Arc::new(FailingRestore(files))
        } else {
            Arc::new(files)
        }
    }

    /// The backup directories of runs in the temp directory.
    pub(super) fn backup_dirs(&self) -> Vec<PathBuf> {
        let Ok(entries) = std::fs::read_dir(self.location_info.temp_dir()) else {
            return Vec::new();
        };
        entries
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.to_string_lossy().ends_with("-backup"))
            .collect()
    }

    pub(super) fn limits(&self) -> crate::shared::limits::ContentInstallLimits {
        let mut limits = crate::shared::limits::ContentInstallLimits::default();
        if let Some((files, bytes)) = self.override_limits {
            limits.max_override_files = files;
            limits.max_override_bytes = bytes;
        }
        if let Some((files, bytes)) = self.plan_limits {
            limits.max_plan_files = files;
            limits.max_plan_bytes = bytes;
        }
        limits
    }

    pub(super) fn instance_file(&self, instance_id: &str, relative_path: &str) -> Option<String> {
        std::fs::read_to_string(
            self.location_info
                .instance_dir(&InstanceId::try_from(instance_id).unwrap())
                .join(relative_path),
        )
        .ok()
    }

    /// Every path under the instance directory, relative and `/`-separated.
    pub(super) fn instance_files(&self, instance_id: &str) -> Vec<String> {
        fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    walk(root, &path, out);
                } else {
                    let relative = path.strip_prefix(root).unwrap();
                    out.push(relative.to_string_lossy().replace('\\', "/"));
                }
            }
        }

        let root = self
            .location_info
            .instance_dir(&InstanceId::try_from(instance_id).unwrap());
        let mut out = Vec::new();
        if root.exists() {
            walk(&root, &root, &mut out);
        }
        out.sort();
        out
    }

    pub(super) fn staging_is_clean(&self) -> bool {
        let temp = self.location_info.temp_dir();
        !temp.exists() || std::fs::read_dir(temp).unwrap().next().is_none()
    }

    /// Put a file into the existing instance, as if it had been there before the plan.
    pub(super) fn put(&self, relative_path: &str, body: &str) {
        let path = self
            .location_info
            .instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap())
            .join(relative_path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    /// Record a file of the existing instance in the pack storage.
    pub(super) async fn record(&self, relative_path: &str, pack_file: PackFile) {
        self.pack_storage
            .update_pack_file(
                &InstanceId::try_from(INSTANCE_ID).unwrap(),
                relative_path,
                &pack_file,
            )
            .await
            .unwrap();
    }

    /// Put a file into the existing instance and record it as installed by `installed_by`.
    pub(super) async fn put_installed(
        &self,
        relative_path: &str,
        body: &str,
        installed_by: Option<PlanOrigin>,
    ) {
        self.put(relative_path, body);
        self.record(
            relative_path,
            PackFile {
                installed_by,
                ..PackFile::from_hash(file_name(relative_path), sha1(body))
            },
        )
        .await;
    }

    pub(super) async fn records(&self, instance_id: &str) -> HashMap<String, PackFile> {
        self.pack_storage
            .get_pack_files(&InstanceId::try_from(instance_id).unwrap())
            .await
            .unwrap()
    }

    pub(super) fn downloaded_urls(&self) -> Vec<String> {
        self.downloader
            .downloads()
            .into_iter()
            .map(|download| download.url)
            .collect()
    }
}

pub(super) fn sha1(body: &str) -> String {
    sha1_smol::Sha1::from(body).hexdigest()
}

pub(super) fn file_name(relative_path: &str) -> String {
    relative_path.rsplit('/').next().unwrap().into()
}

/// The content of `provider_id` the plans in these tests are for, unless a test says otherwise.
pub(super) fn pack_of(provider_id: &ProviderId) -> PlanOrigin {
    PlanOrigin {
        provider_id: provider_id.clone(),
        content_id: PACK_ID.into(),
    }
}

pub(super) fn plugin(plugin_id: &str) -> ProviderId {
    ProviderId {
        plugin_id: plugin_id.into(),
        capability_id: "provider".into(),
    }
}

pub(super) fn existing(files: Vec<InstallFileEntry>) -> InstallPlan {
    InstallPlan {
        target: InstallTarget::existing(InstanceId::try_from(INSTANCE_ID).unwrap()),
        files,
        overrides: Vec::new(),
    }
}

/// A plan for the existing instance that moves it to `game`.
pub(super) fn existing_on(game: &InstanceGame, files: Vec<InstallFileEntry>) -> InstallPlan {
    InstallPlan {
        target: InstallTarget::Existing {
            instance_id: InstanceId::try_from(INSTANCE_ID).unwrap(),
            game: Some(game.clone()),
        },
        files,
        overrides: Vec::new(),
    }
}

pub(super) fn neoforge_1_21_4() -> InstanceGame {
    InstanceGame {
        game_version: "1.21.4".into(),
        mod_loader: ModLoader::NeoForge,
        loader_version: Some(LoaderVersionPreference::Exact("21.4.1".into())),
    }
}

/// The instance as installed from the pack `PACK_ID` of `provider_id`.
pub(super) fn installed_from(provider_id: &ProviderId, modpack_id: &str) -> PackInfo {
    PackInfo {
        provider_id: provider_id.clone(),
        modpack_id: modpack_id.into(),
        version_id: "v1".into(),
    }
}

pub(super) fn provider_ref(platform: &str, content_id: &str, path: &str) -> InstallFileEntry {
    InstallFileEntry::new(
        ContentSource::ProviderRef {
            platform: platform.into(),
            content_id: content_id.into(),
            version: None,
        },
        path,
        ContentType::Mod,
    )
}

pub(super) fn direct(url: &str, path: &str) -> InstallFileEntry {
    InstallFileEntry::new(
        ContentSource::direct(url.into(), None),
        path,
        ContentType::Mod,
    )
}

/// A direct entry with the hash of what the recording downloader writes for it: its URL.
pub(super) fn hashed(url: &str, path: &str) -> InstallFileEntry {
    InstallFileEntry::new(
        ContentSource::direct(url.into(), Some(FileHash::Sha1(sha1(url)))),
        path,
        ContentType::Mod,
    )
}

pub(super) fn modrinth_id() -> ProviderId {
    ProviderId {
        plugin_id: ProviderId::BUILTIN_PLUGIN_ID.into(),
        capability_id: "modrinth".into(),
    }
}

/// Update info naming `content_id` on the built-in Modrinth, as a ref's file is recorded.
pub(super) fn modrinth_update(
    content_id: &str,
    version: &str,
) -> HashMap<ProviderId, ContentFileUpdateInfo> {
    HashMap::from([(
        modrinth_id(),
        ContentFileUpdateInfo {
            content_id: content_id.into(),
            version: version.into(),
        },
    )])
}

/// A direct entry naming the Modrinth content it is, as a `.mrpack` lists its files.
pub(super) fn identified(
    url: &str,
    path: &str,
    platform: &str,
    content_id: &str,
    version: &str,
) -> InstallFileEntry {
    InstallFileEntry::new(
        ContentSource::Direct {
            url: url.into(),
            hash: Some(FileHash::Sha1(sha1(url))),
            identity: Some(ContentIdentity {
                platform: platform.into(),
                content_id: content_id.into(),
                version: version.into(),
            }),
        },
        path,
        ContentType::Mod,
    )
}

/// A zip in the cache directory with `entries` as `(name, body)`; a name ending in `/` is a
/// directory entry.
pub(super) fn zip_archive(fixture: &Fixture, entries: &[(&str, &str)]) -> PathBuf {
    let dir = fixture.location_info.cache_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("pack.mrpack");

    let mut writer = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
    let options = zip::write::SimpleFileOptions::default();
    for (name, body) in entries {
        if let Some(dir) = name.strip_suffix('/') {
            writer.add_directory(dir, options).unwrap();
        } else {
            writer.start_file(*name, options).unwrap();
            writer.write_all(body.as_bytes()).unwrap();
        }
    }
    writer.finish().unwrap();

    path
}

pub(super) fn archive_overrides(archive_path: PathBuf) -> Vec<OverrideSource> {
    vec![archive_subdir(archive_path, "overrides", FileSide::Both)]
}

pub(super) fn archive_subdir(
    archive_path: PathBuf,
    subdir: &str,
    side: FileSide,
) -> OverrideSource {
    OverrideSource::new(
        OverrideLocation::Archive {
            archive_path,
            subdir: subdir.into(),
        },
        side,
    )
}
