//! Which host functions a user should be warned about before giving a plugin to them.
//!
//! The core decides; a frontend only shows what it is told. Every host function the core offers
//! is classified here on purpose, so that adding one without deciding fails a test instead of
//! passing as harmless.

use serde::{Deserialize, Serialize};

/// What a plugin can do to the user's machine through a host function it declares. Ordered as
/// listed here, which is the order a plugin's risks are reported in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HostFunctionRisk {
    /// `run_command`: starts programs on the host, each run with the user's consent.
    RunsPrograms,
    /// `write_instance_file`: puts files into a game instance, such as a mod that runs with the
    /// game, without asking.
    WritesInstanceFiles,
    /// `install_java`: puts a runtime on disk that `run_command` is then allowed to start.
    InstallsJava,
}

/// How the core classifies one host function.
pub(crate) enum HostFunctionClass {
    Risky(HostFunctionRisk),
    /// Reviewed and carries no warning.
    Plain,
}

/// The class of a host function the core offers; `None` for a name it does not know.
pub(crate) fn classify_host_function(name: &str) -> Option<HostFunctionClass> {
    use HostFunctionClass::{Plain, Risky};

    Some(match name {
        "run_command" => Risky(HostFunctionRisk::RunsPrograms),
        "write_instance_file" => Risky(HostFunctionRisk::WritesInstanceFiles),
        "install_java" => Risky(HostFunctionRisk::InstallsJava),
        "log"
        | "get_id"
        | "get_java"
        | "instance_get_dir"
        | "instance_plugin_get_dir"
        | "instance_create"
        | "list_content"
        | "enable_contents"
        | "disable_contents"
        | "read_instance_file"
        | "http_get"
        | "http_post_json"
        | "download_to_cache" => Plain,
        _ => return None,
    })
}

/// The risks of the host functions a plugin declares, once each, in enum order. A name the core
/// does not know adds none: the plugin cannot be enabled with it anyway.
pub fn host_function_risks(declared: &[String]) -> Vec<HostFunctionRisk> {
    let mut risks: Vec<_> = declared
        .iter()
        .filter_map(|name| match classify_host_function(name)? {
            HostFunctionClass::Risky(risk) => Some(risk),
            HostFunctionClass::Plain => None,
        })
        .collect();
    risks.sort_unstable();
    risks.dedup();
    risks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn should_report_each_risk_once_in_enum_order_when_declared_in_any_order() {
        let declared = names(&["install_java", "log", "run_command", "install_java"]);

        assert_eq!(
            host_function_risks(&declared),
            [
                HostFunctionRisk::RunsPrograms,
                HostFunctionRisk::InstallsJava
            ]
        );
    }

    #[test]
    fn should_report_nothing_when_only_plain_or_unknown_functions_are_declared() {
        assert!(
            host_function_risks(&names(&["http_get", "read_instance_file", "nope"])).is_empty()
        );
        assert!(host_function_risks(&[]).is_empty());
    }

    #[test]
    fn should_classify_write_instance_file_as_writing_instance_files() {
        assert_eq!(
            host_function_risks(&names(&["write_instance_file"])),
            [HostFunctionRisk::WritesInstanceFiles]
        );
    }
}
