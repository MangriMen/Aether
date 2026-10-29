//! The content provider contract end to end, without a WASM runtime: a `capabilities.json` in
//! the new format becomes a registered provider whose trait methods call the functions it names.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use aether_core_plugin_api::v0::{
    ContentAuthorDto, ContentCompatibilityResultDto, ContentMetadataDto, ContentSourceDto,
    ContentTypeDto, ContentVersionMetadataDto, FileHashDto, FileSideDto, GetAuthorsParamsDto,
    GetContentsMetadataParamsDto, GetVersionsMetadataParamsDto, InstallFileEntryDto,
    InstallPlanDto, InstallTargetDto, InstanceGameDto, ModLoaderDto, NewInstanceDto,
    OverrideLocationDto, OverrideSourceDto, PackMetadataDto, PackSourceDto, ResolveFileParamsDto,
    ResolveInstallPlanParamsDto, ResolveTargetDto, ResolvedFileDto,
};
use async_trait::async_trait;
use extism_convert::{FromBytes, Msgpack, ToBytes};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    features::{
        events::{Event, EventEmitter, EventError, PluginEvent},
        instance::{
            ContentCompatibilityCheckParams, ContentItem, ContentProvider, ContentSource,
            ContentType, FileSide, Importer, InstallTarget, InstanceError, InstanceGame,
            InstanceId, NewInstance, OverrideLocation, OverrideSource, PackSource, ProviderId,
            ResolveInstallPlanParams, ResolveTarget, Updater,
        },
        minecraft::ModLoader,
        plugins::{
            ApiConfig, LoadConfig, Plugin, PluginCallError, PluginCapabilities,
            PluginContentProviderCapability, PluginError, PluginHandle, PluginInstance,
            PluginInternalEvent, PluginManifest, PluginMetadata, PluginRegistry, PluginState,
            RuntimeConfig,
            infra::{
                PluginCapabilitiesV1, PluginContentProviderProxy, PluginInfrastructureListener,
            },
        },
        settings::LocationInfo,
    },
    shared::{
        capability::{CapabilityRegistry, infra::MemoryCapabilityRegistry},
        hash::{FileHash, HashAlgorithm},
    },
};

const PLUGIN_ID: &str = "curseforge-plugin";
const SHA1_ABC: &str = "a9993e364706816aba3e25717850c26c9cd0d89d";

/// A provider in the new format: no `install*` handlers, no `supports*` flags, a platform.
const RESOLVE_ONLY_CAPABILITIES: &str = r#"{
    "contentProviders": [{
        "id": "curseforge",
        "name": "CurseForge",
        "platform": "curseforge",
        "handlers": {
            "search": "cf_search",
            "getContent": "cf_get_content",
            "resolveFile": "cf_resolve_file",
            "resolveInstallPlan": "cf_resolve_install_plan",
            "resolvePackMetadata": "cf_resolve_pack_metadata",
            "getContentsMetadata": "cf_get_contents_metadata",
            "getVersionsMetadata": "cf_get_versions_metadata",
            "getAuthors": "cf_get_authors"
        }
    }]
}"#;

// ─── Mocks ─────────────────────────────────────────────────────

type Calls = Arc<Mutex<Vec<(String, Vec<u8>)>>>;

/// A plugin exporting the functions it has answers for, recording every call.
struct RecordingPlugin {
    answers: HashMap<String, Vec<u8>>,
    calls: Calls,
    last_answer: Vec<u8>,
}

impl PluginInstance for RecordingPlugin {
    fn get_id(&self) -> String {
        PLUGIN_ID.to_owned()
    }

    fn supports(&self, name: &str) -> bool {
        self.answers.contains_key(name)
    }

    fn call_bytes<'b>(&'b mut self, name: &str, args: &[u8]) -> Result<&'b [u8], PluginError> {
        self.calls
            .lock()
            .unwrap()
            .push((name.to_owned(), args.to_vec()));
        self.last_answer =
            self.answers
                .get(name)
                .cloned()
                .ok_or_else(|| PluginError::FunctionCallFailed {
                    function_name: name.to_owned(),
                    plugin_id: PLUGIN_ID.to_owned(),
                    error: PluginCallError::MissingFunction,
                })?;
        Ok(&self.last_answer)
    }

    fn handle_event(&mut self, _event: &PluginInternalEvent) -> Result<(), PluginError> {
        Ok(())
    }
}

struct NoOpEmitter;

#[async_trait]
impl EventEmitter<Event> for NoOpEmitter {
    async fn emit(&self, _event: Event) -> Result<(), EventError> {
        Ok(())
    }
    fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
}

// ─── Fixtures ──────────────────────────────────────────────────

fn encode<T: Serialize>(value: T) -> Vec<u8> {
    Msgpack(value).to_bytes().expect("DTO should encode")
}

fn decode<T: DeserializeOwned>(bytes: &[u8]) -> T {
    let Msgpack(value) = Msgpack::<T>::from_bytes(bytes).expect("arguments should decode");
    value
}

fn capabilities(json: &str) -> PluginCapabilities {
    let file: PluginCapabilitiesV1 = serde_json::from_str(json).expect("capabilities.json");
    PluginCapabilities::from(file)
}

fn provider_capability(json: &str) -> PluginContentProviderCapability {
    let capabilities = capabilities(json);
    capabilities.validate().expect("valid capabilities");
    capabilities.content_providers.into_iter().next().unwrap()
}

