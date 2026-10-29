//! A content provider plugin on real Extism, built from WAT in the test: registration from a
//! `capabilities.json` of the new format and the `resolve_*` round trip through the wasm
//! boundary, the errors a looping or crashing plugin gives the user, a busy plugin next to the
//! others, and what WASI may do in the read-only `/instances`.

use std::{
    fmt::Write,
    path::PathBuf,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use aether_core_plugin_api::v0::{
    ContentAuthorDto, ContentMetadataDto, ContentSearchResultDto, ContentVersionMetadataDto,
    FileHashDto, ModLoaderDto, PackMetadataDto, ProviderIdDto, ResolvedFileDto,
};
use async_trait::async_trait;
use extism::{Function, Manifest, PluginBuilder, UserData, Wasm};
use extism_convert::{Msgpack, ToBytes};
use serde::Serialize;

use crate::features::instance::{CapabilityId, PluginId};
use crate::{
    features::{
        events::{Event, EventEmitter, EventError, PluginEvent},
        instance::{
            ContentProvider, ContentSearchParams, ContentType, Importer, InstanceError, InstanceId,
            PackSource, ProviderId, Updater,
        },
        plugins::{
            ApiConfig, LoadConfig, Plugin, PluginCapabilities, PluginContentProviderCapability,
            PluginHandle, PluginManifest, PluginMetadata, PluginRegistry, PluginState,
            RuntimeConfig,
            infra::{
                PluginCapabilitiesV1, PluginContentProviderProxy, PluginInfrastructureListener,
                extism::models::ExtismPluginInstance,
                plugin_utils::{get_default_allowed_paths, to_extism_mount_key},
            },
        },
        settings::LocationInfo,
    },
    shared::{
        capability::{CapabilityRegistry, infra::MemoryCapabilityRegistry},
        hash::{FileHash, HashAlgorithm},
        limits::PluginSandboxLimits,
    },
};

const PLUGIN_ID: &str = "curseforge-plugin";

fn plugin_id() -> PluginId {
    PluginId::from_static(PLUGIN_ID)
}

const SHA1_ABC: &str = "a9993e364706816aba3e25717850c26c9cd0d89d";

const CAPABILITIES: &str = r#"{
    "contentProviders": [{
        "id": "curseforge",
        "name": "CurseForge",
        "platform": "curseforge",
        "handlers": {
            "search": "cf_search",
            "getContent": "cf_get_content",
            "resolveFile": "cf_resolve_file",
            "resolvePackMetadata": "cf_resolve_pack_metadata",
            "getContentsMetadata": "cf_get_contents_metadata",
            "getVersionsMetadata": "cf_get_versions_metadata",
            "getAuthors": "cf_get_authors"
        }
    }]
}"#;

// ─── The plugin ────────────────────────────────────────────────

/// What one export of the test plugin does.
enum Body {
    /// Hands back these bytes as its output, as the PDK does with a Msgpack answer.
    Answer(Vec<u8>),
    /// Calls the host function `wait` first, then answers.
    WaitThenAnswer(Vec<u8>),
    Spin,
    Panic,
}

fn encode<T: Serialize>(value: T) -> Vec<u8> {
    Msgpack(value).to_bytes().expect("DTO should encode")
}

/// A module exporting `exports`. The answers are data segments; `$emit` copies one into Extism
/// memory and sets it as the output.
fn provider_wat(exports: &[(&str, Body)]) -> String {
    let mut data = String::new();
    let mut functions = String::new();
    let mut offset = 0usize;

    for (name, body) in exports {
        let code = match body {
            Body::Answer(bytes) | Body::WaitThenAnswer(bytes) => {
                let _ = write!(data, "(data (i32.const {offset}) \"");
                for byte in bytes {
                    let _ = write!(data, "\\{byte:02x}");
                }
                data.push_str("\")\n");
                let wait = if matches!(body, Body::WaitThenAnswer(_)) {
                    "(call $wait)"
                } else {
                    ""
                };
                let code = format!(
                    "{wait} (call $emit (i32.const {offset}) (i32.const {})) (i32.const 0)",
                    bytes.len()
                );
                offset += bytes.len();
                code
            }
            Body::Spin => "(loop $forever (br $forever)) (i32.const 0)".to_owned(),
            Body::Panic => "unreachable".to_owned(),
        };
        let _ = writeln!(functions, "(func (export \"{name}\") (result i32) {code})");
    }

    format!(
        r#"(module
        (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
        (import "extism:host/env" "store_u8" (func $store_u8 (param i64 i32)))
        (import "extism:host/env" "output_set" (func $output_set (param i64 i64)))
        (import "extism:host/user" "wait" (func $wait))
        (memory (export "memory") 1)
        {data}
        (func $emit (param $from i32) (param $len i32)
            (local $offset i64)
            (local $i i32)
            (local.set $offset (call $alloc (i64.extend_i32_u (local.get $len))))
            (block $copied
                (loop $copy
                    (br_if $copied (i32.ge_u (local.get $i) (local.get $len)))
                    (call $store_u8
                        (i64.add (local.get $offset) (i64.extend_i32_u (local.get $i)))
                        (i32.load8_u (i32.add (local.get $from) (local.get $i))))
                    (local.set $i (i32.add (local.get $i) (i32.const 1)))
                    (br $copy)))
            (call $output_set (local.get $offset) (i64.extend_i32_u (local.get $len))))
        {functions})"#
    )
}

