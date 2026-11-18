//! The log functions Extism links into every plugin (`extism:host/env`: `log_info`, `log_error`
//! and so on, what the `info!` and `error!` macros of `extism-pdk` call), taken over by ones of
//! the core. Extism's own write the plugin's text to the log as it is, which would let a plugin
//! forge a line of the core or flood the file around the limits of the `log` host function.
//!
//! The core's functions go through [`handle_log`], so both ways in log the same text the same
//! way. Extism lets a host function of the same name shadow its own; the tests at the bottom
//! pin that, so that an update of Extism cannot silently bring the hole back.
//!
//! Not covered: the text of an error a plugin sets (`error_set`), which Extism logs itself.

use aether_core_plugin_api::v0::LogLevelDto;
use extism::{CurrentPlugin, Function, PTR, UserData, Val};

use crate::{features::instance::PluginId, shared::limits::MAX_PLUGIN_LOG_BYTES};

use super::features::handle_log;

const NAMESPACE: &str = "extism:host/env";

/// What one of these functions needs to know when the plugin calls it.
#[derive(Clone)]
struct LogShadow {
    plugin_id: PluginId,
    level: LogLevelDto,
    /// Whether the manifest declares `log`: a plugin that does not may not write to the log by
    /// this way either, so its message is dropped.
    allowed: bool,
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "the signature is the one `Function::new` wants: the user data comes by value"
)]
fn shadow(
    plugin: &mut CurrentPlugin,
    input: &[Val],
    _output: &mut [Val],
    data: UserData<LogShadow>,
) -> Result<(), extism::Error> {
    let offset = input
        .first()
        .and_then(Val::i64)
        .unwrap_or_default()
        .cast_unsigned();
    let handle = plugin
        .memory_handle(offset)
        .ok_or_else(|| extism::Error::msg("invalid handle offset for log message"))?;

    // The settings are cloned out so the lock is not held while the line is written.
    let state = data.get()?;
    let shadow = state.lock().map(|guard| guard.clone());

    // Only the head of the block is read, and only when it will be written: the block may be as
    // big as the plugin's memory. The memory is released whether or not the text is written.
    let text = match &shadow {
        Ok(shadow) if shadow.allowed => plugin.memory_bytes(handle).map(|bytes| {
            let head = &bytes[..bytes.len().min(MAX_PLUGIN_LOG_BYTES + 4)];
            String::from_utf8_lossy(head).into_owned()
        }),
        _ => Ok(String::new()),
    };
    plugin.memory_free(handle)?;

    let shadow = shadow.map_err(|_| extism::Error::msg("log state poisoned"))?;
    if shadow.allowed {
        match text {
            Ok(text) => handle_log(&shadow.plugin_id, shadow.level, &text),
            Err(_) => handle_log(
                &shadow.plugin_id,
                LogLevelDto::Error,
                "Could not read a log message",
            ),
        }
    }
    Ok(())
}