struct Fixture {
    root: tempfile::TempDir,
    location_info: Arc<LocationInfo>,
    calls: Calls,
    proxy: PluginContentProviderProxy,
}

impl Fixture {
    fn new(capabilities_json: &str, answers: Vec<(&str, Vec<u8>)>) -> Self {
        let root = tempfile::tempdir().unwrap();
        let location_info = Arc::new(LocationInfo::new(
            root.path().join("settings"),
            root.path().join("config"),
        ));
        let calls = Calls::default();
        let plugin = RecordingPlugin {
            answers: answers
                .into_iter()
                .map(|(name, answer)| (name.to_owned(), answer))
                .collect(),
            calls: calls.clone(),
            last_answer: vec![],
        };

        let proxy = PluginContentProviderProxy::new(
            PLUGIN_ID.to_owned(),
            PluginHandle::new(plugin),
            provider_capability(capabilities_json),
            location_info.clone(),
        );

        Self {
            root,
            location_info,
            calls,
            proxy,
        }
    }

    fn called(&self) -> Vec<String> {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .map(|(name, _)| name.clone())
            .collect()
    }

    fn arguments_of_last_call<T: DeserializeOwned>(&self) -> T {
        decode(&self.calls.lock().unwrap().last().expect("a call").1)
    }

    fn cache_file(&self, name: &str) -> std::path::PathBuf {
        let dir = self.location_info.plugin_cache_dir(PLUGIN_ID);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, b"zip").unwrap();
        path
    }
}

fn existing_target() -> ResolveTarget {
    ResolveTarget::Existing {
        instance_id: InstanceId::try_from("instance").unwrap(),
        game_version: "1.21.1".into(),
        loader: Some(ModLoader::Fabric),
    }
}

fn plan_params(target: ResolveTarget) -> ResolveInstallPlanParams {
    ResolveInstallPlanParams {
        content_id: "238222".into(),
        version: Some("4512345".into()),
        installed_version: None,
        target,
    }
}

fn jei_entry() -> InstallFileEntryDto {
    InstallFileEntryDto {
        source: ContentSourceDto::ProviderRef {
            platform: "curseforge".into(),
            content_id: "238222".into(),
            version: Some("4512345".into()),
        },
        relative_path: "mods/jei.jar".into(),
        content_type: ContentTypeDto::Mod,
        optional: None,
        side: FileSideDto::Client,
        size: None,
        name: None,
    }
}

fn plan_for(target: InstallTargetDto, overrides: Vec<OverrideSourceDto>) -> InstallPlanDto {
    InstallPlanDto {
        target,
        files: vec![jei_entry()],
        overrides,
    }
}

fn existing_plan_target(instance_id: &str) -> InstallTargetDto {
    InstallTargetDto::Existing {
        instance_id: instance_id.into(),
        game: None,
    }
}

fn assert_unsupported(err: InstanceError, expected_operation: &str) {
    let InstanceError::UnsupportedOperation {
        provider_id,
        operation,
    } = err
    else {
        panic!("expected UnsupportedOperation, got {err:?}")
    };
    assert_eq!(provider_id.to_string(), format!("{PLUGIN_ID}#curseforge"));
    assert_eq!(operation, expected_operation);
}

fn assert_invalid_answer(result: Result<impl std::fmt::Debug, InstanceError>) {
    assert_refused_with(
        result,
        &format!("Plugin '{PLUGIN_ID}' returned an invalid answer from"),
    );
}

fn assert_refused_with(result: Result<impl std::fmt::Debug, InstanceError>, expected: &str) {
    match result {
        Err(InstanceError::ContentProviderError { reason }) => {
            assert!(
                reason.contains(expected),
                "expected `{expected}` in: {reason}"
            );
        }
        other => panic!("expected the answer to be refused, got {other:?}"),
    }
}

// ─── capabilities.json → domain ────────────────────────────────

#[test]
fn should_read_the_provider_when_the_plugin_declares_it() {
    let capability = provider_capability(RESOLVE_ONLY_CAPABILITIES);

    assert_eq!(capability.metadata.platform.as_deref(), Some("curseforge"));
    assert_eq!(
        capability.handlers.resolve_file.as_deref(),
        Some("cf_resolve_file")
    );
    assert_eq!(
        capability.handlers.resolve_install_plan.as_deref(),
        Some("cf_resolve_install_plan")
    );
    assert_eq!(
        capability.handlers.resolve_pack_metadata.as_deref(),
        Some("cf_resolve_pack_metadata")
    );
}

/// Keys the core does not know are ignored: what the provider supports follows from its
/// handlers.
#[test]
fn should_ignore_unknown_keys_when_the_provider_declares_them() {
    let capability = provider_capability(
        r#"{
            "contentProviders": [{
                "id": "other",
                "name": "Other",
                "supportsInstallAtomic": false,
                "supportsInstallModpacks": true,
                "handlers": { "search": "s", "getContent": "g", "installAtomic": "i" }
            }]
        }"#,
    );

    assert_eq!(capability.metadata.platform, None);
    assert_eq!(capability.handlers.resolve_file, None);
}

