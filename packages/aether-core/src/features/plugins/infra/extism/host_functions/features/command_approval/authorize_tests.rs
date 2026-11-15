//! `authorize_command` as a whole: a real jar in a real cache folder, a stored approval, and an
//! approver that answers as the test says.

use std::{
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use uuid::Uuid;

use super::*;
use crate::features::{
    events::test_support::NoOpEmitter,
    plugins::{
        CommandApprovalAnswer, PendingCommandApproval, PluginSettingsStorage as SettingsStorage,
    },
};
use crate::shared::io::IoError;

const WASM: &str = "wasm-hash-1";

fn plugin() -> PluginId {
    PluginId::try_from("packwiz").unwrap()
}

/// Answers every question with `answer` and remembers what was asked. `before_answer` runs
/// while the user is "still deciding".
struct ScriptedApprover {
    answer: Result<CommandGrant, CommandApprovalRefusal>,
    asked: Mutex<Vec<CommandApprovalRequest>>,
    before_answer: Mutex<Option<Box<dyn FnOnce() + Send>>>,
}

impl ScriptedApprover {
    fn new(answer: Result<CommandGrant, CommandApprovalRefusal>) -> Self {
        Self {
            answer,
            asked: Mutex::default(),
            before_answer: Mutex::default(),
        }
    }

    fn asked(&self) -> Vec<CommandApprovalRequest> {
        self.asked.lock().unwrap().clone()
    }
}

#[async_trait]
impl CommandApprover for ScriptedApprover {
    async fn request(
        &self,
        request: CommandApprovalRequest,
    ) -> Result<CommandGrant, CommandApprovalRefusal> {
        self.asked.lock().unwrap().push(request);
        if let Some(hook) = self.before_answer.lock().unwrap().take() {
            hook();
        }
        self.answer
    }

    fn pending(&self) -> Vec<PendingCommandApproval> {
        vec![]
    }

    fn answer(&self, _id: Uuid, _answer: CommandApprovalAnswer) -> Result<(), PluginError> {
        Ok(())
    }

    fn cancel_plugin(&self, _plugin_id: &PluginId) -> bool {
        false
    }

    fn forget_denials(&self, _plugin_id: &PluginId) {}
}

#[derive(Default)]
struct MemorySettings {
    stored: Mutex<Option<PluginSettings>>,
    unreadable: bool,
    refuses_saves: bool,
}

#[async_trait]
impl SettingsStorage for MemorySettings {
    async fn get(&self, _plugin_id: &PluginId) -> Result<Option<PluginSettings>, PluginError> {
        if self.unreadable {
            return Err(PluginError::Storage(IoError::DeserializationError(
                "broken".into(),
            )));
        }
        Ok(self.stored.lock().unwrap().clone())
    }

    async fn upsert(
        &self,
        _plugin_id: &PluginId,
        settings: &PluginSettings,
    ) -> Result<(), PluginError> {
        if self.refuses_saves {
            return Err(PluginError::Storage(IoError::DeserializationError(
                "read-only".into(),
            )));
        }
        *self.stored.lock().unwrap() = Some(settings.clone());
        Ok(())
    }
}

struct Fixture {
    root: tempfile::TempDir,
    location: LocationInfo,
    registry: PluginRegistry,
    settings: MemorySettings,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let location = LocationInfo::new(root.path().join("settings"), root.path().join("config"));
        // The plugin's cache folder is a mount: a path in it resolves only once it exists.
        std::fs::create_dir_all(root.path().join("config/cache/plugins/packwiz")).unwrap();
        Self {
            root,
            location,
            registry: PluginRegistry::new(Arc::new(NoOpEmitter)),
            settings: MemorySettings::default(),
        }
    }

    /// Where `#/cache/<name>` is on disk. The file need not exist yet, so the path is built
    /// rather than resolved.
    fn host_path(&self, name: &str) -> PathBuf {
        self.root
            .path()
            .join("config/cache/plugins/packwiz")
            .join(name)
    }

    fn write_jar(&self, name: &str, manifest: &str) -> PathBuf {
        let host = self.host_path(name);
        let mut writer = zip::ZipWriter::new(std::fs::File::create(&host).unwrap());
        writer
            .start_file(
                "META-INF/MANIFEST.MF",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
        writer.write_all(manifest.as_bytes()).unwrap();
        writer.finish().unwrap();
        host
    }

    fn stored_approvals(&self) -> Vec<CommandApproval> {
        self.settings
            .stored
            .lock()
            .unwrap()
            .clone()
            .map(|settings| settings.command_approvals)
            .unwrap_or_default()
    }

    async fn authorize(
        &self,
        approver: &dyn CommandApprover,
        wasm: &str,
        args: &[&str],
    ) -> Result<(), PluginError> {
        let command = CommandDto {
            program: "java".into(),
            args: args.iter().map(|arg| (*arg).to_owned()).collect(),
            current_dir: Some("#/cache".into()),
        };
        let host_command = SerializableCommand {
            program: "java".into(),
            args: command.args.clone(),
            current_dir: Some(self.root.path().to_path_buf()),
            env: None,
        };
        authorize_command(
            &plugin(),
            wasm,
            &command,
            &host_command,
            &CommandAuthority {
                approver,
                settings: &self.settings,
                registry: &self.registry,
                location_info: &self.location,
            },
        )
        .await
    }
}

const PLAIN: &str = "Manifest-Version: 1.0\r\n\r\n";
const WITH_CLASS_PATH: &str = "Manifest-Version: 1.0\r\nClass-Path: other.jar\r\n\r\n";

fn refused_with(result: Result<(), PluginError>) -> CommandApprovalRefusal {
    match result {
        Err(PluginError::CommandNotApproved { reason, .. }) => reason,
        other => panic!("expected a refusal, got {other:?}"),
    }
}

#[tokio::test]
async fn should_run_the_command_once_without_saving_when_the_user_allows_it_once() {
    let fixture = Fixture::new();
    fixture.write_jar("a.jar", PLAIN);
    let approver = ScriptedApprover::new(Ok(CommandGrant::Once));
    let args = ["-jar", "#/cache/a.jar"];

    fixture.authorize(&approver, WASM, &args).await.unwrap();
    fixture.authorize(&approver, WASM, &args).await.unwrap();

    assert_eq!(approver.asked().len(), 2);
    assert!(fixture.stored_approvals().is_empty());
}

#[tokio::test]
async fn should_not_ask_again_after_always_for_the_same_jar_and_plugin_build() {
    let fixture = Fixture::new();
    fixture.write_jar("a.jar", PLAIN);
    let approver = ScriptedApprover::new(Ok(CommandGrant::Always));
    let args = ["-jar", "#/cache/a.jar", "--anything", "at-all"];

    fixture.authorize(&approver, WASM, &args).await.unwrap();
    fixture
        .authorize(&approver, WASM, &["-jar", "#/cache/a.jar", "other", "args"])
        .await
        .unwrap();

    assert_eq!(approver.asked().len(), 1);
    let saved = fixture.stored_approvals();
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].plugin_sha256, WASM);
}

