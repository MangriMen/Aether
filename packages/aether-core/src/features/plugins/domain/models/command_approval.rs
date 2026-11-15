//! What the user is asked, and what is remembered, when a plugin wants to run a command.
//!
//! A command is `<java> <options> -jar <jar> <arguments>`. What the user can approve for good is
//! the tool, not one call: the identity is the jar's content and the JVM options before it, while
//! the arguments (a pack URL, an instance folder) change on every call and are not part of it.
//! Anything the parser below is not sure about can still be approved once, never for good.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::features::instance::PluginId;

/// The largest jar that may be approved for good; hashing a bigger one for every run is not worth
/// it, and a tool that size is not what a plugin ships.
pub const MAX_APPROVED_JAR_BYTES: u64 = 256 * 1024 * 1024;

/// What a "always" answer is remembered by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandIdentity {
    /// Lowercase hex sha256 of the jar.
    pub jar_sha256: String,
    /// The JVM options in the order they were given, each on the allowlist
    /// ([`parse_jar_launch`]), without `-jar`.
    pub jvm_options: Vec<String>,
}

/// A command the user approved for good.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandApproval {
    pub identity: CommandIdentity,
    /// Hash of the plugin's wasm when the user approved: the approval trusts that plugin's choice
    /// of arguments, so another build of it starts over.
    pub plugin_sha256: String,
    /// The jar as the plugin names it (`/cache/...`), for the list the user can take approvals
    /// back from.
    pub jar: String,
    pub approved_at: DateTime<Utc>,
}

impl CommandApproval {
    pub fn covers(&self, identity: &CommandIdentity, plugin_sha256: &str) -> bool {
        self.identity == *identity && self.plugin_sha256 == plugin_sha256
    }
}

/// Why a command can be approved this once only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OnceOnlyReason {
    /// No `-jar`, or no file after it.
    NotJarLaunch,
    /// `-jar` more than once.
    RepeatedJarFlag,
    /// An `@argfile` before the jar: the options it holds are unseen.
    ArgFile,
    /// An option outside the short list of harmless ones.
    UnlistedOption { option: String },
    /// The jar is not a file in the plugin's own cache, which is where the core can tell what it
    /// is.
    TargetNotInCache,
    /// The jar's manifest pulls in more jars, which a hash of this one does not cover.
    JarHasClassPath,
    /// The jar cannot be read as a jar.
    JarUnreadable,
    /// The jar is over [`MAX_APPROVED_JAR_BYTES`].
    JarTooLarge,
}

/// The part of a `-jar` command line that the identity is made of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JarLaunch {
    pub jvm_options: Vec<String>,
    /// The token after the options, as the plugin wrote it.
    pub target: String,
}

/// Reads `args` (everything after the program) as `[options] -jar <target> [arguments]`.
///
/// The Java launcher takes every leading `-` token as an option, `-jar` included, so the target
/// is the first token that does not start with `-`, wherever `-jar` stood. Whatever is not
/// recognised for certain is refused: an unknown option may run code (`-javaagent`,
/// `-XX:OnError=…`) or take the next token as its value and shift the target.
pub fn parse_jar_launch(args: &[String]) -> Result<JarLaunch, OnceOnlyReason> {
    let mut jar_flags = 0_u32;
    let mut jvm_options = Vec::new();

    for arg in args {
        if arg.starts_with('@') {
            return Err(OnceOnlyReason::ArgFile);
        }
        if !arg.starts_with('-') {
            return match jar_flags {
                0 => Err(OnceOnlyReason::NotJarLaunch),
                1 => Ok(JarLaunch {
                    jvm_options,
                    target: arg.clone(),
                }),
                _ => Err(OnceOnlyReason::RepeatedJarFlag),
            };
        }
        if arg == "-jar" {
            jar_flags += 1;
        } else if is_harmless_option(arg) {
            jvm_options.push(arg.clone());
        } else {
            return Err(OnceOnlyReason::UnlistedOption {
                option: arg.clone(),
            });
        }
    }

    Err(OnceOnlyReason::NotJarLaunch)
}

/// The options a command may carry and still be approved for good: memory sizes and two
/// properties that only change how the tool prints. Every other `-D` and `-XX` can load code or
/// run a command (`-Djava.ext.dirs`, `-XX:OnOutOfMemoryError`), so none of them is here.
fn is_harmless_option(option: &str) -> bool {
    for prefix in ["-Xmx", "-Xms", "-Xss"] {
        if let Some(size) = option.strip_prefix(prefix) {
            return is_memory_size(size);
        }
    }

    if let Some(encoding) = option.strip_prefix("-Dfile.encoding=") {
        return !encoding.is_empty()
            && encoding
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    }

    matches!(
        option,
        "-Djava.awt.headless=true" | "-Djava.awt.headless=false"
    )
}

fn is_memory_size(size: &str) -> bool {
    let digits = size
        .strip_suffix(['k', 'K', 'm', 'M', 'g', 'G'])
        .unwrap_or(size);
    !digits.is_empty() && digits.len() <= 10 && digits.chars().all(|c| c.is_ascii_digit())
}

/// Whether the main section of a jar manifest has a `Class-Path` attribute. A line that starts
/// with a space continues the one before it, so only the attribute name at a line start counts.
pub fn manifest_has_class_path(manifest: &str) -> bool {
    for line in manifest.lines() {
        // The main section ends at the first empty line; the rest belongs to single entries.
        if line.trim_end().is_empty() {
            return false;
        }
        if line
            .get(..11)
            .is_some_and(|name| name.eq_ignore_ascii_case("class-path:"))
        {
            return true;
        }
    }
    false
}