#[test]
fn should_keep_platform_and_resolve_handlers_when_written_back() {
    let capabilities = capabilities(RESOLVE_ONLY_CAPABILITIES);

    let json = serde_json::to_string(&PluginCapabilitiesV1::from(capabilities)).unwrap();
    let back = provider_capability(&json);

    assert_eq!(back.metadata.platform.as_deref(), Some("curseforge"));
    assert_eq!(
        back.handlers.resolve_pack_metadata.as_deref(),
        Some("cf_resolve_pack_metadata")
    );
}

// ─── Proxy: which function is called, with what ────────────────

#[tokio::test]
async fn should_call_the_resolve_file_handler_the_file_names_when_resolving_a_file() {
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![(
            "cf_resolve_file",
            encode(ResolvedFileDto {
                version: "4512345".into(),
                url: "https://edge.forgecdn.net/files/4512/345/jei.jar".into(),
                hash: Some(FileHashDto::Sha1(SHA1_ABC.to_ascii_uppercase())),
                file_name: "jei.jar".into(),
                size: Some(3),
                name: Some("JEI 15.2.0".into()),
            }),
        )],
    );

    let resolved = fixture
        .proxy
        .resolve_file("238222", None, "1.21.1", Some("fabric"))
        .await
        .unwrap();

    assert_eq!(fixture.called(), ["cf_resolve_file"]);
    assert_eq!(
        fixture.arguments_of_last_call::<ResolveFileParamsDto>(),
        ResolveFileParamsDto {
            content_id: "238222".into(),
            version: None,
            game_version: "1.21.1".into(),
            loader: Some("fabric".into()),
        }
    );
    assert_eq!(resolved.version, "4512345");
    assert_eq!(
        resolved.hash,
        Some(FileHash::new(HashAlgorithm::Sha1, SHA1_ABC).unwrap())
    );
    assert_eq!(resolved.size, Some(3));
    assert_eq!(resolved.name.as_deref(), Some("JEI 15.2.0"));
}

#[tokio::test]
async fn should_call_the_resolve_install_plan_handler_the_file_names_when_resolving_a_plan() {
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![(
            "cf_resolve_install_plan",
            encode(plan_for(existing_plan_target("instance"), Vec::new())),
        )],
    );

    let plan = fixture
        .proxy
        .resolve_install_plan(plan_params(existing_target()))
        .await
        .unwrap();

    assert_eq!(fixture.called(), ["cf_resolve_install_plan"]);
    assert_eq!(
        fixture.arguments_of_last_call::<ResolveInstallPlanParamsDto>(),
        ResolveInstallPlanParamsDto {
            content_id: "238222".into(),
            version: Some("4512345".into()),
            installed_version: None,
            target: ResolveTargetDto::Existing {
                instance_id: "instance".into(),
                game_version: "1.21.1".into(),
                loader: Some("fabric".into()),
            },
        }
    );
    assert_eq!(
        plan.target,
        InstallTarget::existing(InstanceId::try_from("instance").unwrap())
    );
    assert_eq!(plan.files.len(), 1);
    assert_eq!(
        plan.files[0].source,
        ContentSource::ProviderRef {
            platform: "curseforge".into(),
            content_id: "238222".into(),
            version: Some("4512345".into()),
        }
    );
    assert_eq!(plan.files[0].side, FileSide::Client);
    assert_eq!(plan.files[0].content_type, ContentType::Mod);
    assert!(plan.overrides.is_empty());
}

#[tokio::test]
async fn should_call_the_pack_metadata_handler_the_file_names_when_resolving_pack_metadata() {
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![(
            "cf_resolve_pack_metadata",
            encode(PackMetadataDto {
                name: "All the Mods".into(),
                version_id: Some("5000000".into()),
                version_name: Some("1.0.0".into()),
                description: None,
                game_version: "1.21.1".into(),
                mod_loader: ModLoaderDto::NeoForge,
                loader_version: Some("21.1.1".into()),
                file_count: Some(400),
                icon_url: None,
            }),
        )],
    );

    let metadata = fixture
        .proxy
        .resolve_pack_metadata(PackSource::Content {
            content_id: "925200".into(),
            version: None,
            installed_version: None,
        })
        .await
        .unwrap();

    assert_eq!(fixture.called(), ["cf_resolve_pack_metadata"]);
    assert_eq!(
        fixture.arguments_of_last_call::<PackSourceDto>(),
        PackSourceDto::Content {
            content_id: "925200".into(),
            version: None,
            installed_version: None,
        }
    );
    assert_eq!(metadata.version_id.as_deref(), Some("5000000"));
    assert_eq!(metadata.mod_loader, ModLoader::NeoForge);
    assert_eq!(metadata.file_count, Some(400));
}

fn project_metadata(content_id: &str, icon_url: Option<&str>) -> ContentMetadataDto {
    ContentMetadataDto {
        content_id: content_id.into(),
        slug: format!("slug-{content_id}"),
        name: "Just Enough Items".into(),
        description: None,
        author: Some("mezz".into()),
        author_ref: None,
        icon_url: icon_url.map(Into::into),
    }
}

fn author(author_ref: &str, name: &str) -> ContentAuthorDto {
    ContentAuthorDto {
        author_ref: author_ref.into(),
        name: name.into(),
    }
}

