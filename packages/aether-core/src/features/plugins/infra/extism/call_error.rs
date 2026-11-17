use aether_core_plugin_api::v0::{HostError, HttpErrorDto, PluginFailureDto};

use crate::features::plugins::{CommandApprovalRefusal, PluginCallError};

// Extism hands back an untyped `anyhow::Error` and builds these messages itself
// (`extism-1.41.0/src/plugin.rs`, `raw_call`; `current_plugin.rs`, `memory_growing`). Text is
// all there is to tell the limits apart, so it is matched here and nowhere else, and the tests
// below run real calls: an Extism update that rewords one fails a test instead of turning a
// limit into "the plugin returned an error" in production.
const TIMEOUT: &str = "timeout";
const OUT_OF_FUEL: &str = "plugin ran out of fuel";
const OUT_OF_MEMORY: &str = "oom";
/// `wasi_common::I32Exit`'s `Display`; the code itself comes back as the call's return code.
const WASI_EXIT_PREFIX: &str = "Exited with i32 exit status ";
/// The return code Extism substitutes when wasmtime trapped (`EXIT_SIGNALED_SIGABRT`).
const TRAP_CODE: i32 = 134;

/// Classifies what `Plugin::call_get_error_code` returned. The limits are checked before the
/// trap code: Extism reports a timeout with the trap code too, since it is a wasmtime interrupt.
pub(super) fn classify_call_error(error: &extism::Error, code: i32) -> PluginCallError {
    // The root cause, not the outermost message: when a plugin reports an error and traps
    // anyway, Extism wraps the plugin's message in the trap.
    let cause = error.root_cause().to_string();

    match cause.as_str() {
        TIMEOUT => PluginCallError::Timeout,
        OUT_OF_FUEL => PluginCallError::OutOfFuel,
        OUT_OF_MEMORY => PluginCallError::OutOfMemory,
        _ if cause.starts_with(WASI_EXIT_PREFIX) => PluginCallError::NonZeroExit(code),
        // What the plugin reported in a failure envelope. Extism keeps the plugin's message as the
        // cause even when it traps after setting it, so this is not hidden behind the trap code.
        _ if let Some(failure) = PluginFailureDto::parse(&cause) => reported_failure(failure),
        _ if code == TRAP_CODE => PluginCallError::Trap(cause),
        _ => PluginCallError::Plugin(cause),
    }
}