#[tokio::test]
async fn should_ask_again_when_the_plugin_build_is_not_the_approved_one() {
    let fixture = Fixture::new();
    fixture.write_jar("a.jar", PLAIN);
    let approver = ScriptedApprover::new(Ok(CommandGrant::Always));
    let args = ["-jar", "#/cache/a.jar"];

    fixture.authorize(&approver, WASM, &args).await.unwrap();
    fixture
        .authorize(&approver, "wasm-hash-2", &args)
        .await
        .unwrap();

    assert_eq!(approver.asked().len(), 2);
}

#[tokio::test]
async fn should_ask_again_when_the_jar_is_not_the_approved_one() {
    let fixture = Fixture::new();
    fixture.write_jar("a.jar", PLAIN);
    let approver = ScriptedApprover::new(Ok(CommandGrant::Always));
    let args = ["-jar", "#/cache/a.jar"];
    fixture.authorize(&approver, WASM, &args).await.unwrap();

    fixture.write_jar("a.jar", "Manifest-Version: 1.0\r\nMain-Class: Evil\r\n\r\n");
    fixture.authorize(&approver, WASM, &args).await.unwrap();

    assert_eq!(approver.asked().len(), 2);
}

#[tokio::test]
async fn should_ask_again_when_the_jvm_options_differ_from_the_approved_ones() {
    let fixture = Fixture::new();
    fixture.write_jar("a.jar", PLAIN);
    let approver = ScriptedApprover::new(Ok(CommandGrant::Always));
    fixture
        .authorize(&approver, WASM, &["-jar", "#/cache/a.jar"])
        .await
        .unwrap();

    fixture
        .authorize(&approver, WASM, &["-Xmx2G", "-jar", "#/cache/a.jar"])
        .await
        .unwrap();

    assert_eq!(approver.asked().len(), 2);
}