/// What the user is shown.
#[derive(Debug, Clone)]
pub struct CommandApprovalRequest {
    pub id: Uuid,
    pub plugin_id: PluginId,
    /// The Java launcher the command starts, as resolved on the host.
    pub java: String,
    pub current_dir: String,
    /// The arguments as the plugin sent them. They can hold a token (a pack URL), so they travel
    /// to the UI only when it asks for the request, never in an event or the log.
    pub arguments: Vec<String>,
    pub jar: Option<CommandJar>,
    /// `Ok` when the user may answer "always".
    pub always: Result<(), OnceOnlyReason>,
}

/// A question that waits for the user.
#[derive(Debug, Clone)]
pub struct PendingCommandApproval {
    pub request: CommandApprovalRequest,
    /// When the question is refused for want of an answer.
    pub expires_at: DateTime<Utc>,
}

/// The jar a command starts, as the core read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandJar {
    /// `/cache/...`, as the plugin names it.
    pub plugin_path: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandApprovalAnswer {
    Once,
    Always,
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandGrant {
    Once,
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandApprovalRefusal {
    /// The user said no, or said it earlier and the plugin has not been reloaded since.
    Denied,
    /// Nobody answered in time.
    TimedOut,
    /// The plugin was stopped or removed, or the launcher is closing.
    Cancelled,
    /// The jar changed while the user was being asked.
    JarChanged,
}

impl CommandApprovalRefusal {
    /// The word a plugin is told, stable and free of paths and hashes.
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Denied => "denied",
            Self::TimedOut => "timed_out",
            Self::Cancelled => "cancelled",
            Self::JarChanged => "jar_changed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn args(raw: &[&str]) -> Vec<String> {
        raw.iter().map(|arg| (*arg).to_owned()).collect()
    }

    #[test]
    fn should_find_the_jar_when_options_come_before_and_after_the_jar_flag() {
        let launch = parse_jar_launch(&args(&[
            "-Xmx1G",
            "-jar",
            "-Dfile.encoding=UTF-8",
            "#/cache/tool.jar",
            "--flag",
        ]))
        .unwrap();

        assert_eq!(launch.target, "#/cache/tool.jar");
        assert_eq!(launch.jvm_options, ["-Xmx1G", "-Dfile.encoding=UTF-8"]);
    }

    #[test]
    fn should_keep_arguments_after_the_jar_out_of_the_options_when_they_look_like_options() {
        let launch =
            parse_jar_launch(&args(&["-jar", "#/cache/a.jar", "-javaagent:x", "-Xmx9G"])).unwrap();

        assert!(launch.jvm_options.is_empty());
    }

    #[rstest]
    #[case::agent("-javaagent:#/cache/a.jar")]
    #[case::native_agent("-agentlib:jdwp=transport=dt_socket")]
    #[case::on_error("-XX:OnError=calc")]
    #[case::on_oom("-XX:OnOutOfMemoryError=calc")]
    #[case::boot_class_path("-Xbootclasspath/a:x.jar")]
    #[case::ext_dirs("-Djava.ext.dirs=x")]
    #[case::class_loader("-Djava.system.class.loader=x.Y")]
    #[case::class_path("-cp")]
    #[case::add_opens("--add-opens")]
    #[case::jar_equals("-jar=x.jar")]
    #[case::version("-version")]
    #[case::memory_with_tail("-Xmx1G;x")]
    #[case::memory_without_size("-Xmx")]
    fn should_offer_once_only_when_an_option_is_not_on_the_allowlist(#[case] option: &str) {
        let result = parse_jar_launch(&args(&[option, "-jar", "#/cache/a.jar"]));

        assert!(matches!(result, Err(OnceOnlyReason::UnlistedOption { .. })));
    }

    #[test]
    fn should_offer_once_only_when_an_argfile_precedes_the_target() {
        let result = parse_jar_launch(&args(&["@opts.txt", "-jar", "#/cache/a.jar"]));

        assert_eq!(result, Err(OnceOnlyReason::ArgFile));
    }

    #[rstest]
    #[case::no_jar_flag(&["#/cache/a.jar"], OnceOnlyReason::NotJarLaunch)]
    #[case::class_name(&["Main.java"], OnceOnlyReason::NotJarLaunch)]
    #[case::repeated(&["-jar", "-jar", "#/cache/a.jar"], OnceOnlyReason::RepeatedJarFlag)]
    #[case::no_target(&["-jar"], OnceOnlyReason::NotJarLaunch)]
    #[case::nothing(&[], OnceOnlyReason::NotJarLaunch)]
    fn should_offer_once_only_when_the_jar_flag_is_missing_repeated_or_has_no_target(
        #[case] raw: &[&str],
        #[case] expected: OnceOnlyReason,
    ) {
        assert_eq!(parse_jar_launch(&args(raw)), Err(expected));
    }

    #[test]
    fn should_offer_once_only_when_the_class_path_is_set_in_the_manifest() {
        let manifest = "Manifest-Version: 1.0\r\nMain-Class: a.B\r\nClass-Path: lib/x.jar\r\n\r\n";

        assert!(manifest_has_class_path(manifest));
    }

    #[test]
    fn should_not_find_a_class_path_when_it_belongs_to_another_section_or_a_continuation() {
        let manifest = "Manifest-Version: 1.0\r\nMain-Class: a.B\r\nX: y\r\n Class-Path: z\r\n\r\nName: a/B.class\r\nClass-Path: q\r\n";

        assert!(!manifest_has_class_path(manifest));
    }

    #[rstest]
    #[case("-Xmx512M")]
    #[case("-Xms64m")]
    #[case("-Xss1024")]
    #[case("-Dfile.encoding=UTF-8")]
    #[case("-Djava.awt.headless=true")]
    fn should_accept_an_option_when_it_is_on_the_allowlist(#[case] option: &str) {
        assert!(is_harmless_option(option));
    }
}