const PATIENCE: Duration = Duration::from_secs(5);

/// Holds the plugin inside its host function `wait` until opened. Gives up after a while, so a
/// regression fails the test instead of hanging it.
#[derive(Default)]
struct Gate {
    entered: AtomicBool,
    open: Mutex<bool>,
    opened: Condvar,
}

impl Gate {
    fn pass(&self) {
        self.entered.store(true, Ordering::SeqCst);
        let open = self.open.lock().unwrap();
        drop(
            self.opened
                .wait_timeout_while(open, PATIENCE, |open| !*open)
                .unwrap(),
        );
    }

    fn open(&self) {
        *self.open.lock().unwrap() = true;
        self.opened.notify_all();
    }

    async fn entered(&self) {
        tokio::time::timeout(PATIENCE, async {
            while !self.entered.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("the plugin should reach `wait`");
    }
}

/// The plugin built the way the loader builds it: WASI on, the default instruction budget.
fn wasm_plugin(id: &str, exports: &[(&str, Body)], gate: Arc<Gate>) -> PluginHandle {
    let wait = Function::new("wait", [], [], UserData::new(()), move |_, _, _, _| {
        gate.pass();
        Ok(())
    });
    let plugin = PluginBuilder::new(Manifest::new([Wasm::data(
        provider_wat(exports).into_bytes(),
    )]))
    .with_functions([wait])
    .with_wasi(true)
    .with_fuel_limit(PluginSandboxLimits::default().default_fuel_limit)
    .build()
    .expect("the test plugin should load");

    PluginHandle::new(ExtismPluginInstance::new(
        plugin,
        id.parse::<PluginId>().unwrap(),
    ))
}

fn capabilities() -> PluginCapabilities {
    let file: PluginCapabilitiesV1 = serde_json::from_str(CAPABILITIES).expect("capabilities");
    let capabilities = PluginCapabilities::try_from(file).expect("valid capabilities");
    capabilities.validate().expect("valid capabilities");
    capabilities
}

fn provider_capability() -> PluginContentProviderCapability {
    capabilities().content_providers.into_iter().next().unwrap()
}

struct Sandbox {
    _root: tempfile::TempDir,
    location_info: Arc<LocationInfo>,
}

fn sandbox() -> Sandbox {
    let root = tempfile::tempdir().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        root.path().join("settings"),
        root.path().join("config"),
    ));
    Sandbox {
        _root: root,
        location_info,
    }
}

fn proxy(sandbox: &Sandbox, id: &str, handle: PluginHandle) -> PluginContentProviderProxy {
    PluginContentProviderProxy::new(
        id.parse::<PluginId>().unwrap(),
        handle,
        provider_capability(),
        sandbox.location_info.clone(),
    )
}

fn resolved_jei() -> Vec<u8> {
    encode(ResolvedFileDto {
        version: "4512345".into(),
        url: "https://edge.forgecdn.net/files/4512/345/jei.jar".into(),
        hash: Some(FileHashDto::Sha1(SHA1_ABC.into())),
        file_name: "jei.jar".into(),
        size: Some(3),
        name: Some("JEI 15.2.0".into()),
    })
}

