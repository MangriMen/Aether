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

    /// Emits a progress step when the caller supplied a progress bar.
    async fn report(&self, bar: Option<&ProgressBarId>, delta: f64, text: &str) {
        if let Some(bar) = bar {
            self.progress_service
                .emit_progress_safe(bar, delta, Some(text))
                .await;
        }
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
        let (Some(processors), Some(data)) =
            (version_info.processors.as_ref(), version_info.data.as_mut())
        else {
            return Ok(());
        };

        let libraries_dir = self.location_info.libraries_dir();
        let client_jar = self
            .location_info
            .version_dir(&version_jar)
            .join(format!("{version_jar}.jar"));

        fill_client_data(
            data,
            &client_jar,
            minecraft_dir,
            &libraries_dir,
            &game_version,
        );

        self.report(loading_bar, 0.0, "Running forge processors")
            .await;

        let total = processors.len();
        let step = progress_step(total);
        for (position, processor) in processors.iter().enumerate() {
            if !runs_on_client(processor) {
                continue;
            }

            execute_processor(processor, data, &libraries_dir, java_version).await?;

            let text = format!("Running forge processor {}/{total}", position + 1);
            self.report(loading_bar, step, &text).await;
        }

        Ok(())
    }
}

/// Fills the well-known Forge data keys; the server side stays empty.
fn fill_client_data(
    data: &mut HashMap<String, modded::SidedDataEntry>,
    client_jar: &Path,
    minecraft_dir: &Path,
    libraries_dir: &Path,
    game_version: &str,
) {
    processor_rules! {
        data;
        "SIDE":
            client => "client",
            server => "";
        "MINECRAFT_JAR":
            client => client_jar.to_string_lossy(),
            server => "";
        "MINECRAFT_VERSION":
            client => game_version,
            server => "";
        "ROOT":
            client => minecraft_dir.to_string_lossy(),
            server => "";
        "LIBRARY_DIR":
            client => libraries_dir.to_string_lossy(),
            server => "";
    }
}

/// Resolves classpath, entry point and arguments of one processor and runs it with java.
async fn execute_processor(
    processor: &modded::Processor,
    data: &HashMap<String, modded::SidedDataEntry>,
    libraries_dir: &Path,
    java_version: &Java,
) -> Result<(), MinecraftDomainError> {
    log::debug!("Running forge processor {}", processor.jar);

    let class_path_entries: Vec<String> = processor
        .classpath
        .iter()
        .chain(std::iter::once(&processor.jar))
        .cloned()
        .collect();
    let class_path = get_class_paths_jar(
        libraries_dir,
        &class_path_entries,
        java_version.architecture(),
    )?;

    let jar_path = get_lib_path(libraries_dir, &processor.jar, false)?;
    let Some(main_class) = get_processor_main_class(jar_path).await? else {
        return Err(MinecraftDomainError::ProcessorFailed {
            reason: format!("Could not find processor main class for {}", processor.jar),
        });
    };

    let args = get_processor_arguments(libraries_dir, &processor.args, data)?;

    let output = build_processor_command(java_version.path(), &class_path, &main_class, &args)
        .output()
        .await
        .map_err(|err| MinecraftDomainError::ProcessorFailed {
            reason: format!(
                "Error running processor: {}",
                IoError::with_path(err, java_version.path())
            ),
        })?;

    if output.status.success() {
        return Ok(());
    }
    Err(MinecraftDomainError::ProcessorFailed {
        reason: format!(
            "Processor error: {}",
            String::from_utf8_lossy(&output.stderr)
        ),
    })
}

/// `java -cp <class_path> <main_class> <args...>`
fn build_processor_command(
    java_path: &str,
    class_path: &str,
    main_class: &str,
    args: &[String],
) -> Command {
    let mut command = Command::new(java_path);
    command.args(["-cp", class_path, main_class]).args(args);
    command
}

/// A processor without `sides` runs everywhere; otherwise `client` must be listed.
fn runs_on_client(processor: &modded::Processor) -> bool {
    processor
        .sides
        .as_ref()
        .is_none_or(|sides| sides.iter().any(|s| s == "client"))
}

/// Share of the overall progress (in percent) that all processors together may fill.
const PROCESSORS_PROGRESS_SPAN: f64 = 30.0;