#[tokio::test]
async fn should_call_the_authors_handler_the_file_names_when_naming_authors() {
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![(
            "cf_get_authors",
            encode(vec![author("team:abc", "Mezz Team")]),
        )],
    );

    let authors = fixture
        .proxy
        .get_authors(&["team:abc".to_owned()])
        .await
        .unwrap();

    assert_eq!(
        fixture.arguments_of_last_call::<GetAuthorsParamsDto>(),
        GetAuthorsParamsDto {
            author_refs: vec!["team:abc".into()]
        }
    );
    assert_eq!(fixture.called(), ["cf_get_authors"]);
    assert_eq!(authors.len(), 1);
    assert_eq!(authors[0].author_ref, "team:abc");
    assert_eq!(authors[0].name, "Mezz Team");
}

#[tokio::test]
async fn should_keep_an_author_ref_when_describing_content_and_drop_a_blank_one() {
    let mut with_ref = project_metadata("1", None);
    with_ref.author_ref = Some("team:abc".into());
    let mut blank_ref = project_metadata("2", None);
    blank_ref.author_ref = Some("  ".into());
    let mut long_ref = project_metadata("4", None);
    long_ref.author_ref = Some("t".repeat(257));
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![(
            "cf_get_contents_metadata",
            encode(vec![
                with_ref,
                blank_ref,
                project_metadata("3", None),
                long_ref,
            ]),
        )],
    );

    let projects = fixture
        .proxy
        .get_contents_metadata(&["1", "2", "3", "4"].map(str::to_owned))
        .await
        .unwrap();

    assert_eq!(projects[0].author_ref.as_deref(), Some("team:abc"));
    assert_eq!(projects[1].author_ref, None);
    assert_eq!(projects[2].author_ref, None);
    assert_eq!(projects[3].author_ref, None);
}

#[tokio::test]
async fn should_trim_and_cut_the_name_of_an_author() {
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![(
            "cf_get_authors",
            encode(vec![
                author("team:a", "  Mezz  "),
                author("team:b", &"n".repeat(1000)),
            ]),
        )],
    );

    let authors = fixture
        .proxy
        .get_authors(&["team:a", "team:b"].map(str::to_owned))
        .await
        .unwrap();

    assert_eq!(authors[0].name, "Mezz");
    assert_eq!(authors[1].name.chars().count(), 128);
}

#[tokio::test]
async fn should_drop_an_author_ref_when_the_plugin_has_no_authors_handler() {
    let mut with_ref = project_metadata("1", None);
    with_ref.author_ref = Some("team:abc".into());
    let capabilities = RESOLVE_ONLY_CAPABILITIES.replace(
        r#",
            "getAuthors": "cf_get_authors""#,
        "",
    );
    assert_ne!(capabilities, RESOLVE_ONLY_CAPABILITIES);
    let fixture = Fixture::new(
        &capabilities,
        vec![("cf_get_contents_metadata", encode(vec![with_ref]))],
    );

    let projects = fixture
        .proxy
        .get_contents_metadata(&["1".to_owned()])
        .await
        .unwrap();

    assert_eq!(projects[0].author_ref, None);
}

#[tokio::test]
async fn should_drop_authors_nobody_asked_about_repeated_or_unnamed_ones() {
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![(
            "cf_get_authors",
            encode(vec![
                author("team:a", "First"),
                author("team:a", "Again"),
                author("team:stranger", "Stranger"),
                author("  ", "Blank ref"),
                author("team:b", "  "),
                author("team:c", "Third"),
            ]),
        )],
    );

    let authors = fixture
        .proxy
        .get_authors(&["team:a", "team:b", "team:c"].map(str::to_owned))
        .await
        .unwrap();

    assert_eq!(
        authors
            .iter()
            .map(|a| (a.author_ref.as_str(), a.name.as_str()))
            .collect::<Vec<_>>(),
        [("team:a", "First"), ("team:c", "Third")]
    );
}

#[tokio::test]
async fn should_call_the_metadata_handlers_the_file_names_when_describing_content() {
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![
            (
                "cf_get_contents_metadata",
                encode(vec![project_metadata(
                    "238222",
                    Some("https://media.forgecdn.net/jei.png"),
                )]),
            ),
            (
                "cf_get_versions_metadata",
                encode(vec![ContentVersionMetadataDto {
                    version_id: "5000000".into(),
                    version_number: "19.21.0".into(),
                    name: Some(String::new()),
                }]),
            ),
        ],
    );

    let projects = fixture
        .proxy
        .get_contents_metadata(&["238222".to_owned()])
        .await
        .unwrap();
    assert_eq!(
        fixture.arguments_of_last_call::<GetContentsMetadataParamsDto>(),
        GetContentsMetadataParamsDto {
            content_ids: vec!["238222".into()]
        }
    );
    assert_eq!(projects[0].name, "Just Enough Items");
    assert_eq!(projects[0].author.as_deref(), Some("mezz"));
    assert_eq!(
        projects[0].icon_url.as_deref(),
        Some("https://media.forgecdn.net/jei.png")
    );

    let versions = fixture
        .proxy
        .get_versions_metadata(&["5000000".to_owned()])
        .await
        .unwrap();
    assert_eq!(
        fixture.arguments_of_last_call::<GetVersionsMetadataParamsDto>(),
        GetVersionsMetadataParamsDto {
            version_ids: vec!["5000000".into()]
        }
    );
    assert_eq!(versions[0].version_number, "19.21.0");
    assert_eq!(versions[0].name, None, "an empty title is no title");
    assert_eq!(
        fixture.called(),
        ["cf_get_contents_metadata", "cf_get_versions_metadata"]
    );
}

