use std::{path::Path, sync::Arc, time::SystemTime};

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::features::{
    instance::InstanceId, process::CrashEvidenceService, settings::LocationInfo,
};

/// Looks for the crash report of the game (`crash-reports`) and the fatal error log of the JVM
/// (`hs_err_pid<pid>.log`, written to the folder the game runs in).
pub struct FsCrashEvidence {
    location_info: Arc<LocationInfo>,
}

impl FsCrashEvidence {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }
}

fn written_since(path: &Path, since: SystemTime) -> bool {
    std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .is_ok_and(|modified| modified >= since)
}

fn crash_report_since(dir: &Path, since: SystemTime) -> bool {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .any(|entry| written_since(&entry.path(), since))
}

#[async_trait]
impl CrashEvidenceService for FsCrashEvidence {
    async fn found(&self, instance_id: &InstanceId, pid: u32, since: DateTime<Utc>) -> bool {
        let since = SystemTime::from(since);
        let reports = self.location_info.crash_reports_dir(instance_id);
        let jvm_log = self
            .location_info
            .instance_dir(instance_id)
            .join(format!("hs_err_pid{pid}.log"));
        tokio::task::spawn_blocking(move || {
            crash_report_since(&reports, since) || written_since(&jvm_log, since)
        })
        .await
        .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn setup() -> (tempfile::TempDir, FsCrashEvidence, InstanceId) {
        let dir = tempfile::tempdir().unwrap();
        let location_info = Arc::new(LocationInfo::new(
            dir.path().join("settings"),
            dir.path().to_path_buf(),
        ));
        let id = InstanceId::try_from("test-instance").unwrap();
        std::fs::create_dir_all(location_info.instance_dir(&id)).unwrap();
        (dir, FsCrashEvidence::new(location_info), id)
    }

    fn set_age(path: &Path, ago: Duration) {
        let file = std::fs::File::options().write(true).open(path).unwrap();
        file.set_modified(SystemTime::now() - ago).unwrap();
    }

    #[tokio::test]
    async fn should_find_a_crash_report_written_after_the_start() {
        let (_dir, evidence, id) = setup();
        let reports = evidence.location_info.crash_reports_dir(&id);
        std::fs::create_dir_all(&reports).unwrap();
        std::fs::write(reports.join("crash-1.txt"), "boom").unwrap();
        let since = Utc::now() - chrono::Duration::seconds(60);

        assert!(evidence.found(&id, 1, since).await);
    }

    #[tokio::test]
    async fn should_not_find_a_crash_report_older_than_the_start() {
        let (_dir, evidence, id) = setup();
        let reports = evidence.location_info.crash_reports_dir(&id);
        std::fs::create_dir_all(&reports).unwrap();
        let report = reports.join("crash-1.txt");
        std::fs::write(&report, "boom").unwrap();
        set_age(&report, Duration::from_secs(3600));
        let since = Utc::now() - chrono::Duration::seconds(60);

        assert!(!evidence.found(&id, 1, since).await);
    }

    #[tokio::test]
    async fn should_find_the_fatal_log_of_the_jvm_by_pid_only() {
        let (_dir, evidence, id) = setup();
        let instance_dir = evidence.location_info.instance_dir(&id);
        std::fs::write(instance_dir.join("hs_err_pid42.log"), "fatal").unwrap();
        let since = Utc::now() - chrono::Duration::seconds(60);

        assert!(evidence.found(&id, 42, since).await);
        assert!(!evidence.found(&id, 43, since).await);
    }

    #[tokio::test]
    async fn should_not_find_a_fatal_log_of_the_jvm_older_than_the_start() {
        let (_dir, evidence, id) = setup();
        let log = evidence
            .location_info
            .instance_dir(&id)
            .join("hs_err_pid42.log");
        std::fs::write(&log, "fatal").unwrap();
        set_age(&log, Duration::from_secs(3600));
        let since = Utc::now() - chrono::Duration::seconds(60);

        assert!(!evidence.found(&id, 42, since).await);
    }

    #[tokio::test]
    async fn should_find_nothing_when_the_game_left_no_trace() {
        let (_dir, evidence, id) = setup();

        assert!(!evidence.found(&id, 1, Utc::now()).await);
    }
}