/// Progress increment reported after each processor run.
fn progress_step(total_processors: usize) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let total = total_processors as f64;
    PROCESSORS_PROGRESS_SPAN / total
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
    use rstest::rstest;
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
    fn client_data_gets_all_known_keys_with_empty_server_side() {
        let mut data = HashMap::new();
        fill_client_data(
            &mut data,
            Path::new("/v/1.20/1.20.jar"),
            Path::new("/mc"),
            Path::new("/libs"),
            "1.20",
        );
        assert_eq!(data.len(), 5);
        assert_eq!(data["SIDE"].client, "client");
        assert_eq!(data["MINECRAFT_VERSION"].client, "1.20");
        assert_eq!(
            data["MINECRAFT_JAR"].client,
            Path::new("/v/1.20/1.20.jar").to_string_lossy()
        );
        assert_eq!(data["ROOT"].client, Path::new("/mc").to_string_lossy());
        assert_eq!(
            data["LIBRARY_DIR"].client,
            Path::new("/libs").to_string_lossy()
        );
        assert!(data.values().all(|entry| entry.server.is_empty()));
    }

    #[test]
    fn processor_command_is_java_cp_main_then_args() {
        let args = vec!["--a".to_string(), "b".to_string()];
        let command = build_processor_command("java", "x.jar;y.jar", "Main", &args);
        let std_command = command.as_std();
        assert_eq!(std_command.get_program(), "java");
        let got: Vec<_> = std_command.get_args().collect();
        assert_eq!(got, ["-cp", "x.jar;y.jar", "Main", "--a", "b"]);
    }

    fn processor_with_sides(sides: Option<&[&str]>) -> modded::Processor {
        modded::Processor {
            jar: "a:b:1".into(),
            classpath: vec![],
            args: vec![],
            outputs: None,
            sides: sides.map(|s| s.iter().map(ToString::to_string).collect()),
        }
    }

    #[rstest]
    #[case::without_sides_runs_on_client(None, true)]
    #[case::with_client_side_runs(Some(&["server", "client"][..]), true)]
    #[case::server_only_is_skipped(Some(&["server"][..]), false)]
    #[case::empty_sides_is_skipped(Some(&[][..]), false)]
    #[case::extract_only_is_skipped(Some(&["extract"][..]), false)]
    fn processor_runs_on_client_by_sides(#[case] sides: Option<&[&str]>, #[case] expected: bool) {
        assert_eq!(runs_on_client(&processor_with_sides(sides)), expected);
    }

    #[test]
    fn progress_step_splits_thirty_percent_evenly() {
        assert_eq!(progress_step(1), 30.0);
        assert_eq!(progress_step(3), 10.0);
        assert!((progress_step(7) * 7.0 - 30.0).abs() < 1e-9);
    }

    #[rstest]
    #[case::empty_list_gives_empty_vec(&[], &[], &[])]
    #[case::plain_values_pass_through_in_order(
        &["--task", "MERGE_MAPPING", "--debug", ""],
        &[("SIDE", "client")],
        &["--task", "MERGE_MAPPING", "--debug", ""]
    )]
    #[case::unbalanced_bracket_is_plain_text(
        &["[g:a:1", "g:a:1]"],
        &[],
        &["[g:a:1", "g:a:1]"]
    )]
    #[case::key_placeholder_uses_client_value(
        &["{SIDE}", "{MINECRAFT_VERSION}"],
        &[("SIDE", "client"), ("MINECRAFT_VERSION", "1.20.1")],
        &["client", "1.20.1"]
    )]
    #[case::key_placeholder_inside_composite_value(
        &["{ROOT}/libraries/", "--in={ROOT}/a/{NAME}.txt"],
        &[("ROOT", "/game"), ("NAME", "x")],
        &["/game/libraries/", "--in=/game/a/x.txt"]
    )]
    #[case::repeated_placeholder_is_replaced_everywhere(&["{A}-{A}"], &[("A", "z")], &["z-z"])]
    #[case::data_value_with_unbalanced_bracket_is_literal(
        &["{M}"],
        &[("M", "[not-closed")],
        &["[not-closed"]
    )]
    // An invalid library reference in an unused entry must not matter
    #[case::unused_data_entries_are_not_resolved(
        &["--plain"],
        &[("BAD", "[broken]")],
        &["--plain"]
    )]
    #[case::use_client_side_only(&["{K}"], &[("K", "c")], &["c"])]
    fn processor_arguments_are_resolved(
        #[case] args: &[&str],
        #[case] entries: &[(&str, &str)],
        #[case] expected: &[&str],
    ) {
        let (_t, libs) = libs_dir();
        let out = get_processor_arguments(&libs, args, &data(entries)).unwrap();
        assert_eq!(out, expected);
    }

    #[rstest]
    #[case::bracket_form_resolves_existing_library(
        "net/minecraftforge/forge/1.20-47/forge-1.20-47-client.jar",
        "[net.minecraftforge:forge:1.20-47:client]"
    )]
    #[case::bracket_form_supports_extension(
        "de/oceanlabs/mcp/mcp_config/1.20/mcp_config-1.20.zip",
        "[de.oceanlabs.mcp:mcp_config:1.20@zip]"
    )]
    fn processor_arguments_bracket_form_resolves_library(
        #[case] rel_path: &str,
        #[case] arg: &str,
    ) {
        let (_t, libs) = libs_dir();
        let path = make_lib(&libs, rel_path);
        let out = get_processor_arguments(&libs, &[arg], &data(&[])).unwrap();
        assert_eq!(out, vec![path]);
    }

    #[rstest]
    #[case::bracket_form(&["[g:a:1]"], &[])]
    #[case::data_value_in_brackets(&["{M}"], &[("M", "[g:a:1]")])]
    fn processor_arguments_missing_library_gives_uncanonicalized_path(
        #[case] args: &[&str],
        #[case] entries: &[(&str, &str)],
    ) {
        let (_t, libs) = libs_dir();
        let out = get_processor_arguments(&libs, args, &data(entries)).unwrap();
        let mut expected = libs.clone();
        expected.push("g/a/1/a-1.jar");
        assert_eq!(out, vec![expected.to_string_lossy().to_string()]);
    }

    #[rstest]
    #[case::bracket_form(&["[broken]"], &[])]
    #[case::data_value(&["{M}"], &[("M", "[broken]")])]
    fn processor_arguments_invalid_library_coordinate_fails(
        #[case] args: &[&str],
        #[case] entries: &[(&str, &str)],
    ) {
        let (_t, libs) = libs_dir();
        let err = get_processor_arguments(&libs, args, &data(entries)).unwrap_err();
        assert!(matches!(err, MinecraftDomainError::ParseFailed { .. }));
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

    #[rstest]
    #[case::unknown_placeholder(&["{UNKNOWN}"], &[("SIDE", "client")])]
    #[case::braces_coming_from_data_values(&["{A}"], &[("A", "{B}")])]
    #[case::error_in_any_argument_fails_whole_call(&["ok", "{NOPE}", "ok2"], &[])]
    fn processor_arguments_unresolved_placeholder_fails(
        #[case] args: &[&str],
        #[case] entries: &[(&str, &str)],
    ) {
        let (_t, libs) = libs_dir();
        let err = get_processor_arguments(&libs, args, &data(entries)).unwrap_err();
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

    #[rstest]
    #[case::read_from_manifest(
        "Manifest-Version: 1.0\r\nMain-Class: net.example.Main\r\nCreated-By: x\r\n",
        Some("net.example.Main")
    )]
    #[case::value_is_trimmed("Main-Class:    a.B   \n", Some("a.B"))]
    #[case::first_occurrence_wins("Main-Class: first\nMain-Class: second\n", Some("first"))]
    #[case::absent_gives_none("Manifest-Version: 1.0\n", None)]
    #[case::key_is_case_sensitive("main-class: a.B\n", None)]
    #[tokio::test]
    async fn main_class_is_read_from_manifest(
        #[case] manifest: &str,
        #[case] expected: Option<&str>,
    ) {
        let tmp = tempfile::tempdir().unwrap();
        let jar = write_jar(tmp.path(), "a.jar", Some(manifest));
        assert_eq!(
            get_processor_main_class(jar).await.unwrap(),
            expected.map(str::to_string)
        );
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
        let result = get_processor_main_class(path.to_string_lossy().to_string()).await;
        assert!(
            matches!(result, Err(MinecraftDomainError::StorageFailure { .. })),
            "{result:?}"
        );
    }
}