#[tokio::test]
async fn should_drop_an_icon_when_it_is_not_a_web_address() {
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![(
            "cf_get_contents_metadata",
            encode(vec![
                project_metadata("1", Some("file:///C:/Users/me/secret.png")),
                project_metadata("2", Some("/cache/icon.png")),
                project_metadata("3", Some("https://example.com/icon.png")),
                project_metadata("4", Some("http://example.com/icon.png")),
            ]),
        )],
    );

    let projects = fixture
        .proxy
        .get_contents_metadata(&["1", "2", "3", "4"].map(str::to_owned))
        .await
        .unwrap();

    assert_eq!(projects[0].icon_url, None);
    assert_eq!(projects[1].icon_url, None);
    assert_eq!(
        projects[2].icon_url.as_deref(),
        Some("https://example.com/icon.png")
    );
    assert_eq!(projects[3].icon_url, None, "cleartext http is not fetched");
}

#[tokio::test]
async fn should_keep_the_other_projects_when_one_described_project_has_no_name() {
    let mut nameless = project_metadata("1", None);
    nameless.name = "  ".into();
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![(
            "cf_get_contents_metadata",
            encode(vec![nameless, project_metadata("2", None)]),
        )],
    );

    let projects = fixture
        .proxy
        .get_contents_metadata(&["1".to_owned(), "2".to_owned()])
        .await
        .unwrap();

    assert_eq!(
        projects
            .iter()
            .map(|p| p.content_id.as_str())
            .collect::<Vec<_>>(),
        ["2"]
    );
}

#[tokio::test]
async fn should_drop_projects_nobody_asked_about_and_repeated_ones() {
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![(
            "cf_get_contents_metadata",
            encode(vec![
                project_metadata("1", None),
                project_metadata("1", None),
                project_metadata("stranger", None),
            ]),
        )],
    );

    let projects = fixture
        .proxy
        .get_contents_metadata(&["1".to_owned()])
        .await
        .unwrap();

    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].content_id, "1");
}

#[tokio::test]
async fn should_keep_a_project_when_it_answers_an_id_asked_by_its_slug() {
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![(
            "cf_get_contents_metadata",
            encode(vec![project_metadata("238222", None)]),
        )],
    );

    let projects = fixture
        .proxy
        .get_contents_metadata(&["Slug-238222".to_owned()])
        .await
        .unwrap();

    assert_eq!(projects[0].content_id, "238222");
}

#[tokio::test]
async fn should_drop_versions_nobody_asked_about_and_numberless_ones() {
    let version = |id: &str, number: &str| ContentVersionMetadataDto {
        version_id: id.into(),
        version_number: number.into(),
        name: None,
    };
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![(
            "cf_get_versions_metadata",
            encode(vec![
                version("1", "1.0"),
                version("1", "1.0"),
                version("2", ""),
                version("stranger", "9.9"),
            ]),
        )],
    );

    let versions = fixture
        .proxy
        .get_versions_metadata(&["1".to_owned(), "2".to_owned()])
        .await
        .unwrap();

    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].version_id, "1");
}

#[tokio::test]
async fn should_report_metadata_as_unsupported_when_the_provider_has_no_handler_for_it() {
    let fixture = Fixture::new(
        r#"{
            "contentProviders": [{
                "id": "curseforge",
                "name": "CurseForge",
                "handlers": { "search": "cf_search", "getContent": "cf_get_content" }
            }]
        }"#,
        vec![],
    );

    assert_unsupported(
        fixture
            .proxy
            .get_contents_metadata(&["238222".to_owned()])
            .await
            .unwrap_err(),
        "get_contents_metadata",
    );
    assert_unsupported(
        fixture
            .proxy
            .get_versions_metadata(&["5000000".to_owned()])
            .await
            .unwrap_err(),
        "get_versions_metadata",
    );
    assert_unsupported(
        fixture
            .proxy
            .get_authors(&["team:abc".to_owned()])
            .await
            .unwrap_err(),
        "get_authors",
    );
    assert!(fixture.called().is_empty());
}

#[tokio::test]
async fn should_report_operations_as_unsupported_when_the_provider_has_no_handler_for_them() {
    let fixture = Fixture::new(
        r#"{
            "contentProviders": [{
                "id": "curseforge",
                "name": "CurseForge",
                "handlers": { "search": "cf_search", "getContent": "cf_get_content" }
            }]
        }"#,
        vec![],
    );

    assert_unsupported(
        fixture
            .proxy
            .resolve_file("238222", None, "1.21.1", None)
            .await
            .unwrap_err(),
        "resolve_file",
    );
    assert_unsupported(
        fixture
            .proxy
            .resolve_install_plan(plan_params(ResolveTarget::New))
            .await
            .unwrap_err(),
        "resolve_install_plan",
    );
    assert_unsupported(
        fixture
            .proxy
            .resolve_pack_metadata(PackSource::Url("https://example.com/pack.zip".into()))
            .await
            .unwrap_err(),
        "resolve_pack_metadata",
    );
    assert!(
        fixture.called().is_empty(),
        "no plugin call without a handler"
    );
}

