use std::{env, str::FromStr};

use log::LevelFilter;
use tauri_plugin_log::fern::colors::ColoredLevelConfig;

const DEFAULT_LOG_MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;
const DEFAULT_LOG_COUNT: usize = 5;

/// Resolved log levels: the level for everything else plus per-target overrides.
///
/// Later entries of `targets` win over earlier ones, as `fern` replaces a repeated target.
#[derive(Debug, PartialEq, Eq)]
struct LogLevels {
    default: LevelFilter,
    targets: Vec<(String, LevelFilter)>,
}

#[cfg(test)]
impl LogLevels {
    fn level_of(&self, target: &str) -> LevelFilter {
        self.targets
            .iter()
            .rev()
            .find(|(t, _)| t == target)
            .map_or(self.default, |(_, level)| *level)
    }
}

/// Builds the levels from the `RUST_LOG` value (`level` and `target=level` directives).
///
/// Noisy dependencies stay quiet even with `RUST_LOG=trace`; raise them with an explicit
/// `hyper=trace`. Directives that cannot be parsed are skipped.
fn resolve_log_levels(rust_log: Option<&str>, is_debug: bool) -> LogLevels {
    let (default_level, external_level) = if is_debug {
        (LevelFilter::Debug, LevelFilter::Info)
    } else {
        (LevelFilter::Info, LevelFilter::Warn)
    };

    let mut default = default_level;
    let mut core_level = default_level;
    let mut overrides = Vec::new();

    for directive in rust_log.unwrap_or_default().split(',') {
        let directive = directive.trim();
        if directive.is_empty() {
            continue;
        }

        match directive.split_once('=') {
            Some((target, level)) => {
                let target = target.trim();
                if !is_valid_target(target) {
                    continue;
                }
                if let Ok(level) = LevelFilter::from_str(level.trim()) {
                    overrides.push((target.to_owned(), level));
                }
            }
            None => {
                if let Ok(level) = LevelFilter::from_str(directive) {
                    default = level;
                    core_level = level;
                } else if is_valid_target(directive) {
                    overrides.push((directive.to_owned(), LevelFilter::Trace));
                }
            }
        }
    }

    let mut targets = vec![
        // Core lib
        ("aether_core".to_owned(), core_level),
        // Platform
        ("tao".to_owned(), LevelFilter::Off),
        ("wry".to_owned(), LevelFilter::Off),
        // Network stack
        ("reqwest".to_owned(), external_level),
        ("rustls".to_owned(), external_level),
        ("h2".to_owned(), external_level),
        ("hyper".to_owned(), external_level),
        ("hyper_util".to_owned(), external_level),
        // Plugin runtime
        ("wasmtime".to_owned(), external_level),
        ("wasmtime_internal_cranelift".to_owned(), LevelFilter::Info),
        ("cranelift_codegen".to_owned(), external_level),
        // Tauri plugins
        ("tauri_plugin_updater".to_owned(), LevelFilter::Info),
        // Span lifecycle records
        ("tracing::span".to_owned(), external_level),
    ];
    // A level below the presets quiets the dependencies too, never the other way round.
    for (_, level) in &mut targets {
        *level = (*level).min(default);
    }
    targets.extend(overrides);

    LogLevels { default, targets }
}

fn is_valid_target(target: &str) -> bool {
    !target.is_empty() && !target.contains(['[', ']', '/', ' '])
}