#[tokio::test]
async fn should_refuse_the_command_when_the_user_denies_it() {
    let fixture = Fixture::new();
    fixture.write_jar("a.jar", PLAIN);
    let approver = ScriptedApprover::new(Err(CommandApprovalRefusal::Denied));

    let result = fixture
        .authorize(&approver, WASM, &["-jar", "#/cache/a.jar"])
        .await;

    assert_eq!(refused_with(result), CommandApprovalRefusal::Denied);
    assert!(fixture.stored_approvals().is_empty());
}

#[tokio::test]
async fn should_refuse_the_command_when_the_jar_changed_while_the_user_was_asked() {
    let fixture = Fixture::new();
    let host = fixture.write_jar("a.jar", PLAIN);
    let approver = ScriptedApprover::new(Ok(CommandGrant::Always));
    *approver.before_answer.lock().unwrap() = Some(Box::new(move || {
        let mut writer = zip::ZipWriter::new(std::fs::File::create(&host).unwrap());
        writer
            .start_file(
                "META-INF/MANIFEST.MF",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
        writer
            .write_all(b"Manifest-Version: 1.0\r\nMain-Class: Evil\r\n\r\n")
            .unwrap();
        writer.finish().unwrap();
    }));

    let result = fixture
        .authorize(&approver, WASM, &["-jar", "#/cache/a.jar"])
        .await;

    assert_eq!(refused_with(result), CommandApprovalRefusal::JarChanged);
    assert!(
        fixture.stored_approvals().is_empty(),
        "an answer about a jar that is gone must not be kept"
    );
}

#[tokio::test]
async fn should_ask_every_time_and_never_save_when_the_jar_names_other_jars() {
    let fixture = Fixture::new();
    fixture.write_jar("a.jar", WITH_CLASS_PATH);
    // An approver that wrongly grants "always" must not get it kept.
    let approver = ScriptedApprover::new(Ok(CommandGrant::Always));
    let args = ["-jar", "#/cache/a.jar"];

    fixture.authorize(&approver, WASM, &args).await.unwrap();
    fixture.authorize(&approver, WASM, &args).await.unwrap();

    let asked = approver.asked();
    assert_eq!(asked.len(), 2);
    assert_eq!(asked[0].always, Err(OnceOnlyReason::JarHasClassPath));
    assert!(fixture.stored_approvals().is_empty());
}

#[tokio::test]
async fn should_ask_even_when_settings_hold_an_approval_for_a_jar_that_names_other_jars() {
    let fixture = Fixture::new();
    let host = fixture.write_jar("a.jar", WITH_CLASS_PATH);
    let sha256 = hash_file_hex_async(HashAlgorithm::Sha256, &host)
        .await
        .unwrap();
    // A hand-edited settings file.
    *fixture.settings.stored.lock().unwrap() = Some(PluginSettings {
        command_approvals: vec![CommandApproval {
            identity: CommandIdentity {
                jar_sha256: sha256,
                jvm_options: vec![],
            },
            plugin_sha256: WASM.to_owned(),
            jar: "/cache/a.jar".into(),
            approved_at: chrono::Utc::now(),
        }],
        ..PluginSettings::default()
    });
    let approver = ScriptedApprover::new(Ok(CommandGrant::Once));

    fixture
        .authorize(&approver, WASM, &["-jar", "#/cache/a.jar"])
        .await
        .unwrap();

    assert_eq!(approver.asked().len(), 1);
}

#[tokio::test]
async fn should_ask_every_time_for_a_program_outside_the_plugin_cache() {
    let fixture = Fixture::new();
    let approver = ScriptedApprover::new(Ok(CommandGrant::Always));
    let args = ["-jar", "/somewhere/a.jar"];

    fixture.authorize(&approver, WASM, &args).await.unwrap();
    fixture.authorize(&approver, WASM, &args).await.unwrap();

    let asked = approver.asked();
    assert_eq!(asked.len(), 2);
    assert_eq!(asked[0].always, Err(OnceOnlyReason::TargetNotInCache));
    assert!(fixture.stored_approvals().is_empty());
}

#[tokio::test]
async fn should_ask_every_time_when_the_command_reads_options_from_a_file() {
    let fixture = Fixture::new();
    fixture.write_jar("a.jar", PLAIN);
    let approver = ScriptedApprover::new(Ok(CommandGrant::Always));
    let args = ["@#/cache/opts.txt", "-jar", "#/cache/a.jar"];

    fixture.authorize(&approver, WASM, &args).await.unwrap();
    fixture.authorize(&approver, WASM, &args).await.unwrap();

    assert_eq!(approver.asked().len(), 2);
    assert!(fixture.stored_approvals().is_empty());
}

#[tokio::test]
async fn should_ask_and_run_when_the_settings_cannot_be_read() {
    let mut fixture = Fixture::new();
    fixture.settings.unreadable = true;
    fixture.write_jar("a.jar", PLAIN);
    let approver = ScriptedApprover::new(Ok(CommandGrant::Always));

    fixture
        .authorize(&approver, WASM, &["-jar", "#/cache/a.jar"])
        .await
        .unwrap();

    assert_eq!(approver.asked().len(), 1);
    assert!(
        fixture.stored_approvals().is_empty(),
        "saving over settings that do not parse would delete what is in them"
    );
}

#[tokio::test]
async fn should_run_the_command_when_the_approval_cannot_be_saved() {
    let mut fixture = Fixture::new();
    fixture.settings.refuses_saves = true;
    fixture.write_jar("a.jar", PLAIN);
    let approver = ScriptedApprover::new(Ok(CommandGrant::Always));

    let result = fixture
        .authorize(&approver, WASM, &["-jar", "#/cache/a.jar"])
        .await;

    assert!(result.is_ok(), "the user said yes: {result:?}");
}

#[tokio::test]
async fn should_keep_the_other_approvals_when_a_new_one_is_saved() {
    let fixture = Fixture::new();
    fixture.write_jar("a.jar", PLAIN);
    fixture.write_jar("b.jar", "Manifest-Version: 1.0\r\nMain-Class: B\r\n\r\n");
    let approver = ScriptedApprover::new(Ok(CommandGrant::Always));

    fixture
        .authorize(&approver, WASM, &["-jar", "#/cache/a.jar"])
        .await
        .unwrap();
    fixture
        .authorize(&approver, WASM, &["-jar", "#/cache/b.jar"])
        .await
        .unwrap();

    assert_eq!(fixture.stored_approvals().len(), 2);
}
