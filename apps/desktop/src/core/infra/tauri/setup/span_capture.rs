use tracing::Subscriber;
use tracing_error::ErrorLayer;
use tracing_subscriber::{Layer, filter::filter_fn, layer::SubscriberExt, registry};

/// Whether spans of this target belong to the app, so only they enter a span trace.
fn is_app_target(target: &str) -> bool {
    [("aether_core"), ("aether_lib")].iter().any(|root| {
        target == *root
            || target
                .strip_prefix(root)
                .is_some_and(|rest| rest.starts_with("::"))
    })
}

/// A subscriber that only records the spans of the app, for `SpanTrace::capture`.
///
/// It prints nothing: events reach the log through the `log-always` feature of `tracing`.
/// Dependencies' spans are left out so a trace shows only our own frames.
pub fn span_capture() -> impl Subscriber + Send + Sync {
    registry()
        .with(ErrorLayer::default().with_filter(filter_fn(|meta| is_app_target(meta.target()))))
}

#[cfg(test)]
mod tests {
    use tracing::{Level, span};
    use tracing_error::SpanTrace;

    use super::*;

    /// Targets of the recorded frames, innermost first.
    fn frames() -> Vec<String> {
        let mut targets = Vec::new();
        SpanTrace::capture().with_spans(|meta, _| {
            targets.push(meta.target().to_owned());
            true
        });
        targets
    }

    #[test]
    fn should_match_only_app_targets() {
        assert!(is_app_target("aether_core"));
        assert!(is_app_target("aether_core::features::instance"));
        assert!(is_app_target("aether_lib::features::update"));
        assert!(!is_app_target("aether_core_extra"));
        assert!(!is_app_target("hyper::client"));
        assert!(!is_app_target("tracing::span"));
    }

    #[test]
    fn should_record_only_the_app_frame_when_a_foreign_span_is_innermost() {
        tracing::subscriber::with_default(span_capture(), || {
            let n = span!(target: "aether_core::x", Level::DEBUG, "own")
                .in_scope(|| span!(target: "hyper", Level::DEBUG, "foreign").in_scope(frames));
            assert_eq!(n, ["aether_core::x"]);
        });
    }
}
