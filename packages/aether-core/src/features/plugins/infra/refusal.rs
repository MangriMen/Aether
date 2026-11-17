use crate::features::{
    instance::InstanceError,
    plugins::{PluginCallError, PluginError},
};

/// The error of a capability's call (an importer, a content provider, an updater) when the plugin
/// reported that it was refused something the user decides on: a host one of its network host
/// functions may not reach, or the local address of one (the user can allow either), or a
/// command `run_command` did not run (the
/// user denied it, or did not answer). Either is told as such, not folded into the text of a
/// generic failure. `None` for any other error.
pub(super) fn refusal(err: &PluginError) -> Option<InstanceError> {
    let PluginError::FunctionCallFailed {
        plugin_id, error, ..
    } = err
    else {
        return None;
    };
    match error {
        PluginCallError::HostNotAllowed { host, details } => {
            Some(InstanceError::PluginHostNotAllowed {
                plugin_id: plugin_id.to_string(),
                host: host.clone(),
                details: details.clone(),
            })
        }
        PluginCallError::AddressBlocked {
            host,
            port,
            details,
        } => Some(InstanceError::PluginAddressBlocked {
            plugin_id: plugin_id.to_string(),
            host: host.clone(),
            port: *port,
            details: details.clone(),
        }),
        PluginCallError::CommandNotApproved { reason, details } => {
            Some(InstanceError::PluginCommandNotApproved {
                plugin_id: plugin_id.to_string(),
                reason: *reason,
                details: details.clone(),
            })
        }
        _ => None,
    }
}

/// A plugin whose exports fail the way a plugin built on the PDK fails: with a message given to
/// `error_set`, then a non-zero code.
#[cfg(test)]
pub(crate) mod failing_plugin {
    use std::fmt::Write as _;

    use extism::{Manifest, PluginBuilder, Wasm};

    use crate::features::{
        instance::PluginId,
        plugins::{PluginHandle, infra::extism::models::ExtismPluginInstance},
    };

    /// What the plugin reports when `http_get` refused `cdn.example` to it and it passed the
    /// error back in a failure envelope.
    pub(crate) const HOST_REFUSED: &str = r#"aether-plugin-failure:{"message":"cdn refused","host":{"type":"http","data":{"message":"Host `cdn.example` is not listed in the plugin's allowed_hosts","error":{"type":"hostNotAllowed","data":{"host":"cdn.example"}}}}}"#;

    /// What the plugin reports when the user denied the command it asked `run_command` for and
    /// it passed the error back in a failure envelope.
    pub(crate) const COMMAND_DENIED: &str = r#"aether-plugin-failure:{"message":"java refused","host":{"type":"commandNotApproved","data":{"reason":"denied","message":"The command of plugin \"packwiz\" was not approved: denied"}}}"#;

    /// Any other failure: an `http` error of another kind.
    pub(crate) const NOT_FOUND: &str = r#"aether-plugin-failure:{"message":"got 404","host":{"type":"http","data":{"message":"404","error":{"type":"status","data":{"status":404}}}}}"#;

    /// Each of `exports` reports `message` and fails.
    pub(crate) fn failing(id: &PluginId, exports: &[&str], message: &str) -> PluginHandle {
        let escaped = message.bytes().fold(String::new(), |mut acc, byte| {
            let _ = write!(acc, "\\{byte:02x}");
            acc
        });
        let functions = exports.iter().fold(String::new(), |mut acc, name| {
            let _ = writeln!(acc, "(func (export \"{name}\") (result i32) (call $fail))");
            acc
        });
        let wat = format!(
            r#"(module
            (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
            (import "extism:host/env" "store_u8" (func $store_u8 (param i64 i32)))
            (import "extism:host/env" "error_set" (func $error_set (param i64)))
            (memory (export "memory") 1)
            (data (i32.const 0) "{escaped}")
            (func $fail (result i32)
                (local $offset i64)
                (local $i i32)
                (local.set $offset (call $alloc (i64.const {len})))
                (block $copied
                    (loop $copy
                        (br_if $copied (i32.ge_u (local.get $i) (i32.const {len})))
                        (call $store_u8
                            (i64.add (local.get $offset) (i64.extend_i32_u (local.get $i)))
                            (i32.load8_u (local.get $i)))
                        (local.set $i (i32.add (local.get $i) (i32.const 1)))
                        (br $copy)))
                (call $error_set (local.get $offset))
                (i32.const -1))
            {functions})"#,
            len = message.len(),
        );
        let plugin = PluginBuilder::new(Manifest::new([Wasm::data(wat.into_bytes())]))
            .build()
            .expect("the test plugin should load");

        PluginHandle::new(ExtismPluginInstance::new(plugin, id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::{instance::PluginId, plugins::CommandApprovalRefusal};

    fn call_failed(error: PluginCallError) -> PluginError {
        PluginError::FunctionCallFailed {
            function_name: "import".to_owned(),
            plugin_id: PluginId::from_static("packwiz"),
            error,
        }
    }

    #[test]
    fn should_name_the_plugin_and_the_host_when_a_host_was_refused_to_it() {
        let err = call_failed(PluginCallError::HostNotAllowed {
            host: "cdn.example".to_owned(),
            details: "cdn refused".to_owned(),
        });

        assert!(matches!(
            refusal(&err),
            Some(InstanceError::PluginHostNotAllowed { plugin_id, host, details })
                if plugin_id == "packwiz" && host == "cdn.example" && details == "cdn refused"
        ));
    }

    #[test]
    fn should_name_the_plugin_the_host_and_the_port_when_a_local_address_was_refused_to_it() {
        let err = call_failed(PluginCallError::AddressBlocked {
            host: "nas.lan".to_owned(),
            port: 8080,
            details: "nas refused".to_owned(),
        });

        assert!(matches!(
            refusal(&err),
            Some(InstanceError::PluginAddressBlocked { plugin_id, host, port: 8080, details })
                if plugin_id == "packwiz" && host == "nas.lan" && details == "nas refused"
        ));
    }

    #[test]
    fn should_name_the_plugin_and_the_reason_when_its_command_was_not_approved() {
        for reason in [
            CommandApprovalRefusal::Denied,
            CommandApprovalRefusal::TimedOut,
            CommandApprovalRefusal::Cancelled,
            CommandApprovalRefusal::JarChanged,
        ] {
            let err = call_failed(PluginCallError::CommandNotApproved {
                reason,
                details: "java refused".to_owned(),
            });

            assert!(
                matches!(
                    refusal(&err),
                    Some(InstanceError::PluginCommandNotApproved { plugin_id, reason: got, details })
                        if plugin_id == "packwiz" && got == reason && details == "java refused"
                ),
                "{reason:?}"
            );
        }
    }

    #[test]
    fn should_give_nothing_when_the_call_failed_another_way() {
        for err in [
            call_failed(PluginCallError::Plugin("mod not found".to_owned())),
            call_failed(PluginCallError::Timeout),
            PluginError::Busy {
                plugin_id: PluginId::from_static("packwiz"),
            },
        ] {
            assert!(refusal(&err).is_none(), "{err:?}");
        }
    }
}