#[tokio::test]
async fn should_report_optional_operations_as_unsupported_when_the_provider_has_no_handler() {
    let fixture = Fixture::new(
        r#"{
            "contentProviders": [{
                "id": "curseforge",
                "name": "CurseForge",
                "handlers": { "search": "cf_search", "getContent": "cf_get_content" }
            }]
        }"#,
        vec![],
    );

    assert_unsupported(
        fixture
            .proxy
            .list_versions("238222".into())
            .await
            .unwrap_err(),
        "list_versions",
    );
    assert!(fixture.called().is_empty());
}

/// `capabilities.json` names a function the wasm does not export.
#[tokio::test]
async fn should_report_a_handler_as_unsupported_when_it_is_missing_from_the_plugin() {
    let fixture = Fixture::new(
        r#"{
            "contentProviders": [{
                "id": "curseforge",
                "name": "CurseForge",
                "handlers": {
                    "search": "cf_search",
                    "getContent": "cf_get_content",
                    "listVersion": "cf_list_version"
                }
            }]
        }"#,
        vec![],
    );

    assert_unsupported(
        fixture
            .proxy
            .list_versions("238222".into())
            .await
            .unwrap_err(),
        "list_versions",
    );
}

// ─── Proxy: what is refused at the boundary ────────────────────

/// `function_exists` only checks the name; an answer of the wrong shape is caught by decoding.
#[tokio::test]
async fn should_refuse_an_answer_when_it_has_the_wrong_shape() {
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![("cf_resolve_install_plan", encode("not a plan"))],
    );

    assert_refused_with(
        fixture
            .proxy
            .resolve_install_plan(plan_params(existing_target()))
            .await,
        "the plugin exchanged malformed data",
    );
}

#[tokio::test]
async fn should_refuse_a_hash_when_it_is_malformed() {
    let mut entry = jei_entry();
    entry.source = ContentSourceDto::Direct {
        url: "https://edge.forgecdn.net/files/4512/345/jei.jar".into(),
        hash: Some(FileHashDto::Sha1("not-a-digest".into())),
        identity: None,
    };
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![(
            "cf_resolve_install_plan",
            encode(InstallPlanDto {
                target: existing_plan_target("instance"),
                files: vec![entry],
                overrides: Vec::new(),
            }),
        )],
    );

    assert_invalid_answer(
        fixture
            .proxy
            .resolve_install_plan(plan_params(existing_target()))
            .await,
    );
}

/// The game a plan names for the requested instance is passed on as it is: whether the plan may
/// change it is the executor's check, against the instance's pack.
#[tokio::test]
async fn should_pass_on_the_game_when_a_plan_names_one_for_the_instance() {
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![(
            "cf_resolve_install_plan",
            encode(plan_for(
                InstallTargetDto::Existing {
                    instance_id: "instance".into(),
                    game: Some(InstanceGameDto {
                        game_version: "1.21.4".into(),
                        mod_loader: ModLoaderDto::Fabric,
                        loader_version: None,
                    }),
                },
                Vec::new(),
            )),
        )],
    );

    let plan = fixture
        .proxy
        .resolve_install_plan(plan_params(existing_target()))
        .await
        .unwrap();

    assert_eq!(
        plan.target,
        InstallTarget::Existing {
            instance_id: InstanceId::try_from("instance").unwrap(),
            game: Some(InstanceGame {
                game_version: "1.21.4".into(),
                mod_loader: ModLoader::Fabric,
                loader_version: None,
            }),
        }
    );
}

#[tokio::test]
async fn should_refuse_a_plan_when_it_is_for_another_instance() {
    let new_instance = NewInstanceDto {
        name: "Pack".into(),
        game_version: "1.21.1".into(),
        mod_loader: ModLoaderDto::Fabric,
        loader_version: None,
        icon_path: None,
        skip_install_instance: None,
        pack_info: None,
    };

    for (requested, answered) in [
        (existing_target(), existing_plan_target("other")),
        (existing_target(), InstallTargetDto::New(new_instance)),
        (ResolveTarget::New, existing_plan_target("instance")),
    ] {
        let fixture = Fixture::new(
            RESOLVE_ONLY_CAPABILITIES,
            vec![(
                "cf_resolve_install_plan",
                encode(plan_for(answered, Vec::new())),
            )],
        );

        assert_invalid_answer(
            fixture
                .proxy
                .resolve_install_plan(plan_params(requested))
                .await,
        );
    }
}

/// An id that is not a plain directory name would point the plan outside `instances/`.
#[tokio::test]
async fn should_refuse_a_plan_when_its_instance_id_is_not_a_plain_name() {
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![(
            "cf_resolve_install_plan",
            encode(plan_for(existing_plan_target("../x"), Vec::new())),
        )],
    );

    let err = fixture
        .proxy
        .resolve_install_plan(plan_params(existing_target()))
        .await
        .unwrap_err();

    assert!(
        matches!(&err, InstanceError::NotFound { instance_id } if instance_id == "../x"),
        "{err}"
    );
}

