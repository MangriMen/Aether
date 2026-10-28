use std::{collections::HashSet, path::Path};

use crate::features::{
    minecraft::{
        MinecraftDomainError, TEMPORARY_REPLACE_CHAR, parse_arguments, parse_rules, vanilla,
    },
    settings::LocationInfo,
};
use crate::shared::{io::infra::canonicalize, java::domain::get_classpath_separator};

/// Canonicalizes `path` as a string, reporting a missing path as `entity_type`.
fn canonical_string(path: &Path, entity_type: &str) -> Result<String, MinecraftDomainError> {
    canonicalize(path)
        .map(|canonical| canonical.to_string_lossy().into_owned())
        .map_err(|_| MinecraftDomainError::PathNotFound {
            path: path.to_path_buf(),
            entity_type: entity_type.to_owned(),
        })
}

// TODO: Wrap arguments in struct
#[allow(clippy::too_many_arguments)]
pub fn get_jvm_arguments(
    arguments: Option<&[vanilla::Argument]>,
    natives_path: &Path,
    libraries_path: &Path,
    class_paths: &str,
    version_name: &str,
    max_memory: u32,
    custom_args: &[String],
    java_arch: &str,
) -> Result<Vec<String>, MinecraftDomainError> {
    let mut result = Vec::new();

    if let Some(arguments) = arguments {
        parse_arguments(
            arguments,
            &mut result,
            |template| {
                parse_jvm_argument(
                    template,
                    natives_path,
                    libraries_path,
                    class_paths,
                    version_name,
                    java_arch,
                )
            },
            java_arch,
        )?;
    } else {
        result.push(format!(
            "-Djava.library.path={}",
            canonical_string(natives_path, "native_library")?
        ));
        result.push("-cp".to_owned());
        result.push(class_paths.to_owned());
    }

    // `-Xmx0M` is not a valid heap size and the JVM refuses to start; without the flag it picks
    // its own default.
    if max_memory > 0 {
        result.push(format!("-Xmx{max_memory}M"));
    }
    result.extend(custom_args.iter().filter(|arg| !arg.is_empty()).cloned());

    Ok(result)
}

/// The JVM argument that points the game at the log config its version names, if it names one.
/// Placed first, so that an argument of the user's for the same property still wins.
pub fn get_logging_argument(
    version_info: &vanilla::VersionInfo,
    location_info: &LocationInfo,
) -> Result<Option<String>, MinecraftDomainError> {
    let Some((template, file)) = version_info.client_log_config() else {
        return Ok(None);
    };
    let path = location_info.log_config_path(&file.id).ok_or_else(|| {
        MinecraftDomainError::StorageFailure {
            reason: format!("Invalid log config name: {}", file.id),
        }
    })?;

    Ok(Some(template.replace(
        "${path}",
        &canonical_string(&path, "log_config")?,
    )))
}

fn parse_jvm_argument(
    argument: &str,
    natives_path: &Path,
    libraries_path: &Path,
    class_paths: &str,
    version_name: &str,
    java_arch: &str,
) -> Result<String, MinecraftDomainError> {
    let natives = canonical_string(natives_path, "native_library")?;
    let libraries = canonical_string(libraries_path, "library")?;

    let placeholders = [
        ("${natives_directory}", natives.as_str()),
        ("${library_directory}", libraries.as_str()),
        ("${classpath_separator}", get_classpath_separator(java_arch)),
        ("${launcher_name}", "aether"),
        ("${launcher_version}", env!("CARGO_PKG_VERSION")),
        ("${version_name}", version_name),
        ("${classpath}", class_paths),
    ];

    // Normal templates arrive with spaces swapped for the temporary char; restore them
    // so the template stays one argument with its spaces intact.
    let template = argument.replace(TEMPORARY_REPLACE_CHAR, " ");
    Ok(placeholders
        .iter()
        .fold(template, |acc, (name, value)| acc.replace(name, value)))
}