fn pack_metadata() -> Vec<u8> {
    encode(PackMetadataDto {
        name: "All the Mods".into(),
        version_id: Some("5000000".into()),
        version_name: None,
        description: None,
        game_version: "1.21.1".into(),
        mod_loader: ModLoaderDto::NeoForge,
        loader_version: None,
        file_count: None,
        icon_url: None,
    })
}

fn search_result(plugin_id: &PluginId) -> Vec<u8> {
    encode(ContentSearchResultDto {
        page: 1,
        page_size: 20,
        page_count: 1,
        provider_id: ProviderIdDto {
            plugin_id: plugin_id.to_string(),
            capability_id: "curseforge".into(),
        },
        items: Vec::new(),
    })
}

fn search_params(plugin_id: &PluginId) -> ContentSearchParams {
    ContentSearchParams {
        content_type: ContentType::Mod,
        provider_id: ProviderId::from_parts(plugin_id, "curseforge").unwrap(),
        page: 1,
        page_size: 20,
        query: Some("jei".into()),
        game_versions: None,
        loader: None,
    }
}

fn provider_error(result: Result<impl std::fmt::Debug, InstanceError>) -> String {
    match result {
        Err(InstanceError::ContentProviderError { reason }) => reason,
        other => panic!("expected a content provider error, got {other:?}"),
    }
}

// ─── Registration and `resolve_*` ──────────────────────────────

struct NoOpEmitter;

#[async_trait]
impl EventEmitter<Event> for NoOpEmitter {
    async fn emit(&self, _event: Event) -> Result<(), EventError> {
        Ok(())
    }
    fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
}

fn manifest() -> PluginManifest {
    PluginManifest {
        metadata: PluginMetadata {
            id: plugin_id(),
            name: "CurseForge".into(),
            version: semver::Version::new(0, 1, 0),
            description: None,
            authors: vec![],
            license: None,
        },
        runtime: RuntimeConfig {
            allowed_hosts: vec![],
            allowed_paths: vec![],
        },
        load: LoadConfig::Extism {
            file: "plugin.wasm".try_into().unwrap(),
            memory_limit: None,
        },
        api: ApiConfig {
            version: semver::VersionReq::STAR,
            features: vec![],
            host_functions: Some(vec![]),
        },
    }
}

/// The provider of a loaded wasm plugin is registered under its platform and answers through
/// the wasm boundary; once the plugin is no longer loaded the provider is gone, which is what
/// makes a provider chosen for the platform unavailable.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_serve_a_platform_from_a_loaded_wasm_plugin_and_drop_it_when_unloaded() {
    let sandbox = sandbox();
    let handle = wasm_plugin(
        PLUGIN_ID,
        &[("cf_resolve_file", Body::Answer(resolved_jei()))],
        Arc::default(),
    );
    let plugins = Arc::new(PluginRegistry::new(Arc::new(NoOpEmitter)));
    let plugin = |state| Plugin {
        manifest: manifest(),
        capabilities: Some(capabilities()),
        hash: "hash".into(),
        verified: false,
        state,
    };
    plugins.insert(plugin_id(), plugin(PluginState::Loaded(handle)));
    let providers = Arc::new(MemoryCapabilityRegistry::<Arc<dyn ContentProvider>>::default());
    let listener = PluginInfrastructureListener::new(
        plugins.clone(),
        Arc::new(MemoryCapabilityRegistry::<Arc<dyn Importer>>::default()),
        Arc::new(MemoryCapabilityRegistry::<Arc<dyn Updater>>::default()),
        providers.clone(),
        sandbox.location_info.clone(),
    );
    let edit = || PluginEvent::Edit {
        plugin_id: plugin_id(),
    };

    listener.on_plugin_event(edit()).await;

    let provider = providers
        .find_by_plugin_and_capability_id(&plugin_id(), &CapabilityId::from_static("curseforge"))
        .await
        .expect("registered")
        .capability;
    assert_eq!(provider.metadata().platform.as_deref(), Some("curseforge"));
    let resolved = provider
        .resolve_file("238222", None, "1.21.1", Some("fabric"))
        .await
        .unwrap();
    assert_eq!(resolved.version, "4512345");
    assert_eq!(resolved.file_name, "jei.jar");
    assert_eq!(
        resolved.hash,
        Some(FileHash::new(HashAlgorithm::Sha1, SHA1_ABC).unwrap())
    );
    assert_eq!(resolved.name.as_deref(), Some("JEI 15.2.0"));

    plugins.insert(plugin_id(), plugin(PluginState::NotLoaded));
    listener.on_plugin_event(edit()).await;

    assert!(
        providers
            .find_by_plugin_and_capability_id(
                &plugin_id(),
                &CapabilityId::from_static("curseforge"),
            )
            .await
            .is_err(),
        "an unloaded plugin serves nothing"
    );
}