/// The keys of the answer name instances; one that is not a plain name is refused.
#[tokio::test]
async fn should_refuse_a_compatibility_answer_when_a_key_is_not_a_plain_name() {
    let fixture = Fixture::new(
        r#"{
            "contentProviders": [{
                "id": "curseforge",
                "name": "CurseForge",
                "handlers": {
                    "search": "cf_search",
                    "getContent": "cf_get_content",
                    "checkCompatibility": "cf_check_compatibility"
                }
            }]
        }"#,
        vec![(
            "cf_check_compatibility",
            encode(HashMap::from([(
                "../x".to_owned(),
                ContentCompatibilityResultDto {
                    is_compatible: true,
                },
            )])),
        )],
    );
    let check_params = ContentCompatibilityCheckParams {
        provider_id: ProviderId {
            plugin_id: PLUGIN_ID.to_owned(),
            capability_id: "curseforge".to_owned(),
        },
        content_item: ContentItem {
            id: "238222".to_owned(),
            slug: "jei".to_owned(),
            name: "JEI".to_owned(),
            description: None,
            long_description: None,
            author: "mezz".to_owned(),
            url: String::new(),
            icon_url: String::new(),
            versions: Vec::new(),
            content_type: ContentType::Mod,
        },
    };

    let err = fixture
        .proxy
        .check_compatibility(&[], &check_params)
        .await
        .unwrap_err();

    assert!(
        matches!(&err, InstanceError::NotFound { instance_id } if instance_id == "../x"),
        "{err}"
    );
}

#[tokio::test]
async fn should_translate_override_paths_when_they_are_in_the_plugin_cache() {
    let answer = |overrides| encode(plan_for(existing_plan_target("instance"), overrides));
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![(
            "cf_resolve_install_plan",
            answer(
                [
                    ("overrides", FileSideDto::Both),
                    ("client-overrides", FileSideDto::Client),
                ]
                .into_iter()
                .map(|(subdir, side)| OverrideSourceDto {
                    location: OverrideLocationDto::Archive {
                        archive_path: "/cache/downloads/pack.zip".into(),
                        subdir: subdir.into(),
                    },
                    side,
                })
                .collect(),
            ),
        )],
    );
    let archive = {
        std::fs::create_dir_all(
            fixture
                .location_info
                .plugin_cache_dir(PLUGIN_ID)
                .join("downloads"),
        )
        .unwrap();
        fixture.cache_file("downloads/pack.zip")
    };

    let plan = fixture
        .proxy
        .resolve_install_plan(plan_params(existing_target()))
        .await
        .unwrap();

    let archive_path = crate::shared::io::infra::canonicalize(archive).unwrap();
    assert_eq!(
        plan.overrides,
        [
            ("overrides", FileSide::Both),
            ("client-overrides", FileSide::Client),
        ]
        .map(|(subdir, side)| OverrideSource::new(
            OverrideLocation::Archive {
                archive_path: archive_path.clone(),
                subdir: subdir.into(),
            },
            side,
        ))
    );
}

#[tokio::test]
async fn should_refuse_override_paths_when_they_are_outside_the_plugin_cache() {
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("secret.txt"), b"secret").unwrap();

    for path in [
        outside.path().to_string_lossy().into_owned(),
        "/instances/other".to_owned(),
        "/cache/../../../..".to_owned(),
    ] {
        let fixture = Fixture::new(
            RESOLVE_ONLY_CAPABILITIES,
            vec![(
                "cf_resolve_install_plan",
                encode(plan_for(
                    existing_plan_target("instance"),
                    vec![OverrideSourceDto {
                        location: OverrideLocationDto::Directory { path: path.clone() },
                        side: FileSideDto::Both,
                    }],
                )),
            )],
        );

        assert_invalid_answer(
            fixture
                .proxy
                .resolve_install_plan(plan_params(existing_target()))
                .await,
        );
    }
}

fn new_instance_with_icon(icon_path: Option<String>) -> InstallTargetDto {
    InstallTargetDto::New(NewInstanceDto {
        name: "Pack".into(),
        game_version: "1.21.1".into(),
        mod_loader: ModLoaderDto::Fabric,
        loader_version: None,
        icon_path,
        skip_install_instance: None,
        pack_info: None,
    })
}

/// The icon of a new instance is a file the plugin names, so it is held to the plugin's own
/// cache like the overrides.
#[tokio::test]
async fn should_translate_the_new_instance_icon_when_it_is_in_the_plugin_cache() {
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![(
            "cf_resolve_install_plan",
            encode(plan_for(
                new_instance_with_icon(Some("/cache/icon.png".into())),
                Vec::new(),
            )),
        )],
    );
    let icon = fixture.cache_file("icon.png");

    let plan = fixture
        .proxy
        .resolve_install_plan(plan_params(ResolveTarget::New))
        .await
        .unwrap();

    let InstallTarget::New(instance) = plan.target else {
        panic!("a new instance was asked for: {:?}", plan.target)
    };
    assert_eq!(
        instance.icon_path,
        Some(
            crate::shared::io::infra::canonicalize(icon)
                .unwrap()
                .to_string_lossy()
                .into_owned()
        )
    );
}

#[tokio::test]
async fn should_keep_a_new_instance_when_it_has_no_icon() {
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![(
            "cf_resolve_install_plan",
            encode(plan_for(new_instance_with_icon(None), Vec::new())),
        )],
    );

    let plan = fixture
        .proxy
        .resolve_install_plan(plan_params(ResolveTarget::New))
        .await
        .unwrap();

    assert!(matches!(
        plan.target,
        InstallTarget::New(NewInstance {
            icon_path: None,
            ..
        })
    ));
}