pub fn get_class_paths(
    libraries_path: &Path,
    libraries: &[vanilla::Library],
    client_path: &Path,
    java_arch: &str,
    minecraft_updated: bool,
) -> Result<String, MinecraftDomainError> {
    let mut entries: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for library in libraries {
        let denied = library
            .rules
            .as_deref()
            .is_some_and(|rules| !parse_rules(rules, java_arch, minecraft_updated));
        if denied || !library.include_in_classpath {
            continue;
        }

        let path = get_lib_path(libraries_path, &library.name, false)?;
        if seen.insert(path.clone()) {
            entries.push(path);
        }
    }

    let client = canonical_string(client_path, "classpath")?;
    if seen.insert(client.clone()) {
        entries.push(client);
    }

    Ok(entries.join(get_classpath_separator(java_arch)))
}

pub fn get_class_paths_jar<T: AsRef<str>>(
    libraries_path: &Path,
    libraries: &[T],
    java_arch: &str,
) -> Result<String, MinecraftDomainError> {
    let paths = libraries
        .iter()
        .map(|lib| get_lib_path(libraries_path, lib.as_ref(), false))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(paths.join(get_classpath_separator(java_arch)))
}

pub fn get_lib_path(
    libraries_path: &Path,
    lib: &str,
    allow_not_exist: bool,
) -> Result<String, MinecraftDomainError> {
    let full_path = libraries_path.join(daedalus::get_path_from_artifact(lib)?);

    if allow_not_exist && !full_path.exists() {
        return Ok(full_path.to_string_lossy().into_owned());
    }

    canonical_string(&full_path, "library")
}

