use crate::features::plugins::PluginCallError;

// Extism hands back an untyped `anyhow::Error` and builds these messages itself
// (`extism-1.30.0/src/plugin.rs`, `raw_call`; `current_plugin.rs`, `memory_growing`). Text is
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
        _ if code == TRAP_CODE => PluginCallError::Trap(cause),
        _ => PluginCallError::Plugin(cause),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use extism::{Manifest, Plugin, PluginBuilder, Wasm};

    use crate::features::plugins::{
        PluginCallError, PluginError, PluginInstance, infra::extism::models::ExtismPluginInstance,
    };

    /// One export per way a call can fail. `$report` sets an error the way the PDK does:
    /// copies the message into Extism memory and hands it to `error_set`.
    const PLUGIN: &str = r#"(module
        (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
        (import "extism:host/env" "store_u8" (func $store_u8 (param i64 i32)))
        (import "extism:host/env" "error_set" (func $error_set (param i64)))
        (import "wasi_snapshot_preview1" "proc_exit" (func $proc_exit (param i32)))
        (memory (export "memory") 1)
        (data (i32.const 0) "mod not found")

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

        (func $report
            (local $offset i64)
            (local $i i32)
            (local.set $offset (call $alloc (i64.const 13)))
            (block $copied
                (loop $copy
                    (br_if $copied (i32.ge_u (local.get $i) (i32.const 13)))
                    (call $store_u8
                        (i64.add (local.get $offset) (i64.extend_i32_u (local.get $i)))
                        (i32.load8_u (local.get $i)))
                    (local.set $i (i32.add (local.get $i) (i32.const 1)))
                    (br $copy)))
            (call $error_set (local.get $offset)))

        (func (export "fail") (result i32)
            (call $report)
            (i32.const -1))

        (func (export "fail_then_panic") (result i32)
            (call $report)
            unreachable)

        (func (export "quiet_failure") (result i32)
            (i32.const 7)))"#;

    fn plugin(
        configure: impl FnOnce(Manifest) -> Manifest,
        fuel: Option<u64>,
    ) -> ExtismPluginInstance {
        let manifest = configure(Manifest::new([Wasm::data(PLUGIN.as_bytes().to_vec())]));
        let mut builder = PluginBuilder::new(manifest).with_wasi(true);
        if let Some(fuel) = fuel {
            builder = builder.with_fuel_limit(fuel);
        }
        let plugin: Plugin = builder.build().expect("the test plugin should load");

        ExtismPluginInstance::new(plugin, "test".to_owned())
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