/// A failure the plugin reported in its envelope, with the plugin's own message: the refusal of a
/// command, either of its own call of `run_command` or of a call of another plugin that failed
/// that way; or the refusal of a host by one of its own network host functions (a host refused
/// to another plugin it called reaches it as a failed call, not as this); else only that
/// message. A refusal of a kind this launcher does not know is only the plugin's message.
fn reported_failure(failure: PluginFailureDto) -> PluginCallError {
    let reported = match failure.host {
        Some(HostError::CommandNotApproved { reason, .. }) => {
            CommandApprovalRefusal::try_from(reason)
        }
        Some(HostError::Http {
            error: HttpErrorDto::HostNotAllowed { host },
            ..
        }) => {
            return PluginCallError::HostNotAllowed {
                host,
                details: failure.message,
            };
        }
        _ => return PluginCallError::Plugin(failure.message),
    };
    match reported {
        Ok(reason) => PluginCallError::CommandNotApproved {
            reason,
            details: failure.message,
        },
        Err(_) => PluginCallError::Plugin(failure.message),
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;
    use std::time::Duration;

    use extism::{Manifest, Plugin, PluginBuilder, Wasm};

    use crate::features::instance::PluginId;
    use crate::features::plugins::{
        CommandApprovalRefusal, PluginCallError, PluginError, PluginInstance,
        infra::extism::models::ExtismPluginInstance,
    };

    /// One export per way a call can fail. `$report` sets an error the way the PDK does:
    /// copies the message into Extism memory and hands it to `error_set`. `{data}` holds the
    /// messages (see `MESSAGES`), `{exports}` the exports that report them.
    const PLUGIN: &str = r#"(module
        (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
        (import "extism:host/env" "store_u8" (func $store_u8 (param i64 i32)))
        (import "extism:host/env" "error_set" (func $error_set (param i64)))
        (import "wasi_snapshot_preview1" "proc_exit" (func $proc_exit (param i32)))
        (memory (export "memory") 1)
        (data (i32.const 0) "mod not found")
        {data}

        (func (export "spin") (result i32)
            (loop $forever (br $forever))
            (i32.const 0))

        (func (export "hoard") (result i32)
            (loop $more
                (br_if $more (i32.ne (memory.grow (i32.const 16)) (i32.const -1))))
            (i32.const 0))

        (func (export "panic") (result i32)
            unreachable)

        (func (export "exit") (result i32)
            (call $proc_exit (i32.const 3))
            (i32.const 0))

        (func $report (param $at i32) (param $len i32)
            (local $offset i64)
            (local $i i32)
            (local.set $offset (call $alloc (i64.extend_i32_u (local.get $len))))
            (block $copied
                (loop $copy
                    (br_if $copied (i32.ge_u (local.get $i) (local.get $len)))
                    (call $store_u8
                        (i64.add (local.get $offset) (i64.extend_i32_u (local.get $i)))
                        (i32.load8_u (i32.add (local.get $at) (local.get $i))))
                    (local.set $i (i32.add (local.get $i) (i32.const 1)))
                    (br $copy)))
            (call $error_set (local.get $offset)))

        (func (export "fail") (result i32)
            (call $report (i32.const 0) (i32.const 13))
            (i32.const -1))

        (func (export "fail_then_panic") (result i32)
            (call $report (i32.const 0) (i32.const 13))
            unreachable)

        (func (export "quiet_failure") (result i32)
            (i32.const 7))

        {exports})"#;

    /// What a plugin may put in `error_set`, by export name. Hand-written strings, as a plugin
    /// in any language would produce them: the contract is the wire text, not the Rust types.
    const MESSAGES: &[(&str, &str)] = &[
        (
            "refused",
            r#"aether-plugin-failure:{"message":"AccessViolation","host":{"type":"commandNotApproved","data":{"reason":"timedOut","message":"The command of plugin \"packwiz\" was not approved: timedOut"}}}"#,
        ),
        (
            "refused_with_prefix_and_tail",
            r#"Error: aether-plugin-failure:{"message":"m","host":{"type":"commandNotApproved","data":{"reason":"denied","message":"no"}}}

Caused by: x"#,
        ),
        // What the PDK writes when a plugin passes on the refusal another plugin reported to it.
        (
            "refused_in_a_chain",
            r#"aether-plugin-failure:{"message":"b said so","host":{"type":"commandNotApproved","data":{"reason":"cancelled","message":"b said so"}}}"#,
        ),
        (
            "unknown_reason",
            r#"aether-plugin-failure:{"message":"m","host":{"type":"commandNotApproved","data":{"reason":"because","message":"m"}}}"#,
        ),
        (
            "host_refused",
            r#"aether-plugin-failure:{"message":"cdn refused","host":{"type":"http","data":{"message":"Host `cdn.example` is not allowed","error":{"type":"hostNotAllowed","data":{"host":"cdn.example"}}}}}"#,
        ),
        (
            "http_status",
            r#"aether-plugin-failure:{"message":"got 404","host":{"type":"http","data":{"message":"404","error":{"type":"status","data":{"status":404}}}}}"#,
        ),
        (
            "unknown_http_error",
            r#"aether-plugin-failure:{"message":"newer","host":{"type":"http","data":{"message":"m","error":{"type":"quantumTunnel","data":{"host":"cdn.example"}}}}}"#,
        ),
        ("broken_json", "aether-plugin-failure:{not json"),
        (
            "only_message",
            r#"aether-plugin-failure:{"message":"just text"}"#,
        ),
    ];

    /// Each message is a data segment at its own offset (`MESSAGES` index * 512, after the
    /// plain one at 0), reported by the export of the same name; `{name}_then_panic` traps after.
    fn plugin_source() -> String {
        let mut data = String::new();
        let mut exports = String::new();

        for (index, (name, text)) in MESSAGES.iter().enumerate() {
            let at = (index + 1) * 512;
            let escaped = text.bytes().fold(String::new(), |mut acc, byte| {
                let _ = write!(acc, "\\{byte:02x}");
                acc
            });
            let len = text.len();
            let _ = writeln!(data, "(data (i32.const {at}) \"{escaped}\")");
            let _ = writeln!(
                exports,
                "(func (export \"{name}\") (result i32) \
                    (call $report (i32.const {at}) (i32.const {len})) (i32.const -1))\n\
                 (func (export \"{name}_then_panic\") (result i32) \
                    (call $report (i32.const {at}) (i32.const {len})) unreachable)"
            );
        }

        PLUGIN
            .replace("{data}", &data)
            .replace("{exports}", &exports)
    }

    fn plugin(
        configure: impl FnOnce(Manifest) -> Manifest,
        fuel: Option<u64>,
    ) -> ExtismPluginInstance {
        let manifest = configure(Manifest::new([Wasm::data(plugin_source().into_bytes())]));
        let mut builder = PluginBuilder::new(manifest).with_wasi(true);
        if let Some(fuel) = fuel {
            builder = builder.with_fuel_limit(fuel);
        }
        let plugin: Plugin = builder.build().expect("the test plugin should load");

        ExtismPluginInstance::new(plugin, PluginId::from_static("test"))
    }

    fn unlimited() -> ExtismPluginInstance {
        plugin(|manifest| manifest, None)
    }

    fn call_error(instance: &mut ExtismPluginInstance, name: &str) -> PluginCallError {
        match instance.call_bytes(name, &[]) {
            Err(PluginError::FunctionCallFailed { error, .. }) => error,
            Err(other) => panic!("expected FunctionCallFailed, got: {other:?}"),
            Ok(output) => panic!("the call to {name} should fail, got output {output:?}"),
        }
    }

    #[test]
    fn should_report_a_timeout_when_the_plugin_loops_forever() {
        let mut instance = plugin(|m| m.with_timeout(Duration::from_millis(100)), None);

        assert_eq!(call_error(&mut instance, "spin"), PluginCallError::Timeout);
    }

    #[test]
    fn should_report_an_exhausted_budget_when_the_plugin_runs_out_of_instructions() {
        let mut instance = plugin(|m| m, Some(100_000));

        assert_eq!(
            call_error(&mut instance, "spin"),
            PluginCallError::OutOfFuel
        );
    }

    #[test]
    fn should_report_a_limit_error_when_memory_grows_past_the_limit() {
        let mut instance = plugin(|m| m.with_memory_max(64), None);

        assert_eq!(
            call_error(&mut instance, "hoard"),
            PluginCallError::OutOfMemory
        );
    }

    #[test]
    fn should_report_a_trap_when_the_plugin_panics() {
        let error = call_error(&mut unlimited(), "panic");

        let PluginCallError::Trap(reason) = error else {
            panic!("expected Trap, got: {error:?}");
        };
        assert!(reason.contains("unreachable"), "{reason}");
    }

    #[test]
    fn should_report_the_exit_code_when_wasi_exits_non_zero() {
        assert_eq!(
            call_error(&mut unlimited(), "exit"),
            PluginCallError::NonZeroExit(3)
        );
    }

    #[test]
    fn should_pass_the_plugins_own_error_message_through_when_the_plugin_sets_one() {
        assert_eq!(
            call_error(&mut unlimited(), "fail"),
            PluginCallError::Plugin("mod not found".to_owned())
        );
    }

    #[test]
    fn should_prefer_the_plugins_message_over_the_trap_when_a_trap_follows_it() {
        assert_eq!(
            call_error(&mut unlimited(), "fail_then_panic"),
            PluginCallError::Plugin("mod not found".to_owned())
        );
    }

    #[test]
    fn should_report_the_refusal_when_the_plugin_reports_a_refused_command() {
        assert_eq!(
            call_error(&mut unlimited(), "refused"),
            PluginCallError::CommandNotApproved {
                reason: CommandApprovalRefusal::TimedOut,
                details: "AccessViolation".to_owned(),
            }
        );
    }

    #[test]
    fn should_find_the_envelope_when_a_prefix_comes_before_it_and_text_after() {
        assert_eq!(
            call_error(&mut unlimited(), "refused_with_prefix_and_tail"),
            PluginCallError::CommandNotApproved {
                reason: CommandApprovalRefusal::Denied,
                details: "m".to_owned(),
            }
        );
    }

    #[test]
    fn should_keep_the_refusal_when_a_trap_follows_it() {
        assert_eq!(
            call_error(&mut unlimited(), "refused_then_panic"),
            PluginCallError::CommandNotApproved {
                reason: CommandApprovalRefusal::TimedOut,
                details: "AccessViolation".to_owned(),
            }
        );
    }

    #[test]
    fn should_report_the_refusal_of_another_plugin_when_the_call_chain_failed_that_way() {
        assert_eq!(
            call_error(&mut unlimited(), "refused_in_a_chain"),
            PluginCallError::CommandNotApproved {
                reason: CommandApprovalRefusal::Cancelled,
                details: "b said so".to_owned(),
            }
        );
    }

    #[test]
    fn should_report_the_refused_host_when_the_plugin_reports_a_host_not_allowed() {
        assert_eq!(
            call_error(&mut unlimited(), "host_refused"),
            PluginCallError::HostNotAllowed {
                host: "cdn.example".to_owned(),
                details: "cdn refused".to_owned(),
            }
        );
    }

    #[test]
    fn should_keep_the_refused_host_when_a_trap_follows_it() {
        assert_eq!(
            call_error(&mut unlimited(), "host_refused_then_panic"),
            PluginCallError::HostNotAllowed {
                host: "cdn.example".to_owned(),
                details: "cdn refused".to_owned(),
            }
        );
    }

    #[test]
    fn should_give_the_message_of_the_envelope_when_it_reports_nothing_else_the_host_knows() {
        for (name, message) in [
            ("unknown_reason", "m"),
            ("only_message", "just text"),
            ("http_status", "got 404"),
            ("unknown_http_error", "newer"),
        ] {
            assert_eq!(
                call_error(&mut unlimited(), name),
                PluginCallError::Plugin(message.to_owned()),
                "{name}"
            );
        }
    }

    #[test]
    fn should_pass_the_text_through_when_the_envelope_is_broken() {
        assert_eq!(
            call_error(&mut unlimited(), "broken_json"),
            PluginCallError::Plugin("aether-plugin-failure:{not json".to_owned())
        );
    }

    #[test]
    fn should_report_a_missing_export_when_the_function_does_not_exist() {
        assert_eq!(
            call_error(&mut unlimited(), "absent"),
            PluginCallError::MissingFunction
        );
    }

    /// Pins an Extism quirk: `call_get_error_code` drops a non-zero return code that comes
    /// without an error message, so such a call counts as a success. A plugin built on the PDK
    /// always sets the message; if Extism starts reporting the code, this test says so.
    #[test]
    fn should_accept_a_non_zero_return_code_when_the_plugin_sets_no_error_message() {
        let mut instance = unlimited();

        assert!(instance.call_bytes("quiet_failure", &[]).is_ok());
    }

    /// The message the user sees names the limit, not "function call failed".
    #[test]
    fn should_name_the_limit_in_the_message_when_a_limit_is_hit() {
        let mut instance = plugin(|m| m.with_timeout(Duration::from_millis(100)), None);

        let error = instance
            .call_bytes("spin", &[])
            .expect_err("the call should time out");

        assert_eq!(
            error.to_string(),
            "Function \"spin\" call failed in plugin \"test\": the plugin exceeded its time limit"
        );
    }
}
