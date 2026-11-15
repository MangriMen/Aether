//! The user's say on one `run_command`: reading what the command starts, matching it against what
//! was approved for good, and asking when it was not.
//!
//! Runs after the program and the working directory passed their checks, so the question is only
//! ever about a command that could run.

use std::{fs::File, io::Read, path::Path};

use aether_core_plugin_api::v0::CommandDto;
use uuid::Uuid;

use crate::{
    core::app::AetherContainer,
    features::{
        instance::PluginId,
        plugins::{
            CommandApproval, CommandApprovalRefusal, CommandApprovalRequest, CommandGrant,
            CommandIdentity, CommandJar, MAX_APPROVED_JAR_BYTES, OnceOnlyReason, PluginError,
            PluginPath, PluginSettings, PluginsFeature, manifest_has_class_path, parse_jar_launch,
        },
    },
    shared::{
        hash::{HashAlgorithm, infra::hash_file_hex_async},
        serializable_command::domain::SerializableCommand,
    },
};

use super::super::super::super::plugin_utils::{self, MountSet};

/// The largest manifest read from a jar; a real one is a few lines.
const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;

/// What the core could tell about the jar a command starts.
struct Inspected {
    /// The jar's identity, when it is a file the core can vouch for.
    identity: Option<CommandIdentity>,
    jar: Option<CommandJar>,
    always: Result<(), OnceOnlyReason>,
    /// Where the jar is on the host, to hash it again before the command runs.
    host_path: Option<std::path::PathBuf>,
}

/// Lets the command run, or says why it may not. Returns once the command may be started.
pub(crate) async fn authorize_command(
    plugin_id: &PluginId,
    command: &CommandDto,
    host_command: &SerializableCommand,
    container: &AetherContainer,
) -> Result<(), PluginError> {
    let refused = |reason| PluginError::CommandNotApproved {
        plugin_id: plugin_id.clone(),
        reason,
    };

    let inspected = inspect(plugin_id, command, container).await;
    let plugin_sha256 = container.plugin_storage().wasm_sha256(plugin_id).await?;

    if let (Some(identity), Ok(())) = (&inspected.identity, &inspected.always)
        && is_approved(container, plugin_id, identity, &plugin_sha256).await
    {
        return still_the_same_jar(&inspected).await.map_err(refused);
    }

    let request = CommandApprovalRequest {
        id: Uuid::new_v4(),
        plugin_id: plugin_id.clone(),
        java: host_command.program.clone(),
        current_dir: host_command
            .current_dir
            .as_ref()
            .map(|dir| dir.to_string_lossy().into_owned())
            .unwrap_or_default(),
        arguments: command.args.clone(),
        jar: inspected.jar.clone(),
        always: inspected.always.clone(),
    };

    let grant = container
        .command_approver()
        .request(request)
        .await
        .map_err(refused)?;

    // What the user saw is what runs: a jar swapped while the question was open is refused.
    still_the_same_jar(&inspected).await.map_err(refused)?;

    if grant == CommandGrant::Always
        && let Some(identity) = inspected.identity
        && let Some(jar) = inspected.jar
    {
        remember(
            container,
            plugin_id,
            identity,
            plugin_sha256,
            jar.plugin_path,
        )
        .await;
    }

    Ok(())
}

async fn is_approved(
    container: &AetherContainer,
    plugin_id: &PluginId,
    identity: &CommandIdentity,
    plugin_sha256: &str,
) -> bool {
    match container.plugin_settings_storage().get(plugin_id).await {
        Ok(Some(settings)) => settings
            .command_approvals
            .iter()
            .any(|approval| approval.covers(identity, plugin_sha256)),
        Ok(None) => false,
        Err(err) => {
            // A broken settings file is repaired from the settings screen; here it only means
            // there is nothing to go by, so the user is asked.
            tracing::warn!(target: "plugin", "[{plugin_id}]: settings unreadable, asking: {err}");
            false
        }
    }
}

/// Saves "always". A failed save is not a failed command: the user said yes, so it runs, and
/// they are asked again next time.
async fn remember(
    container: &AetherContainer,
    plugin_id: &PluginId,
    identity: CommandIdentity,
    plugin_sha256: String,
    jar: String,
) {
    let registry = container.plugin_registry();
    let _lock = registry.lock_settings(plugin_id).await;
    let storage = container.plugin_settings_storage();

    let mut settings: PluginSettings = match storage.get(plugin_id).await {
        Ok(settings) => settings.unwrap_or_default(),
        Err(err) => {
            // Saving over a file that does not parse would delete what the user wrote in it.
            tracing::warn!(
                target: "plugin",
                "[{plugin_id}]: approval not saved, settings unreadable: {err}"
            );
            return;
        }
    };
    settings
        .command_approvals
        .retain(|approval| approval.identity != identity);
    settings.command_approvals.push(CommandApproval {
        identity,
        plugin_sha256,
        jar,
        approved_at: chrono::Utc::now(),
    });

    if let Err(err) = storage.upsert(plugin_id, &settings).await {
        tracing::warn!(target: "plugin", "[{plugin_id}]: approval not saved: {err}");
    }
}

