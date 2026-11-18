//! Guards the `log-always` feature of `tracing`: with a global tracing subscriber set,
//! events must still reach the `log` logger, or the app writes nothing to its log file.

use std::sync::Mutex;

use log::{Level, LevelFilter, Log, Metadata, Record};

static RECORDS: Mutex<Vec<String>> = Mutex::new(Vec::new());

struct Collector;

impl Log for Collector {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if record.level() <= Level::Debug {
            RECORDS.lock().unwrap().push(record.args().to_string());
        }
    }

    fn flush(&self) {}
}

#[test]
fn should_reach_the_log_when_a_global_subscriber_is_set() {
    log::set_boxed_logger(Box::new(Collector)).unwrap();
    log::set_max_level(LevelFilter::Trace);
    tracing::subscriber::set_global_default(aether_lib::core::span_capture()).unwrap();

    tracing::warn!(target: "aether_lib::guard", "event through the subscriber");

    let records = RECORDS.lock().unwrap();
    assert!(
        records
            .iter()
            .any(|r| r.contains("event through the subscriber")),
        "got {records:?}"
    );
}