/// The metadata handlers through the wasm boundary: msgpack lists of ids in, lists of items out.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_describe_content_when_a_wasm_plugin_answers_the_metadata_handlers() {
    let sandbox = sandbox();
    let handle = wasm_plugin(
        PLUGIN_ID,
        &[
            (
                "cf_get_contents_metadata",
                Body::Answer(encode(vec![ContentMetadataDto {
                    content_id: "238222".into(),
                    slug: "jei".into(),
                    name: "Just Enough Items".into(),
                    description: None,
                    author: Some("mezz".into()),
                    author_ref: Some("team:abc".into()),
                    icon_url: Some("https://media.forgecdn.net/jei.png".into()),
                }])),
            ),
            (
                "cf_get_authors",
                Body::Answer(encode(vec![ContentAuthorDto {
                    author_ref: "team:abc".into(),
                    name: "Mezz Team".into(),
                }])),
            ),
            (
                "cf_get_versions_metadata",
                Body::Answer(encode(vec![ContentVersionMetadataDto {
                    version_id: "4512345".into(),
                    version_number: "15.2.0".into(),
                    name: Some("JEI 15.2.0".into()),
                }])),
            ),
        ],
        Arc::default(),
    );
    let proxy = proxy(&sandbox, PLUGIN_ID, handle);

    let projects = proxy
        .get_contents_metadata(&["238222".to_owned()])
        .await
        .unwrap();
    let versions = proxy
        .get_versions_metadata(&["4512345".to_owned()])
        .await
        .unwrap();

    assert_eq!(projects[0].content_id, "238222");
    assert_eq!(projects[0].name, "Just Enough Items");
    assert_eq!(projects[0].author.as_deref(), Some("mezz"));
    assert_eq!(projects[0].author_ref.as_deref(), Some("team:abc"));
    assert_eq!(versions[0].version_number, "15.2.0");
    assert_eq!(versions[0].name.as_deref(), Some("JEI 15.2.0"));

    let authors = proxy.get_authors(&["team:abc".to_owned()]).await.unwrap();
    assert_eq!(authors.len(), 1);
    assert_eq!(authors[0].author_ref, "team:abc");
    assert_eq!(authors[0].name, "Mezz Team");
}

/// A failing `getAuthors` leaves the metadata handler alone.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_still_describe_content_when_a_wasm_plugin_fails_naming_authors() {
    let sandbox = sandbox();
    let handle = wasm_plugin(
        PLUGIN_ID,
        &[
            (
                "cf_get_contents_metadata",
                Body::Answer(encode(vec![ContentMetadataDto {
                    content_id: "238222".into(),
                    slug: "jei".into(),
                    name: "Just Enough Items".into(),
                    description: None,
                    author: None,
                    author_ref: Some("team:abc".into()),
                    icon_url: None,
                }])),
            ),
            ("cf_get_authors", Body::Panic),
        ],
        Arc::default(),
    );
    let proxy = proxy(&sandbox, PLUGIN_ID, handle);

    let authors = provider_error(proxy.get_authors(&["team:abc".to_owned()]).await);
    let projects = proxy
        .get_contents_metadata(&["238222".to_owned()])
        .await
        .unwrap();

    assert!(authors.contains("the plugin crashed"), "{authors}");
    assert_eq!(projects[0].name, "Just Enough Items");
    assert_eq!(projects[0].author_ref.as_deref(), Some("team:abc"));
}

// ─── What the user is told ─────────────────────────────────────

/// With the default budget, not a test-sized one: the loop ends on fuel, long before the
/// thirty-minute timeout, and the message names the limit.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_name_the_instruction_limit_when_a_provider_loops() {
    let sandbox = sandbox();
    let proxy = proxy(
        &sandbox,
        PLUGIN_ID,
        wasm_plugin(
            PLUGIN_ID,
            &[("cf_resolve_file", Body::Spin)],
            Arc::default(),
        ),
    );

    let reason = provider_error(proxy.resolve_file("238222", None, "1.21.1", None).await);

    assert!(
        reason.contains("the plugin exceeded its instruction limit"),
        "{reason}"
    );
}