#[must_use]
pub fn default_log_builder() -> tauri_plugin_log::Builder {
    let rust_log = env::var("RUST_LOG").ok();
    let levels = resolve_log_levels(rust_log.as_deref(), cfg!(debug_assertions));

    let mut builder = tauri_plugin_log::Builder::new()
        .with_colors(ColoredLevelConfig::default())
        .max_file_size(DEFAULT_LOG_MAX_FILE_SIZE.into())
        .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(
            DEFAULT_LOG_COUNT,
        ))
        .level(levels.default);

    for (target, level) in levels.targets {
        builder = builder.level_for(target, level);
    }

    builder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_use_defaults_when_rust_log_is_missing_or_blank() {
        for value in [None, Some(""), Some("   "), Some(" , ,")] {
            let levels = resolve_log_levels(value, false);
            assert_eq!(levels.default, LevelFilter::Info);
            assert_eq!(levels.level_of("aether_core"), LevelFilter::Info);
            assert_eq!(levels.level_of("hyper"), LevelFilter::Warn);
        }
        let debug = resolve_log_levels(None, true);
        assert_eq!(debug.default, LevelFilter::Debug);
        assert_eq!(debug.level_of("hyper"), LevelFilter::Info);
    }

    #[test]
    fn should_apply_the_level_to_the_core_when_it_is_plain() {
        let levels = resolve_log_levels(Some("debug"), false);
        assert_eq!(levels.default, LevelFilter::Debug);
        assert_eq!(levels.level_of("aether_core"), LevelFilter::Debug);
    }

    #[test]
    fn should_keep_noisy_dependencies_quiet_when_the_level_is_trace() {
        let levels = resolve_log_levels(Some("trace"), false);
        assert_eq!(levels.level_of("hyper"), LevelFilter::Warn);
        assert_eq!(levels.level_of("wry"), LevelFilter::Off);
    }

    #[test]
    fn should_apply_target_level_over_the_defaults() {
        let levels = resolve_log_levels(Some("warn,hyper=trace,aether_core=error"), false);
        assert_eq!(levels.default, LevelFilter::Warn);
        assert_eq!(levels.level_of("hyper"), LevelFilter::Trace);
        assert_eq!(levels.level_of("aether_core"), LevelFilter::Error);
    }

    #[test]
    fn should_let_the_last_directive_win_for_a_repeated_target() {
        let levels = resolve_log_levels(Some("hyper=trace,hyper=off"), false);
        assert_eq!(levels.level_of("hyper"), LevelFilter::Off);
    }

    #[test]
    fn should_skip_invalid_directives() {
        let levels = resolve_log_levels(Some("hyper=loud,[span]=debug,a/b=info,=info,warn"), false);
        assert_eq!(levels.default, LevelFilter::Warn);
        assert_eq!(levels.level_of("hyper"), LevelFilter::Warn);
        assert_eq!(
            levels.targets.len(),
            resolve_log_levels(None, false).targets.len()
        );
    }

    #[test]
    fn should_ignore_case_and_whitespace_when_parsing() {
        let levels = resolve_log_levels(Some(" DEBUG , Hyper = TRACE"), false);
        assert_eq!(levels.default, LevelFilter::Debug);
        assert_eq!(levels.level_of("Hyper"), LevelFilter::Trace);
    }

    #[test]
    fn should_use_trace_when_a_directive_is_a_bare_target() {
        let levels = resolve_log_levels(Some("aether_lib"), false);
        assert_eq!(levels.default, LevelFilter::Info);
        assert_eq!(levels.level_of("aether_lib"), LevelFilter::Trace);
    }

    #[test]
    fn should_lower_noisy_dependencies_when_the_level_is_below_them() {
        for value in ["off", "error"] {
            let levels = resolve_log_levels(Some(value), true);
            for target in [
                "hyper",
                "tracing::span",
                "tauri_plugin_updater",
                "wasmtime_internal_cranelift",
            ] {
                assert!(
                    levels.level_of(target) <= levels.default,
                    "{target} with {value}"
                );
            }
        }
    }

    #[test]
    fn should_keep_span_lifecycle_records_quiet_when_the_level_is_not_below_it() {
        for value in [None, Some("trace"), Some("debug")] {
            for is_debug in [true, false] {
                let levels = resolve_log_levels(value, is_debug);
                let expected = if is_debug {
                    LevelFilter::Info
                } else {
                    LevelFilter::Warn
                };
                assert_eq!(levels.level_of("tracing::span"), expected);
            }
        }
    }
}