/// `functions`, the host functions of a plugin, with the five that take the place of Extism's
/// log functions added. They are linked whether or not the plugin declares `log`: a plugin that
/// does not has its messages dropped.
pub(crate) fn with_extism_log_functions(
    mut functions: Vec<Function>,
    plugin_id: &PluginId,
    declared: &[String],
) -> Vec<Function> {
    let allowed = declared.iter().any(|name| name == "log");

    functions.extend(
        [
            ("log_error", LogLevelDto::Error),
            ("log_warn", LogLevelDto::Warn),
            ("log_info", LogLevelDto::Info),
            ("log_debug", LogLevelDto::Debug),
            ("log_trace", LogLevelDto::Trace),
        ]
        .into_iter()
        .map(|(name, level)| {
            let data = LogShadow {
                plugin_id: plugin_id.clone(),
                level,
                allowed,
            };
            Function::new(name, [PTR], [], UserData::new(data), shadow).with_namespace(NAMESPACE)
        }),
    );
    functions
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use extism::{Manifest, PluginBuilder, Wasm};
    use tracing::{Event, Subscriber, field::Visit};
    use tracing_subscriber::{Layer, layer::SubscriberExt, registry};

    use super::*;

    /// Exports `via_<level>` for each level: it hands its input to the Extism log function of
    /// that name, as the macros of `extism-pdk` do.
    const PLUGIN: &str = r#"(module
        (import "extism:host/env" "input_offset" (func $input_offset (result i64)))
        (import "extism:host/env" "log_error" (func $error (param i64)))
        (import "extism:host/env" "log_warn" (func $warn (param i64)))
        (import "extism:host/env" "log_info" (func $info (param i64)))
        (import "extism:host/env" "log_debug" (func $debug (param i64)))
        (import "extism:host/env" "log_trace" (func $trace (param i64)))
        (func (export "via_error") (result i32) (call $error (call $input_offset)) (i32.const 0))
        (func (export "via_warn") (result i32) (call $warn (call $input_offset)) (i32.const 0))
        (func (export "via_info") (result i32) (call $info (call $input_offset)) (i32.const 0))
        (func (export "via_debug") (result i32) (call $debug (call $input_offset)) (i32.const 0))
        (func (export "via_trace") (result i32) (call $trace (call $input_offset)) (i32.const 0)))"#;

    /// `(target, message)` of every event.
    #[derive(Clone, Default)]
    struct Events(Arc<Mutex<Vec<(String, String)>>>);

    struct Message(String);

    impl Visit for Message {
        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
            if field.name() == "message" {
                self.0 = format!("{value:?}");
            }
        }
    }

    impl<S: Subscriber> Layer<S> for Events {
        fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
            let mut message = Message(String::new());
            event.record(&mut message);
            self.0
                .lock()
                .expect("events")
                .push((event.metadata().target().to_owned(), message.0));
        }
    }

    fn call(function: &str, input: &str, declared: &[&str]) -> Vec<(String, String)> {
        let events = Events::default();
        let subscriber = registry().with(events.clone());
        let declared: Vec<String> = declared.iter().map(|name| (*name).to_owned()).collect();

        tracing::subscriber::with_default(subscriber, || {
            let functions =
                with_extism_log_functions(Vec::new(), &PluginId::from_static("test"), &declared);
            let mut plugin = PluginBuilder::new(Manifest::new([Wasm::data(PLUGIN)]))
                .with_wasi(false)
                .with_functions(functions)
                .build()
                .expect("plugin builds");
            plugin
                .call::<&str, ()>(function, input)
                .expect("call succeeds");
        });

        // Extism traces its own work at debug level; only what reaches the log as the plugin's
        // counts.
        let recorded = events.0.lock().expect("events").clone();
        recorded
            .into_iter()
            .filter(|(target, _)| target == "plugin" || target == "extism::pdk")
            .collect()
    }

    #[test]
    fn should_link_the_five_functions_next_to_the_others() {
        let functions = with_extism_log_functions(Vec::new(), &PluginId::from_static("test"), &[]);

        let mut names: Vec<_> = functions.iter().map(Function::name).collect();
        names.sort_unstable();

        assert_eq!(
            names,
            [
                "log_debug",
                "log_error",
                "log_info",
                "log_trace",
                "log_warn"
            ]
        );
    }

    #[test]
    fn should_log_the_escaped_text_under_the_core_target_when_a_plugin_uses_an_extism_function() {
        for level in ["error", "warn", "info", "debug", "trace"] {
            let events = call(&format!("via_{level}"), "a\n[FAKE] line\u{1b}[2J", &["log"]);

            assert_eq!(
                events,
                [(
                    "plugin".to_owned(),
                    r"[test]: a\n[FAKE] line\u{1b}[2J".to_owned()
                )],
                "{level}: the core's function must shadow the one of Extism (target extism::pdk)"
            );
        }
    }

    #[test]
    fn should_cut_a_long_message_when_a_plugin_uses_an_extism_function() {
        let events = call("via_info", &"x".repeat(MAX_PLUGIN_LOG_BYTES * 4), &["log"]);

        assert_eq!(events.len(), 1);
        assert!(
            events[0].1.len() < MAX_PLUGIN_LOG_BYTES + 100,
            "{}",
            events[0].1.len()
        );
    }

    #[test]
    fn should_log_nothing_when_the_plugin_does_not_declare_log() {
        assert!(call("via_error", "boom", &[]).is_empty());
    }
}