async fn still_the_same_jar(inspected: &Inspected) -> Result<(), CommandApprovalRefusal> {
    let (Some(path), Some(jar)) = (&inspected.host_path, &inspected.jar) else {
        return Ok(());
    };
    match hash_file_hex_async(HashAlgorithm::Sha256, path).await {
        Ok(hash) if hash == jar.sha256 => Ok(()),
        _ => Err(CommandApprovalRefusal::JarChanged),
    }
}

async fn inspect(
    plugin_id: &PluginId,
    command: &CommandDto,
    container: &AetherContainer,
) -> Inspected {
    let once_only = |reason| Inspected {
        identity: None,
        jar: None,
        always: Err(reason),
        host_path: None,
    };

    let launch = match parse_jar_launch(&command.args) {
        Ok(launch) => launch,
        Err(reason) => return once_only(reason),
    };

    // Only a file in the plugin's own cache: that is where the core can tell what the file is.
    let in_cache = launch
        .target
        .strip_prefix('#')
        .and_then(|path| PluginPath::parse(path).ok());
    let Some(plugin_path) = in_cache else {
        return once_only(OnceOnlyReason::TargetNotInCache);
    };
    let Ok(host_path) = plugin_utils::resolve_plugin_path(
        &container.location_info(),
        plugin_id,
        &plugin_path,
        MountSet::CACHE,
    ) else {
        return once_only(OnceOnlyReason::TargetNotInCache);
    };

    let size = match std::fs::metadata(&host_path) {
        Ok(meta) if meta.is_file() => meta.len(),
        _ => return once_only(OnceOnlyReason::JarUnreadable),
    };
    if size > MAX_APPROVED_JAR_BYTES {
        return once_only(OnceOnlyReason::JarTooLarge);
    }
    let Ok(sha256) = hash_file_hex_async(HashAlgorithm::Sha256, &host_path).await else {
        return once_only(OnceOnlyReason::JarUnreadable);
    };

    let always = match jar_has_class_path(&host_path) {
        Ok(false) => Ok(()),
        Ok(true) => Err(OnceOnlyReason::JarHasClassPath),
        Err(()) => Err(OnceOnlyReason::JarUnreadable),
    };

    Inspected {
        identity: Some(CommandIdentity {
            jar_sha256: sha256.clone(),
            jvm_options: launch.jvm_options,
        }),
        jar: Some(CommandJar {
            plugin_path: plugin_path.as_str().to_owned(),
            sha256,
            size,
        }),
        always,
        host_path: Some(host_path),
    }
}

/// Whether the jar's manifest names more jars. `Err` when it cannot be read as a jar at all.
fn jar_has_class_path(path: &Path) -> Result<bool, ()> {
    let file = File::open(path).map_err(|_| ())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|_| ())?;
    let entry = match archive.by_name("META-INF/MANIFEST.MF") {
        Ok(entry) => entry,
        // A jar without a manifest has no `Class-Path`, and no `Main-Class` either.
        Err(zip::result::ZipError::FileNotFound) => return Ok(false),
        Err(_) => return Err(()),
    };

    let mut manifest = String::new();
    entry
        .take(MAX_MANIFEST_BYTES + 1)
        .read_to_string(&mut manifest)
        .map_err(|_| ())?;
    if manifest.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(());
    }
    Ok(manifest_has_class_path(&manifest))
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn jar_with(entries: &[(&str, &str)]) -> tempfile::NamedTempFile {
        let file = tempfile::NamedTempFile::new().unwrap();
        let mut writer = zip::ZipWriter::new(file.reopen().unwrap());
        for (name, body) in entries {
            writer
                .start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(body.as_bytes()).unwrap();
        }
        writer.finish().unwrap();
        file
    }

    #[test]
    fn should_find_the_class_path_when_the_manifest_has_one() {
        let jar = jar_with(&[(
            "META-INF/MANIFEST.MF",
            "Manifest-Version: 1.0\r\nClass-Path: lib/a.jar\r\n\r\n",
        )]);

        assert_eq!(jar_has_class_path(jar.path()), Ok(true));
    }

    #[test]
    fn should_not_find_a_class_path_when_the_manifest_has_none() {
        let jar = jar_with(&[(
            "META-INF/MANIFEST.MF",
            "Manifest-Version: 1.0\r\nMain-Class: a.B\r\n\r\n",
        )]);

        assert_eq!(jar_has_class_path(jar.path()), Ok(false));
    }

    #[test]
    fn should_not_find_a_class_path_when_the_jar_has_no_manifest() {
        let jar = jar_with(&[("a/B.class", "x")]);

        assert_eq!(jar_has_class_path(jar.path()), Ok(false));
    }

    #[test]
    fn should_report_unreadable_when_the_file_is_not_a_jar() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(b"not a zip").unwrap();

        assert_eq!(jar_has_class_path(file.path()), Err(()));
    }
}