/// A Rust `panic!` in a plugin aborts into `unreachable`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_name_a_crash_when_a_provider_panics() {
    let sandbox = sandbox();
    let proxy = proxy(
        &sandbox,
        PLUGIN_ID,
        wasm_plugin(
            PLUGIN_ID,
            &[("cf_resolve_file", Body::Panic)],
            Arc::default(),
        ),
    );

    let reason = provider_error(proxy.resolve_file("238222", None, "1.21.1", None).await);

    assert!(reason.contains("the plugin crashed"), "{reason}");
}

// ─── A busy plugin ─────────────────────────────────────────────

/// While a plugin works on a pack, another plugin's search answers and the runtime's only
/// worker stays free; a search in the busy plugin waits for its turn and then answers too.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn should_search_elsewhere_and_queue_its_own_search_when_a_plugin_is_busy() {
    let sandbox = sandbox();
    let gate = Arc::new(Gate::default());
    let busy = Arc::new(proxy(
        &sandbox,
        PLUGIN_ID,
        wasm_plugin(
            PLUGIN_ID,
            &[
                (
                    "cf_resolve_pack_metadata",
                    Body::WaitThenAnswer(pack_metadata()),
                ),
                ("cf_search", Body::Answer(search_result(&plugin_id()))),
            ],
            gate.clone(),
        ),
    ));
    let other = proxy(
        &sandbox,
        "other-plugin",
        wasm_plugin(
            "other-plugin",
            &[(
                "cf_search",
                Body::Answer(search_result(&PluginId::from_static("other-plugin"))),
            )],
            Arc::default(),
        ),
    );

    let pack = tokio::spawn({
        let busy = busy.clone();
        async move {
            busy.resolve_pack_metadata(PackSource::Url("https://example.com/pack.zip".into()))
                .await
        }
    });
    gate.entered().await;

    let found = other
        .search(search_params(&PluginId::from_static("other-plugin")))
        .await
        .unwrap();
    assert_eq!(found.provider_id.plugin_id.as_str(), "other-plugin");
    assert!(
        !pack.is_finished(),
        "the other search answered while the pack was being worked on"
    );

    let own_search = tokio::spawn({
        let busy = busy.clone();
        async move { busy.search(search_params(&plugin_id())).await }
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(
        !own_search.is_finished(),
        "a search in the busy plugin waits for its turn"
    );

    gate.open();
    assert_eq!(
        pack.await.unwrap().unwrap().version_id.as_deref(),
        Some("5000000")
    );
    assert_eq!(
        own_search
            .await
            .unwrap()
            .unwrap()
            .provider_id
            .plugin_id
            .as_str(),
        PLUGIN_ID
    );
}

// ─── `/instances` through WASI ─────────────────────────────────

/// Talks WASI to the one folder mounted (fd 3). Each export returns the WASI errno of the
/// step that failed, 0 on success.
const WASI_PLUGIN: &str = r#"(module
    (import "wasi_snapshot_preview1" "path_open"
        (func $path_open (param i32 i32 i32 i32 i32 i64 i64 i32 i32) (result i32)))
    (import "wasi_snapshot_preview1" "fd_write"
        (func $fd_write (param i32 i32 i32 i32) (result i32)))
    (import "wasi_snapshot_preview1" "fd_read"
        (func $fd_read (param i32 i32 i32 i32) (result i32)))
    (import "wasi_snapshot_preview1" "fd_close" (func $fd_close (param i32) (result i32)))
    (memory (export "memory") 1)
    (data (i32.const 0) "inst/pack.toml")
    (data (i32.const 16) "inst/new.toml")
    (data (i32.const 32) "written")

    ;; fd lands at 80; iovec at 64; byte count at 84; read buffer at 128.
    (func $open (param $path i32) (param $len i32) (param $oflags i32) (param $rights i64)
        (result i32)
        (call $path_open (i32.const 3) (i32.const 0) (local.get $path) (local.get $len)
            (local.get $oflags) (local.get $rights) (i64.const 0) (i32.const 0)
            (i32.const 80)))

    (func (export "write") (result i32)
        (local $rc i32)
        (local.set $rc (call $open (i32.const 0) (i32.const 14) (i32.const 0) (i64.const 64)))
        (if (local.get $rc) (then (return (local.get $rc))))
        (i32.store (i32.const 64) (i32.const 32))
        (i32.store (i32.const 68) (i32.const 7))
        (local.set $rc (call $fd_write (i32.load (i32.const 80)) (i32.const 64) (i32.const 1)
            (i32.const 84)))
        (drop (call $fd_close (i32.load (i32.const 80))))
        (local.get $rc))

    ;; 200 if the file is not the 8 bytes expected.
    (func (export "read") (result i32)
        (local $rc i32)
        (local.set $rc (call $open (i32.const 0) (i32.const 14) (i32.const 0) (i64.const 2)))
        (if (local.get $rc) (then (return (local.get $rc))))
        (i32.store (i32.const 64) (i32.const 128))
        (i32.store (i32.const 68) (i32.const 64))
        (local.set $rc (call $fd_read (i32.load (i32.const 80)) (i32.const 64) (i32.const 1)
            (i32.const 84)))
        (drop (call $fd_close (i32.load (i32.const 80))))
        (if (local.get $rc) (then (return (local.get $rc))))
        (if (i32.ne (i32.load (i32.const 84)) (i32.const 8)) (then (return (i32.const 200))))
        (i32.const 0))

    ;; O_CREAT with FD_READ only.
    (func (export "create") (result i32)
        (local $rc i32)
        (local.set $rc (call $open (i32.const 16) (i32.const 13) (i32.const 1) (i64.const 2)))
        (if (local.get $rc) (then (return (local.get $rc))))
        (drop (call $fd_close (i32.load (i32.const 80))))
        (i32.const 0)))"#;