impl From<daedalus::Error> for MinecraftDomainError {
    fn from(value: daedalus::Error) -> Self {
        match value {
            daedalus::Error::ParseError(err) => MinecraftDomainError::ParseFailed { reason: err },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::minecraft::vanilla::{
        Argument, ArgumentValue, Library, Os, OsRule, Rule, RuleAction,
    };
    use rstest::rstest;
    use std::collections::HashMap;
    use std::path::PathBuf;

    const SEP: &str = if cfg!(windows) { ";" } else { ":" };

    struct Env {
        _tmp: tempfile::TempDir,
        root: PathBuf,
        libs: PathBuf,
        natives: PathBuf,
        client: PathBuf,
    }

    fn env() -> Env {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let libs = root.join("libraries");
        let natives = root.join("natives");
        std::fs::create_dir_all(&libs).unwrap();
        std::fs::create_dir_all(&natives).unwrap();
        let client = root.join("client.jar");
        std::fs::write(&client, b"jar").unwrap();
        Env {
            _tmp: tmp,
            root,
            libs,
            natives,
            client,
        }
    }

    fn canon(p: &Path) -> String {
        canonicalize(p).unwrap().to_string_lossy().to_string()
    }

    /// Creates the file of a library with the given maven coordinates (plain jar) and
    /// returns its canonical path.
    fn make_lib(e: &Env, rel: &str) -> String {
        let p = e.libs.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, b"jar").unwrap();
        canon(&p)
    }

    fn lib(name: &str) -> Library {
        Library {
            downloads: None,
            extract: None,
            name: name.to_string(),
            url: None,
            natives: None,
            rules: None,
            checksums: None,
            include_in_classpath: true,
            downloadable: true,
        }
    }

    fn allow_os(name: Os) -> Vec<Rule> {
        vec![Rule {
            action: RuleAction::Allow,
            os: Some(OsRule {
                name: Some(name),
                version: None,
                arch: None,
            }),
            features: None,
        }]
    }

    fn split_set(cp: &str) -> HashSet<String> {
        cp.split(SEP).map(str::to_string).collect()
    }

    fn jvm(
        e: &Env,
        args: Option<&[vanilla::Argument]>,
        cp: &str,
        mem: u32,
        custom: &[String],
    ) -> Result<Vec<String>, MinecraftDomainError> {
        get_jvm_arguments(args, &e.natives, &e.libs, cp, "1.20.1", mem, custom, "x64")
    }

    // ---- get_lib_path ----

    #[test]
    fn lib_path_existing_is_canonical_maven_layout() {
        let e = env();
        let expected = make_lib(&e, "com/example/foo/1.0/foo-1.0.jar");
        assert_eq!(
            get_lib_path(&e.libs, "com.example:foo:1.0", false).unwrap(),
            expected
        );
        assert_eq!(
            get_lib_path(&e.libs, "com.example:foo:1.0", true).unwrap(),
            expected
        );
    }

    #[test]
    fn lib_path_supports_extension_and_classifier() {
        let e = env();
        let zip = make_lib(&e, "g/a/1/a-1.zip");
        let natives = make_lib(&e, "g/a/1/a-1-natives-linux.jar");
        let classified_zip = make_lib(&e, "g/a/1/a-1-data.zip");
        assert_eq!(get_lib_path(&e.libs, "g:a:1@zip", false).unwrap(), zip);
        assert_eq!(
            get_lib_path(&e.libs, "g:a:1:natives-linux", false).unwrap(),
            natives
        );
        assert_eq!(
            get_lib_path(&e.libs, "g:a:1:data@zip", false).unwrap(),
            classified_zip
        );
    }

    #[test]
    fn lib_path_missing_fails_with_library_not_found() {
        let e = env();
        let err = get_lib_path(&e.libs, "g:a:1", false).unwrap_err();
        assert!(matches!(
            err,
            MinecraftDomainError::PathNotFound { ref entity_type, .. } if entity_type == "library"
        ));
    }

    #[test]
    fn lib_path_missing_with_allow_not_exist_returns_uncanonicalized_path() {
        let e = env();
        let mut expected = e.libs.clone();
        expected.push("g/a/1/a-1.jar");
        assert_eq!(
            get_lib_path(&e.libs, "g:a:1", true).unwrap(),
            expected.to_string_lossy()
        );
    }

    #[test]
    fn lib_path_invalid_coordinate_is_parse_error() {
        let e = env();
        for allow in [true, false] {
            let err = get_lib_path(&e.libs, "justname", allow).unwrap_err();
            assert!(matches!(err, MinecraftDomainError::ParseFailed { .. }));
        }
    }

    // ---- get_class_paths_jar ----

    #[test]
    fn class_paths_jar_keeps_order_and_joins_with_separator() {
        let e = env();
        let b = make_lib(&e, "g/b/2/b-2.jar");
        let a = make_lib(&e, "g/a/1/a-1.jar");
        let cp = get_class_paths_jar(&e.libs, &["g:b:2", "g:a:1"], "x64").unwrap();
        assert_eq!(cp, format!("{b}{SEP}{a}"));
    }

    #[test]
    fn class_paths_jar_does_not_deduplicate() {
        let e = env();
        let a = make_lib(&e, "g/a/1/a-1.jar");
        let cp = get_class_paths_jar(&e.libs, &["g:a:1", "g:a:1"], "x64").unwrap();
        assert_eq!(cp, format!("{a}{SEP}{a}"));
    }

    #[test]
    fn class_paths_jar_empty_is_empty_string() {
        let e = env();
        let none: [&str; 0] = [];
        assert_eq!(get_class_paths_jar(&e.libs, &none, "x64").unwrap(), "");
    }

    #[test]
    fn class_paths_jar_missing_library_fails() {
        let e = env();
        make_lib(&e, "g/a/1/a-1.jar");
        let err = get_class_paths_jar(&e.libs, &["g:a:1", "g:missing:1"], "x64").unwrap_err();
        assert!(matches!(err, MinecraftDomainError::PathNotFound { .. }));
    }

    // ---- get_class_paths ----

    #[test]
    fn class_paths_with_no_libraries_is_just_client() {
        let e = env();
        let cp = get_class_paths(&e.libs, &[], &e.client, "x64", true).unwrap();
        assert_eq!(cp, canon(&e.client));
    }

    #[test]
    fn class_paths_contains_libraries_and_client_joined_by_separator() {
        let e = env();
        let a = make_lib(&e, "g/a/1/a-1.jar");
        let b = make_lib(&e, "g/b/2/b-2.jar");
        let cp = get_class_paths(
            &e.libs,
            &[lib("g:a:1"), lib("g:b:2")],
            &e.client,
            "x64",
            true,
        )
        .unwrap();
        assert_eq!(cp.matches(SEP).count(), 2);
        assert_eq!(split_set(&cp), HashSet::from([a, b, canon(&e.client)]));
    }

    #[test]
    fn class_paths_deduplicates_libraries() {
        let e = env();
        let a = make_lib(&e, "g/a/1/a-1.jar");
        let cp = get_class_paths(
            &e.libs,
            &[lib("g:a:1"), lib("g:a:1")],
            &e.client,
            "x64",
            true,
        )
        .unwrap();
        assert_eq!(split_set(&cp), HashSet::from([a, canon(&e.client)]));
        assert_eq!(cp.matches(SEP).count(), 1);
    }

    #[test]
    fn class_paths_deduplicates_client_that_is_also_a_library() {
        let e = env();
        let a = make_lib(&e, "g/a/1/a-1.jar");
        let client = e.libs.join("g/a/1/a-1.jar");
        let cp = get_class_paths(&e.libs, &[lib("g:a:1")], &client, "x64", true).unwrap();
        assert_eq!(cp, a);
    }

    #[test]
    fn class_paths_skips_libraries_not_included_in_classpath() {
        let e = env();
        make_lib(&e, "g/a/1/a-1.jar");
        let b = make_lib(&e, "g/b/2/b-2.jar");
        let mut hidden = lib("g:a:1");
        hidden.include_in_classpath = false;
        let cp = get_class_paths(&e.libs, &[hidden, lib("g:b:2")], &e.client, "x64", true).unwrap();
        assert_eq!(split_set(&cp), HashSet::from([b, canon(&e.client)]));
    }

    #[test]
    fn class_paths_not_included_library_need_not_exist() {
        let e = env();
        let mut hidden = lib("g:missing:1");
        hidden.include_in_classpath = false;
        let cp = get_class_paths(&e.libs, &[hidden], &e.client, "x64", true).unwrap();
        assert_eq!(cp, canon(&e.client));
    }

    #[test]
    fn class_paths_applies_library_rules() {
        let e = env();
        let a = make_lib(&e, "g/a/1/a-1.jar");
        make_lib(&e, "g/b/2/b-2.jar");
        let mut allowed = lib("g:a:1");
        allowed.rules = Some(allow_os(Os::native()));
        let mut denied = lib("g:b:2");
        denied.rules = Some(allow_os(Os::Unknown));
        let cp = get_class_paths(&e.libs, &[allowed, denied], &e.client, "x64", true).unwrap();
        assert_eq!(split_set(&cp), HashSet::from([a, canon(&e.client)]));
    }

    #[test]
    fn class_paths_skipped_by_rules_library_need_not_exist() {
        let e = env();
        let mut denied = lib("g:missing:2");
        denied.rules = Some(allow_os(Os::Unknown));
        let cp = get_class_paths(&e.libs, &[denied], &e.client, "x64", true).unwrap();
        assert_eq!(cp, canon(&e.client));
    }

    #[test]
    fn class_paths_rules_depend_on_minecraft_updated() {
        let e = env();
        make_lib(&e, "g/a/1/a-1.jar");
        let mut l = lib("g:a:1");
        // Native OS name with an aarch64 Java: matches only for updated Minecraft
        l.rules = Some(allow_os(Os::native()));
        let updated = get_class_paths(
            &e.libs,
            std::slice::from_ref(&l),
            &e.client,
            "aarch64",
            true,
        )
        .unwrap();
        let old = get_class_paths(&e.libs, &[l], &e.client, "aarch64", false).unwrap();
        assert_eq!(updated.matches(SEP).count(), 1);
        assert_eq!(old, canon(&e.client));
    }

    #[test]
    fn class_paths_ignores_natives_field_and_uses_library_name() {
        let e = env();
        let a = make_lib(&e, "g/a/1/a-1.jar");
        let mut l = lib("g:a:1");
        l.natives = Some(HashMap::from([(Os::native(), "natives-x".to_string())]));
        let cp = get_class_paths(&e.libs, &[l], &e.client, "x64", true).unwrap();
        assert_eq!(split_set(&cp), HashSet::from([a, canon(&e.client)]));
    }

    #[test]
    fn class_paths_library_with_classifier_in_name_is_resolved() {
        let e = env();
        let n = make_lib(&e, "g/a/1/a-1-natives-x.jar");
        let cp =
            get_class_paths(&e.libs, &[lib("g:a:1:natives-x")], &e.client, "x64", true).unwrap();
        assert_eq!(split_set(&cp), HashSet::from([n, canon(&e.client)]));
    }

    #[test]
    fn class_paths_missing_library_fails() {
        let e = env();
        let err = get_class_paths(&e.libs, &[lib("g:a:1")], &e.client, "x64", true).unwrap_err();
        assert!(matches!(
            err,
            MinecraftDomainError::PathNotFound { ref entity_type, .. } if entity_type == "library"
        ));
    }

    #[test]
    fn class_paths_missing_client_fails_with_classpath_entity() {
        let e = env();
        let missing = e.root.join("nope.jar");
        let err = get_class_paths(&e.libs, &[], &missing, "x64", true).unwrap_err();
        assert!(matches!(
            err,
            MinecraftDomainError::PathNotFound { ref entity_type, ref path }
                if entity_type == "classpath" && path == &missing
        ));
    }

    #[test]
    fn class_paths_invalid_library_name_fails_with_parse_error() {
        let e = env();
        let err = get_class_paths(&e.libs, &[lib("broken")], &e.client, "x64", true).unwrap_err();
        assert!(matches!(err, MinecraftDomainError::ParseFailed { .. }));
    }

    // ---- get_jvm_arguments ----

    #[test]
    fn jvm_without_arguments_uses_legacy_defaults() {
        let e = env();
        let out = jvm(&e, None, "a.jar:b.jar", 2048, &[]).unwrap();
        assert_eq!(
            out,
            vec![
                format!("-Djava.library.path={}", canon(&e.natives)),
                "-cp".to_string(),
                "a.jar:b.jar".to_string(),
                "-Xmx2048M".to_string(),
            ]
        );
    }

    #[rstest]
    #[case::empty_arguments_have_only_memory_and_custom(
        512,
        &["-Dfoo=bar", "", "-Dbaz"],
        &["-Xmx512M", "-Dfoo=bar", "-Dbaz"]
    )]
    #[case::heap_flag_is_omitted_when_the_memory_is_zero(0, &["-Dfoo=bar"], &["-Dfoo=bar"])]
    fn jvm_with_empty_arguments_emits_memory_and_custom(
        #[case] mem: u32,
        #[case] custom: &[&str],
        #[case] expected: &[&str],
    ) {
        let e = env();
        let custom: Vec<String> = custom.iter().map(ToString::to_string).collect();
        let out = jvm(&e, Some(&[]), "cp", mem, &custom).unwrap();
        assert_eq!(out, expected);
    }

    #[test]
    fn jvm_custom_arguments_are_not_split_or_trimmed() {
        let e = env();
        let custom = vec!["-Dx=a b".to_string()];
        let out = jvm(&e, None, "cp", 1, &custom).unwrap();
        assert_eq!(out.last().unwrap(), "-Dx=a b");
        assert_eq!(out[3], "-Xmx1M");
    }

    #[test]
    fn jvm_placeholders_are_replaced() {
        let e = env();
        let args = vec![
            Argument::Normal("-Djava.library.path=${natives_directory}".to_string()),
            Argument::Normal("-Dminecraft.launcher.brand=${launcher_name}".to_string()),
            Argument::Normal("-Dminecraft.launcher.version=${launcher_version}".to_string()),
            Argument::Normal("-Dlib=${library_directory}".to_string()),
            Argument::Normal("-Dver=${version_name}".to_string()),
            Argument::Normal("-Dsep=${classpath_separator}".to_string()),
            Argument::Normal("${classpath}".to_string()),
        ];
        let out = jvm(&e, Some(&args), "A.jar", 4096, &["-Dc=1".to_string()]).unwrap();
        assert_eq!(
            out,
            vec![
                format!("-Djava.library.path={}", canon(&e.natives)),
                "-Dminecraft.launcher.brand=aether".to_string(),
                format!("-Dminecraft.launcher.version={}", env!("CARGO_PKG_VERSION")),
                format!("-Dlib={}", canon(&e.libs)),
                "-Dver=1.20.1".to_string(),
                format!("-Dsep={SEP}"),
                "A.jar".to_string(),
                "-Xmx4096M".to_string(),
                "-Dc=1".to_string(),
            ]
        );
    }

    #[rstest]
    #[case::placeholder_value_with_spaces_stays_one_argument(
        vec![Argument::Normal("${classpath}".to_string())],
        "C:/My Libs/a.jar",
        &["C:/My Libs/a.jar", "-Xmx1M"]
    )]
    // JVM templates are never split on spaces and keep their whitespace verbatim.
    #[case::whitespace_in_template_is_kept_not_split(
        vec![Argument::Normal("-cp ${classpath}".to_string())],
        "A.jar",
        &["-cp A.jar", "-Xmx1M"]
    )]
    #[case::ruled_value_with_space_stays_one_argument_with_placeholders(
        vec![Argument::Ruled {
            rules: vec![],
            value: ArgumentValue::Many(vec![
                "-Dos.name=Windows 10".to_string(),
                "-Dv=${version_name} x".to_string(),
            ]),
        }],
        "A.jar",
        &["-Dos.name=Windows 10", "-Dv=1.20.1 x", "-Xmx1M"]
    )]
    #[case::unknown_placeholders_are_left_untouched(
        vec![Argument::Normal("-Dx=${unknown}".to_string())],
        "A.jar",
        &["-Dx=${unknown}", "-Xmx1M"]
    )]
    fn jvm_template_values_are_kept_verbatim(
        #[case] args: Vec<Argument>,
        #[case] classpath: &str,
        #[case] expected: &[&str],
    ) {
        let e = env();
        let out = jvm(&e, Some(&args), classpath, 1, &[]).unwrap();
        assert_eq!(out, expected);
    }

    #[test]
    fn jvm_ruled_arguments_follow_rules() {
        let e = env();
        let args = vec![
            Argument::Ruled {
                rules: allow_os(Os::native()),
                value: ArgumentValue::Many(vec![
                    "-Da".to_string(),
                    "-Db=${version_name}".to_string(),
                ]),
            },
            Argument::Ruled {
                rules: allow_os(Os::Unknown),
                value: ArgumentValue::Single("-Dskipped".to_string()),
            },
            Argument::Ruled {
                rules: allow_os(Os::native()),
                value: ArgumentValue::Single("-Dsingle".to_string()),
            },
        ];
        let out = jvm(&e, Some(&args), "cp", 1, &[]).unwrap();
        assert_eq!(out, vec!["-Da", "-Db=1.20.1", "-Dsingle", "-Xmx1M"]);
    }

    #[rstest]
    #[case::without_arguments(None)]
    #[case::with_arguments_even_if_unused(Some(vec![Argument::Normal("-Dx=1".to_string())]))]
    fn jvm_fails_when_natives_dir_missing(#[case] args: Option<Vec<Argument>>) {
        let e = env();
        let err = get_jvm_arguments(
            args.as_deref(),
            &e.root.join("missing"),
            &e.libs,
            "cp",
            "v",
            1,
            &[],
            "x64",
        )
        .unwrap_err();
        assert!(matches!(
            err,
            MinecraftDomainError::PathNotFound { ref entity_type, .. }
                if entity_type == "native_library"
        ));
    }

    #[test]
    fn jvm_with_arguments_fails_when_libraries_dir_missing() {
        let e = env();
        let args = vec![Argument::Normal("-Dx=1".to_string())];
        let err = get_jvm_arguments(
            Some(&args),
            &e.natives,
            &e.root.join("missing"),
            "cp",
            "v",
            1,
            &[],
            "x64",
        )
        .unwrap_err();
        assert!(matches!(
            err,
            MinecraftDomainError::PathNotFound { ref entity_type, .. } if entity_type == "library"
        ));
    }

    #[test]
    fn jvm_without_arguments_does_not_need_libraries_dir() {
        let e = env();
        let out = get_jvm_arguments(
            None,
            &e.natives,
            &e.root.join("missing"),
            "cp",
            "v",
            1,
            &[],
            "x64",
        )
        .unwrap();
        assert_eq!(out.len(), 4);
    }

    fn version_with_log_config(logging: Option<serde_json::Value>) -> vanilla::VersionInfo {
        let mut json = serde_json::json!({
            "assetIndex": { "id": "1", "sha1": "0", "size": 1, "totalSize": 1, "url": "u" },
            "assets": "1",
            "downloads": {},
            "id": "1.12",
            "javaVersion": null,
            "libraries": [],
            "mainClass": "Main",
            "minimumLauncherVersion": 1,
            "releaseTime": "2020-01-01T00:00:00Z",
            "time": "2020-01-01T00:00:00Z",
            "type": "release"
        });
        if let Some(logging) = logging {
            json["logging"] = logging;
        }
        serde_json::from_value(json).unwrap()
    }

    fn log_config(id: &str) -> serde_json::Value {
        serde_json::json!({ "client": {
            "argument": "-Dlog4j.configurationFile=${path}",
            "file": { "id": id, "sha1": "0", "size": 1, "url": "u" },
            "type": "log4j2-xml"
        } })
    }

    #[test]
    fn logging_argument_points_at_the_downloaded_config() {
        let e = env();
        let location_info = LocationInfo::new(e.root.join("settings"), e.root.clone());
        let path = location_info.log_configs_dir().join("client-1.12.xml");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"<xml/>").unwrap();
        let version = version_with_log_config(Some(log_config("client-1.12.xml")));

        let argument = get_logging_argument(&version, &location_info).unwrap();

        assert_eq!(
            argument,
            Some(format!("-Dlog4j.configurationFile={}", canon(&path)))
        );
    }

    #[test]
    fn logging_argument_is_none_when_the_version_names_no_config() {
        let e = env();
        let location_info = LocationInfo::new(e.root.join("settings"), e.root.clone());

        let argument =
            get_logging_argument(&version_with_log_config(None), &location_info).unwrap();

        assert_eq!(argument, None);
    }

    #[test]
    fn logging_argument_fails_when_the_config_is_missing() {
        let e = env();
        let location_info = LocationInfo::new(e.root.join("settings"), e.root.clone());
        let version = version_with_log_config(Some(log_config("client-1.12.xml")));

        let err = get_logging_argument(&version, &location_info).unwrap_err();

        assert!(matches!(
            err,
            MinecraftDomainError::PathNotFound { ref entity_type, .. } if entity_type == "log_config"
        ));
    }

    #[test]
    fn logging_argument_fails_when_the_config_id_leaves_its_directory() {
        let e = env();
        let location_info = LocationInfo::new(e.root.join("settings"), e.root.clone());
        let version = version_with_log_config(Some(log_config("../x.xml")));

        let err = get_logging_argument(&version, &location_info).unwrap_err();

        assert!(matches!(err, MinecraftDomainError::StorageFailure { .. }));
    }
}
