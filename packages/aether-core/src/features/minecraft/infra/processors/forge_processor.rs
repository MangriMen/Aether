use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    path::Path,
    sync::Arc,
};

use async_trait::async_trait;
use tokio::process::Command;

use crate::{
    features::{
        events::{ProgressBarId, ProgressService, ProgressServiceExt},
        java::Java,
        minecraft::{
            MinecraftDomainError, ModLoaderProcessor, get_class_paths_jar, get_lib_path, modded,
            vanilla::VersionInfo,
        },
        settings::LocationInfo,
    },
    processor_rules,
    shared::io::domain::IoError,
    with_mut_ref,
};

pub struct ForgeProcessor<PS: ProgressService> {
    progress_service: Arc<PS>,
    location_info: Arc<LocationInfo>,
}

impl<PS: ProgressService> ForgeProcessor<PS> {
    pub fn new(progress_service: Arc<PS>, location_info: Arc<LocationInfo>) -> Self {
        Self {
            progress_service,
            location_info,
        }
    }

    async fn run_single_processor(
        processor: &modded::Processor,
        data: &HashMap<String, modded::SidedDataEntry>,
        libraries_dir: &Path,
        java_version: &Java,
    ) -> Result<(), MinecraftDomainError> {
        log::debug!("Running forge processor {}", processor.jar);

        let class_path: Vec<String> = with_mut_ref!(cp = processor.classpath.clone() => {
            cp.push(processor.jar.clone());
        });

        let class_path_arg =
            get_class_paths_jar(libraries_dir, &class_path, java_version.architecture())?;

        let processor_jar_path = get_lib_path(libraries_dir, &processor.jar, false)?;
        let processor_main_class = get_processor_main_class(processor_jar_path).await?.ok_or({
            MinecraftDomainError::ProcessorFailed {
                reason: format!("Could not find processor main class for {}", processor.jar),
            }
        })?;

        let processor_args = get_processor_arguments(libraries_dir, &processor.args, data)?;

        let output = Command::new(java_version.path())
            .arg("-cp")
            .arg(class_path_arg)
            .arg(processor_main_class)
            .args(processor_args)
            .output()
            .await
            .map_err(|e| IoError::with_path(e, java_version.path()))
            .map_err(|err| MinecraftDomainError::ProcessorFailed {
                reason: format!("Error running processor: {err}"),
            })?;

        if !output.status.success() {
            return Err(MinecraftDomainError::ProcessorFailed {
                reason: format!(
                    "Processor error: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        Ok(())
    }
}

#[async_trait]
impl<PS: ProgressService> ModLoaderProcessor for ForgeProcessor<PS> {
    async fn run(
        &self,
        game_version: String,
        version_jar: String,
        minecraft_dir: &Path,
        version_info: &mut VersionInfo,
        java_version: &Java,
        loading_bar: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftDomainError> {
        let Some(processors) = &version_info.processors else {
            return Ok(());
        };

        let client_path = self
            .location_info
            .version_dir(&version_jar)
            .join(format!("{version_jar}.jar"));

        let libraries_dir = self.location_info.libraries_dir();

        let Some(ref mut data) = version_info.data else {
            return Ok(());
        };

        processor_rules! {
            data;
            "SIDE":
                client => "client",
                server => "";
            "MINECRAFT_JAR":
                client => client_path.to_string_lossy(),
                server => "";
            "MINECRAFT_VERSION":
                client => game_version.clone(),
                server => "";
            "ROOT":
                client => minecraft_dir.to_string_lossy(),
                server => "";
            "LIBRARY_DIR":
                client => libraries_dir.to_string_lossy(),
                server => "";
        }

        if let Some(loading_bar) = loading_bar {
            self.progress_service
                .emit_progress_safe(loading_bar, 0.0, Some("Running forge processors"))
                .await;
        }

        let total_processors = processors.len();
        for (index, processor) in processors.iter().enumerate() {
            if let Some(sides) = &processor.sides
                && !sides.contains(&String::from("client"))
            {
                continue;
            }

            Self::run_single_processor(processor, data, &libraries_dir, java_version).await?;

            if let Some(loading_bar) = loading_bar {
                #[allow(clippy::cast_precision_loss)]
                let progress = 30.0 / total_processors as f64;
                let message = format!("Running forge processor {}/{}", index + 1, total_processors);
                self.progress_service
                    .emit_progress_safe(loading_bar, progress, Some(&message))
                    .await;
            }
        }

        Ok(())
    }
}

/// Resolves `[group:artifact:version]` to a library path; any other text is returned as is.
fn resolve_library_reference(
    libraries_path: &Path,
    text: &str,
) -> Result<String, MinecraftDomainError> {
    match text
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    {
        Some(coordinate) => get_lib_path(libraries_path, coordinate, true),
        None => Ok(text.to_owned()),
    }
}

fn process_argument(
    libraries_path: &Path,
    argument: &str,
    data: &HashMap<String, modded::SidedDataEntry>,
) -> Result<String, MinecraftDomainError> {
    if argument.starts_with('[') && argument.ends_with(']') {
        return resolve_library_reference(libraries_path, argument);
    }

    let mut result = String::with_capacity(argument.len());
    let mut rest = argument;
    while let Some(open) = rest.find('{') {
        result.push_str(&rest[..open]);
        let after_open = &rest[open + 1..];
        let entry = after_open
            .find('}')
            .and_then(|close| data.get(&after_open[..close]).map(|entry| (close, entry)));
        if let Some((close, entry)) = entry {
            result.push_str(&resolve_library_reference(libraries_path, &entry.client)?);
            rest = &after_open[close + 1..];
        } else {
            result.push('{');
            rest = after_open;
        }
    }
    result.push_str(rest);

    if result.contains('{') && result.contains('}') {
        return Err(MinecraftDomainError::ProcessorFailed {
            reason: format!("Argument contains unresolved placeholders: {result}"),
        });
    }

    Ok(result)
}

pub(super) fn get_processor_arguments<T: AsRef<str>>(
    libraries_path: &Path,
    arguments: &[T],
    data: &HashMap<String, modded::SidedDataEntry>,
) -> Result<Vec<String>, MinecraftDomainError> {
    arguments
        .iter()
        .map(|argument| process_argument(libraries_path, argument.as_ref(), data))
        .collect()
}

pub(super) async fn get_processor_main_class(
    path: String,
) -> Result<Option<String>, MinecraftDomainError> {
    tokio::task::spawn_blocking(move || read_main_class(&path))
        .await
        .map_err(|err| MinecraftDomainError::ProcessorFailed {
            reason: err.to_string(),
        })?
}

fn read_main_class(path: &str) -> Result<Option<String>, MinecraftDomainError> {
    let file = std::fs::File::open(path).map_err(|err| IoError::with_path(err, path))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|_| MinecraftDomainError::ProcessorFailed {
            reason: format!("Cannot read processor at {path}"),
        })?;
    let manifest = archive.by_name("META-INF/MANIFEST.MF").map_err(|_| {
        MinecraftDomainError::ProcessorFailed {
            reason: format!("Cannot read processor manifest at {path}"),
        }
    })?;

    for line in BufReader::new(manifest).lines() {
        let line = line.map_err(|err| IoError::with_path(err, path))?;
        if let Some(value) = line.trim().strip_prefix("Main-Class:") {
            return Ok(Some(value.trim().to_owned()));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn entry(client: &str) -> modded::SidedDataEntry {
        modded::SidedDataEntry {
            client: client.to_string(),
            server: "server-value".to_string(),
        }
    }

    fn data(entries: &[(&str, &str)]) -> HashMap<String, modded::SidedDataEntry> {
        entries
            .iter()
            .map(|(k, v)| ((*k).to_string(), entry(v)))
            .collect()
    }

    fn libs_dir() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let libs = tmp.path().join("libraries");
        std::fs::create_dir_all(&libs).unwrap();
        (tmp, libs)
    }

    fn canon(p: &Path) -> String {
        crate::shared::io::infra::canonicalize(p)
            .unwrap()
            .to_string_lossy()
            .to_string()
    }

    fn make_lib(libs: &Path, rel: &str) -> String {
        let p = libs.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, b"jar").unwrap();
        canon(&p)
    }

    fn unresolved(err: &MinecraftDomainError) -> bool {
        matches!(
            err,
            MinecraftDomainError::ProcessorFailed { reason }
                if reason.contains("unresolved placeholders")
        )
    }

    // ---- get_processor_arguments ----

    #[test]
    fn processor_arguments_empty_list_gives_empty_vec() {
        let (_t, libs) = libs_dir();
        let none: [&str; 0] = [];
        assert!(
            get_processor_arguments(&libs, &none, &data(&[]))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn processor_arguments_plain_values_pass_through_in_order() {
        let (_t, libs) = libs_dir();
        let out = get_processor_arguments(
            &libs,
            &["--task", "MERGE_MAPPING", "--debug", ""],
            &data(&[("SIDE", "client")]),
        )
        .unwrap();
        assert_eq!(out, vec!["--task", "MERGE_MAPPING", "--debug", ""]);
    }

    #[test]
    fn processor_arguments_accept_owned_strings() {
        let (_t, libs) = libs_dir();
        let out = get_processor_arguments(&libs, &["--x".to_string()], &data(&[])).unwrap();
        assert_eq!(out, vec!["--x"]);
    }

    #[test]
    fn processor_arguments_bracket_form_resolves_existing_library() {
        let (_t, libs) = libs_dir();
        let path = make_lib(
            &libs,
            "net/minecraftforge/forge/1.20-47/forge-1.20-47-client.jar",
        );
        let out = get_processor_arguments(
            &libs,
            &["[net.minecraftforge:forge:1.20-47:client]"],
            &data(&[]),
        )
        .unwrap();
        assert_eq!(out, vec![path]);
    }

    #[test]
    fn processor_arguments_bracket_form_supports_extension() {
        let (_t, libs) = libs_dir();
        let path = make_lib(
            &libs,
            "de/oceanlabs/mcp/mcp_config/1.20/mcp_config-1.20.zip",
        );
        let out = get_processor_arguments(
            &libs,
            &["[de.oceanlabs.mcp:mcp_config:1.20@zip]"],
            &data(&[]),
        )
        .unwrap();
        assert_eq!(out, vec![path]);
    }

    #[test]
    fn processor_arguments_bracket_form_missing_library_gives_uncanonicalized_path() {
        let (_t, libs) = libs_dir();
        let out = get_processor_arguments(&libs, &["[g:a:1]"], &data(&[])).unwrap();
        let mut expected = libs.clone();
        expected.push("g/a/1/a-1.jar");
        assert_eq!(out, vec![expected.to_string_lossy().to_string()]);
    }

    #[test]
    fn processor_arguments_bracket_form_invalid_coordinate_fails() {
        let (_t, libs) = libs_dir();
        let err = get_processor_arguments(&libs, &["[broken]"], &data(&[])).unwrap_err();
        assert!(matches!(err, MinecraftDomainError::ParseFailed { .. }));
    }

    #[test]
    fn processor_arguments_unbalanced_bracket_is_plain_text() {
        let (_t, libs) = libs_dir();
        let out = get_processor_arguments(&libs, &["[g:a:1", "g:a:1]"], &data(&[])).unwrap();
        assert_eq!(out, vec!["[g:a:1", "g:a:1]"]);
    }

    #[test]
    fn processor_arguments_key_placeholder_uses_client_value() {
        let (_t, libs) = libs_dir();
        let out = get_processor_arguments(
            &libs,
            &["{SIDE}", "{MINECRAFT_VERSION}"],
            &data(&[("SIDE", "client"), ("MINECRAFT_VERSION", "1.20.1")]),
        )
        .unwrap();
        assert_eq!(out, vec!["client", "1.20.1"]);
    }

    #[test]
    fn processor_arguments_key_placeholder_inside_composite_value() {
        let (_t, libs) = libs_dir();
        let out = get_processor_arguments(
            &libs,
            &["{ROOT}/libraries/", "--in={ROOT}/a/{NAME}.txt"],
            &data(&[("ROOT", "/game"), ("NAME", "x")]),
        )
        .unwrap();
        assert_eq!(out, vec!["/game/libraries/", "--in=/game/a/x.txt"]);
    }

    #[test]
    fn processor_arguments_repeated_placeholder_is_replaced_everywhere() {
        let (_t, libs) = libs_dir();
        let out = get_processor_arguments(&libs, &["{A}-{A}"], &data(&[("A", "z")])).unwrap();
        assert_eq!(out, vec!["z-z"]);
    }

    #[test]
    fn processor_arguments_data_value_in_brackets_resolves_library() {
        let (_t, libs) = libs_dir();
        let path = make_lib(&libs, "g/a/1/a-1.jar");
        let out = get_processor_arguments(
            &libs,
            &["{MAPPINGS}", "--out={MAPPINGS}"],
            &data(&[("MAPPINGS", "[g:a:1]")]),
        )
        .unwrap();
        assert_eq!(out, vec![path.clone(), format!("--out={path}")]);
    }

    #[test]
    fn processor_arguments_data_value_in_brackets_missing_library_is_uncanonicalized() {
        let (_t, libs) = libs_dir();
        let out = get_processor_arguments(&libs, &["{M}"], &data(&[("M", "[g:a:1]")])).unwrap();
        let mut expected = libs.clone();
        expected.push("g/a/1/a-1.jar");
        assert_eq!(out, vec![expected.to_string_lossy().to_string()]);
    }

    #[test]
    fn processor_arguments_data_value_with_unbalanced_bracket_is_literal() {
        let (_t, libs) = libs_dir();
        let out = get_processor_arguments(&libs, &["{M}"], &data(&[("M", "[not-closed")])).unwrap();
        assert_eq!(out, vec!["[not-closed"]);
    }

    #[test]
    fn processor_arguments_data_value_invalid_library_fails() {
        let (_t, libs) = libs_dir();
        let err =
            get_processor_arguments(&libs, &["{M}"], &data(&[("M", "[broken]")])).unwrap_err();
        assert!(matches!(err, MinecraftDomainError::ParseFailed { .. }));
    }

    #[test]
    fn processor_arguments_unused_data_entries_are_not_resolved() {
        // An invalid library reference in an unused entry must not matter
        let (_t, libs) = libs_dir();
        let out =
            get_processor_arguments(&libs, &["--plain"], &data(&[("BAD", "[broken]")])).unwrap();
        assert_eq!(out, vec!["--plain"]);
    }

    #[test]
    fn processor_arguments_unknown_placeholder_fails() {
        let (_t, libs) = libs_dir();
        let err = get_processor_arguments(&libs, &["{UNKNOWN}"], &data(&[("SIDE", "client")]))
            .unwrap_err();
        assert!(unresolved(&err));
    }

    #[test]
    fn processor_arguments_message_contains_partially_resolved_argument() {
        let (_t, libs) = libs_dir();
        let err = get_processor_arguments(&libs, &["{A}/{B}"], &data(&[("A", "x")])).unwrap_err();
        assert!(matches!(
            err,
            MinecraftDomainError::ProcessorFailed { ref reason } if reason.ends_with("x/{B}")
        ));
    }

    #[test]
    fn processor_arguments_any_brace_pair_after_substitution_fails() {
        let (_t, libs) = libs_dir();
        for arg in ["{}", "a{b", "a}b{c", "json={\"k\":1}"] {
            let res = get_processor_arguments(&libs, &[arg], &data(&[]));
            if arg == "a{b" {
                // only an opening brace: nothing to complain about
                assert_eq!(res.unwrap(), vec!["a{b"]);
            } else {
                assert!(unresolved(&res.unwrap_err()), "arg {arg}");
            }
        }
    }

    #[test]
    fn processor_arguments_braces_coming_from_data_values_fail() {
        let (_t, libs) = libs_dir();
        let err = get_processor_arguments(&libs, &["{A}"], &data(&[("A", "{B}")])).unwrap_err();
        assert!(unresolved(&err));
    }

    #[test]
    fn processor_arguments_error_in_any_argument_fails_whole_call() {
        let (_t, libs) = libs_dir();
        let err = get_processor_arguments(&libs, &["ok", "{NOPE}", "ok2"], &data(&[])).unwrap_err();
        assert!(unresolved(&err));
    }

    #[test]
    fn processor_arguments_use_client_side_only() {
        let (_t, libs) = libs_dir();
        let out = get_processor_arguments(&libs, &["{K}"], &data(&[("K", "c")])).unwrap();
        assert_eq!(out, vec!["c"]);
    }

    // ---- get_processor_main_class ----

    fn write_jar(dir: &Path, name: &str, manifest: Option<&str>) -> String {
        let path = dir.join(name);
        let file = std::fs::File::create(&path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let opts = zip::write::SimpleFileOptions::default();
        if let Some(m) = manifest {
            zip.start_file("META-INF/MANIFEST.MF", opts).unwrap();
            zip.write_all(m.as_bytes()).unwrap();
        } else {
            zip.start_file("other.txt", opts).unwrap();
            zip.write_all(b"x").unwrap();
        }
        zip.finish().unwrap();
        path.to_string_lossy().to_string()
    }

    #[tokio::test]
    async fn main_class_is_read_from_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let jar = write_jar(
            tmp.path(),
            "a.jar",
            Some("Manifest-Version: 1.0\r\nMain-Class: net.example.Main\r\nCreated-By: x\r\n"),
        );
        assert_eq!(
            get_processor_main_class(jar).await.unwrap(),
            Some("net.example.Main".to_string())
        );
    }

    #[tokio::test]
    async fn main_class_value_is_trimmed() {
        let tmp = tempfile::tempdir().unwrap();
        let jar = write_jar(tmp.path(), "a.jar", Some("Main-Class:    a.B   \n"));
        assert_eq!(
            get_processor_main_class(jar).await.unwrap(),
            Some("a.B".to_string())
        );
    }

    #[tokio::test]
    async fn main_class_first_occurrence_wins() {
        let tmp = tempfile::tempdir().unwrap();
        let jar = write_jar(
            tmp.path(),
            "a.jar",
            Some("Main-Class: first\nMain-Class: second\n"),
        );
        assert_eq!(
            get_processor_main_class(jar).await.unwrap(),
            Some("first".to_string())
        );
    }

    #[tokio::test]
    async fn main_class_absent_gives_none() {
        let tmp = tempfile::tempdir().unwrap();
        let jar = write_jar(tmp.path(), "a.jar", Some("Manifest-Version: 1.0\n"));
        assert_eq!(get_processor_main_class(jar).await.unwrap(), None);
    }

    #[tokio::test]
    async fn main_class_key_is_case_sensitive() {
        let tmp = tempfile::tempdir().unwrap();
        let jar = write_jar(tmp.path(), "a.jar", Some("main-class: a.B\n"));
        assert_eq!(get_processor_main_class(jar).await.unwrap(), None);
    }

    #[tokio::test]
    async fn main_class_missing_manifest_is_processor_error() {
        let tmp = tempfile::tempdir().unwrap();
        let jar = write_jar(tmp.path(), "a.jar", None);
        let err = get_processor_main_class(jar).await.unwrap_err();
        assert!(matches!(
            err,
            MinecraftDomainError::ProcessorFailed { ref reason }
                if reason.contains("Cannot read processor manifest")
        ));
    }

    #[tokio::test]
    async fn main_class_not_a_zip_is_processor_error() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("bad.jar");
        std::fs::write(&path, b"not a zip").unwrap();
        let err = get_processor_main_class(path.to_string_lossy().to_string())
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            MinecraftDomainError::ProcessorFailed { ref reason }
                if reason.contains("Cannot read processor at")
        ));
    }

    #[tokio::test]
    async fn main_class_missing_file_is_error() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("missing.jar");
        assert!(
            get_processor_main_class(path.to_string_lossy().to_string())
                .await
                .is_err()
        );
    }
}