const PACK: &[u8] = b"original";

/// The plugin with the core's `/instances` mount — the default mount, keyed as the loader keys
/// it — and an instance holding `pack.toml`.
fn instances_plugin(sandbox: &Sandbox) -> (PathBuf, extism::Plugin) {
    let instance = sandbox
        .location_info
        .instance_dir(&InstanceId::try_from("inst").unwrap());
    std::fs::create_dir_all(&instance).unwrap();
    std::fs::write(instance.join("pack.toml"), PACK).unwrap();

    let mounts = get_default_allowed_paths(&sandbox.location_info, &plugin_id())
        .into_iter()
        .filter(|(_, mount)| mount.plugin_root() == "/instances")
        .map(|(host, mount)| {
            (
                to_extism_mount_key(&host, mount.is_read_only()),
                PathBuf::from(mount.plugin_root()),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(mounts.len(), 1, "the core mounts `/instances`");

    let plugin = PluginBuilder::new(
        Manifest::new([Wasm::data(WASI_PLUGIN.as_bytes().to_vec())])
            .with_allowed_paths(mounts.into_iter()),
    )
    .with_wasi(true)
    .build()
    .expect("the WASI plugin should load");

    (instance, plugin)
}

/// The export's return value, which `call` reports only in its message.
fn errno(plugin: &mut extism::Plugin, function: &str) -> i32 {
    const NON_ZERO: &str = "Returned non-zero exit code: ";

    match plugin.call::<&[u8], &[u8]>(function, &[]) {
        Ok(_) => 0,
        Err(err) => {
            let message = err.to_string();
            message
                .strip_prefix(NON_ZERO)
                .and_then(|code| code.parse().ok())
                .unwrap_or_else(|| panic!("`{function}` failed outside WASI: {message}"))
        }
    }
}

/// A write into an instance goes through the checked host function only: opening a file of
/// `/instances` for writing is refused, reading it works. Creating a file is refused too
/// (older Extism let it through), pinned so a change either way is noticed.
#[test]
fn should_refuse_a_wasi_write_into_instances_but_allow_a_read_when_the_mount_is_read_only() {
    let sandbox = sandbox();
    let (instance, mut plugin) = instances_plugin(&sandbox);

    assert_ne!(
        errno(&mut plugin, "write"),
        0,
        "opening for write is refused"
    );
    assert_eq!(std::fs::read(instance.join("pack.toml")).unwrap(), PACK);
    assert_eq!(errno(&mut plugin, "read"), 0, "reading the same file works");

    assert_ne!(
        errno(&mut plugin, "create"),
        0,
        "creating a file is refused"
    );
    assert!(!instance.join("new.toml").exists());
}
