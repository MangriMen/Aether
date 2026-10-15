use std::sync::Arc;

use extism::{CurrentPlugin, Function, PTR, UserData, Val, ValType};

use crate::core::app::AetherContainer;

use super::{PluginContext, features};

/// The shape `extism::host_fn!` gives every host function.
type HostFn = fn(
    &mut CurrentPlugin,
    &[Val],
    &mut [Val],
    UserData<PluginContext>,
) -> Result<(), extism::Error>;

/// One host function as plugins import it from `extism:host/user`.
pub(crate) struct HostFunctionSpec {
    /// Import name — also what a manifest lists in `api.hostFunctions`.
    pub name: &'static str,
    pub params: &'static [ValType],
    pub results: &'static [ValType],
    handler: HostFn,
}

/// Every host function the core offers. The single source for both what gets linked into a
/// plugin and what a manifest may declare, so the two cannot drift apart.
pub(crate) const HOST_FUNCTIONS: &[HostFunctionSpec] = &[
    // ── core ──
    HostFunctionSpec {
        name: "log",
        params: &[ValType::I64, PTR],
        results: &[],
        handler: features::log,
    },
    HostFunctionSpec {
        name: "get_id",
        params: &[],
        results: &[PTR],
        handler: features::get_id,
    },
    HostFunctionSpec {
        name: "run_command",
        params: &[PTR],
        results: &[PTR],
        handler: features::run_command,
    },
    // ── java ──
    HostFunctionSpec {
        name: "get_java",
        params: &[ValType::I64],
        results: &[PTR],
        handler: features::get_java,
    },
    HostFunctionSpec {
        name: "install_java",
        params: &[ValType::I64],
        results: &[PTR],
        handler: features::install_java,
    },
    // ── instance ──
    HostFunctionSpec {
        name: "instance_get_dir",
        params: &[PTR],
        results: &[PTR],
        handler: features::instance_get_dir,
    },
    HostFunctionSpec {
        name: "instance_plugin_get_dir",
        params: &[PTR],
        results: &[PTR],
        handler: features::instance_plugin_get_dir,
    },
    HostFunctionSpec {
        name: "instance_create",
        params: &[PTR],
        results: &[PTR],
        handler: features::instance_create,
    },
    HostFunctionSpec {
        name: "list_content",
        params: &[PTR],
        results: &[PTR],
        handler: features::list_content,
    },
    HostFunctionSpec {
        name: "enable_contents",
        params: &[PTR, PTR],
        results: &[PTR],
        handler: features::enable_contents,
    },
    HostFunctionSpec {
        name: "disable_contents",
        params: &[PTR, PTR],
        results: &[PTR],
        handler: features::disable_contents,
    },
    // The audited way in and out of an instance directory: an instance id plus a relative
    // path, never a WASI path.
    HostFunctionSpec {
        name: "write_instance_file",
        params: &[PTR],
        results: &[PTR],
        handler: features::write_instance_file,
    },
    HostFunctionSpec {
        name: "read_instance_file",
        params: &[PTR],
        results: &[PTR],
        handler: features::read_instance_file,
    },
    // ── http ──
    HostFunctionSpec {
        name: "http_get",
        params: &[PTR],
        results: &[PTR],
        handler: features::http_get,
    },
    HostFunctionSpec {
        name: "http_post_json",
        params: &[PTR],
        results: &[PTR],
        handler: features::http_post_json,
    },
    // The host downloads, verifies and names the file; the plugin gets a `/cache` path back
    // and never handles the bytes.
    HostFunctionSpec {
        name: "download_to_cache",
        params: &[PTR],
        results: &[PTR],
        handler: features::download_to_cache,
    },
];

pub(crate) fn host_function_names() -> Vec<&'static str> {
    HOST_FUNCTIONS.iter().map(|spec| spec.name).collect()
}

/// The host functions a plugin gets: exactly the ones it declares in `api.hostFunctions`.
///
/// A declared name the core does not know is skipped here — the enable use case has already
/// refused such a manifest, and linking can only fail on what is missing, not on what is extra.
pub(crate) fn select_host_functions(declared: &[String]) -> Vec<&'static HostFunctionSpec> {
    HOST_FUNCTIONS
        .iter()
        .filter(|spec| declared.iter().any(|name| name == spec.name))
        .collect()
}

pub(crate) fn get_host_functions(
    plugin_id: &str,
    container: &Arc<AetherContainer>,
    declared: &[String],
) -> Vec<Function> {
    let context = PluginContext::new(plugin_id.to_string(), container);

    select_host_functions(declared)
        .into_iter()
        .map(|spec| {
            Function::new(
                spec.name,
                spec.params.iter().cloned(),
                spec.results.iter().cloned(),
                UserData::new(context.clone()),
                spec.handler,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn should_not_register_a_host_function_name_twice_when_building_the_list() {
        let names = host_function_names();
        let unique: HashSet<_> = names.iter().collect();

        assert_eq!(names.len(), unique.len(), "duplicate name in {names:?}");
    }

    #[test]
    fn should_select_only_declared_host_functions_when_some_are_declared() {
        let declared = vec!["http_get".to_owned(), "log".to_owned()];

        let mut selected: Vec<_> = select_host_functions(&declared)
            .into_iter()
            .map(|spec| spec.name)
            .collect();
        selected.sort_unstable();

        assert_eq!(selected, ["http_get", "log"]);
    }

    #[test]
    fn should_select_nothing_when_the_declaration_is_empty() {
        assert!(select_host_functions(&[]).is_empty());
    }
}