#[tokio::test]
async fn should_refuse_a_new_instance_icon_when_it_is_outside_the_plugin_cache() {
    let outside = tempfile::tempdir().unwrap();
    let secret = outside.path().join("photo.png");
    std::fs::write(&secret, b"png").unwrap();

    for path in [
        secret.to_string_lossy().into_owned(),
        "/instances/other/icon.png".to_owned(),
        "/cache/../../../photo.png".to_owned(),
    ] {
        let fixture = Fixture::new(
            RESOLVE_ONLY_CAPABILITIES,
            vec![(
                "cf_resolve_install_plan",
                encode(plan_for(
                    new_instance_with_icon(Some(path.clone())),
                    Vec::new(),
                )),
            )],
        );

        assert_invalid_answer(
            fixture
                .proxy
                .resolve_install_plan(plan_params(ResolveTarget::New))
                .await,
        );
    }
}

#[tokio::test]
async fn should_hand_a_pack_file_over_under_the_plugin_path_when_the_plugin_can_read_it() {
    let fixture = Fixture::new(
        RESOLVE_ONLY_CAPABILITIES,
        vec![(
            "cf_resolve_pack_metadata",
            encode(PackMetadataDto {
                name: "Pack".into(),
                version_id: None,
                version_name: None,
                description: None,
                game_version: "1.21.1".into(),
                mod_loader: ModLoaderDto::Fabric,
                loader_version: None,
                file_count: None,
                icon_url: None,
            }),
        )],
    );
    let pack = fixture.cache_file("pack.zip");

    fixture
        .proxy
        .resolve_pack_metadata(PackSource::File(pack))
        .await
        .unwrap();

    assert_eq!(
        fixture.arguments_of_last_call::<PackSourceDto>(),
        PackSourceDto::File("/cache/pack.zip".into())
    );
}

#[tokio::test]
async fn should_refuse_a_pack_file_when_the_plugin_cannot_read_it() {
    let fixture = Fixture::new(RESOLVE_ONLY_CAPABILITIES, vec![]);
    let outside = fixture.root.path().join("Downloads").join("pack.zip");
    std::fs::create_dir_all(outside.parent().unwrap()).unwrap();
    std::fs::write(&outside, b"zip").unwrap();

    assert_refused_with(
        fixture
            .proxy
            .resolve_pack_metadata(PackSource::File(outside))
            .await,
        "a pack file has to be in the plugin's cache or in an instance",
    );
    assert!(fixture.called().is_empty());
}

// ─── Registration ──────────────────────────────────────────────

/// A plugin that implements only `search`, `get_content` and `resolve_*` is registered as a
/// content provider under `<plugin>#<capability>`, and the registered provider calls its
/// functions.
#[tokio::test]
async fn should_register_a_plugin_when_it_implements_only_the_resolve_handlers() {
    let root = tempfile::tempdir().unwrap();
    let location_info = Arc::new(LocationInfo::new(
        root.path().join("settings"),
        root.path().join("config"),
    ));
    let calls = Calls::default();
    let instance = PluginHandle::new(RecordingPlugin {
        answers: HashMap::from([(
            "cf_resolve_file".to_owned(),
            encode(ResolvedFileDto {
                version: "4512345".into(),
                url: "https://edge.forgecdn.net/files/4512/345/jei.jar".into(),
                hash: None,
                file_name: "jei.jar".into(),
                size: None,
                name: None,
            }),
        )]),
        calls: calls.clone(),
        last_answer: vec![],
    });

    let plugins = Arc::new(PluginRegistry::new(Arc::new(NoOpEmitter)));
    plugins.insert(
        PLUGIN_ID.to_owned(),
        Plugin {
            manifest: manifest(),
            capabilities: Some(capabilities(RESOLVE_ONLY_CAPABILITIES)),
            hash: "hash".into(),
            verified: false,
            state: PluginState::Loaded(instance),
        },
    );
    let providers = Arc::new(MemoryCapabilityRegistry::<Arc<dyn ContentProvider>>::default());
    let listener = PluginInfrastructureListener::new(
        plugins,
        Arc::new(MemoryCapabilityRegistry::<Arc<dyn Importer>>::default()),
        Arc::new(MemoryCapabilityRegistry::<Arc<dyn Updater>>::default()),
        providers.clone(),
        location_info,
    );

    listener
        .on_plugin_event(PluginEvent::Edit {
            plugin_id: PLUGIN_ID.to_owned(),
        })
        .await;

    let provider = providers
        .find_by_plugin_and_capability_id(PLUGIN_ID, "curseforge")
        .await
        .expect("registered")
        .capability;
    assert_eq!(provider.metadata().platform.as_deref(), Some("curseforge"));
    assert_eq!(
        provider.provider_id().to_string(),
        "curseforge-plugin#curseforge"
    );

    let resolved = provider
        .resolve_file("238222", Some("4512345"), "1.21.1", None)
        .await
        .unwrap();

    assert_eq!(resolved.file_name, "jei.jar");
    assert_eq!(
        calls
            .lock()
            .unwrap()
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        ["cf_resolve_file"]
    );
}

fn manifest() -> PluginManifest {
    PluginManifest {
        metadata: PluginMetadata {
            id: PLUGIN_ID.into(),
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
