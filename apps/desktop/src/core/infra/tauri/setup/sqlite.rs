use std::path::PathBuf;

use aether_core::shared::io::infra::create_dir_all;

pub async fn create_pool(db_path: PathBuf) -> crate::Result<sqlx::SqlitePool> {
    #[cfg(unix)]
    {
        let path = db_path.clone();
        if let Err(err) = tokio::task::spawn_blocking(move || restrict::prepare(&path)).await {
            tracing::warn!("Failed to restrict database permissions: {err}");
        }
    }

    if let Some(parent) = db_path.parent() {
        create_dir_all(parent)
            .await
            .map_err(|err| crate::Error::LaunchError(err.to_string()))?;
    }

    let connection_options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        // Overwrite pages freed from now on, so a logout does not leave token bytes in new free space.
        .pragma("secure_delete", "on");

    let pool = sqlx::SqlitePool::connect_with(connection_options)
        .await
        .map_err(|err| crate::Error::LaunchError(err.to_string()))?;

    Ok(pool)
}

/// Keeps the database (it holds account tokens) away from other users on Unix.
/// Every failure is only logged: the launcher must still start.
#[cfg(unix)]
mod restrict {
    use std::fs::{self, DirBuilder, OpenOptions};
    use std::io;
    use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
    use std::path::{Path, PathBuf};

    const DIR_MODE: u32 = 0o700;
    const FILE_MODE: u32 = 0o600;

    pub fn prepare(db_path: &Path) {
        if let Some(parent) = db_path.parent().filter(|p| !p.as_os_str().is_empty())
            && let Err(err) = prepare_dir(parent)
        {
            tracing::warn!(
                "Failed to restrict permissions of {}: {err}",
                parent.display()
            );
        }

        match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .mode(FILE_MODE)
            .open(db_path)
        {
            Ok(_) => {}
            Err(err) => tracing::warn!("Failed to pre-create {}: {err}", db_path.display()),
        }

        for path in db_files(db_path) {
            restrict_file(&path);
        }
    }

    fn db_files(db_path: &Path) -> [PathBuf; 3] {
        let with_suffix = |suffix: &str| {
            let mut name = db_path.as_os_str().to_owned();
            name.push(suffix);
            PathBuf::from(name)
        };
        [
            db_path.to_path_buf(),
            with_suffix("-wal"),
            with_suffix("-shm"),
        ]
    }

    fn restrict_file(path: &Path) {
        if !path.exists() {
            return;
        }
        if let Err(err) = fs::set_permissions(path, fs::Permissions::from_mode(FILE_MODE)) {
            tracing::warn!(
                "Failed to restrict permissions of {}: {err}",
                path.display()
            );
        }
    }

    /// Ancestors keep default permissions, only the last component becomes private.
    fn prepare_dir(dir: &Path) -> io::Result<()> {
        if !dir.exists() {
            if let Some(grandparent) = dir.parent().filter(|p| !p.as_os_str().is_empty()) {
                fs::create_dir_all(grandparent)?;
            }
            match DirBuilder::new().mode(DIR_MODE).create(dir) {
                Ok(()) => {}
                Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {}
                Err(err) => return Err(err),
            }
        }
        fs::set_permissions(dir, fs::Permissions::from_mode(DIR_MODE))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn mode(path: &Path) -> u32 {
            fs::metadata(path).unwrap().permissions().mode() & 0o777
        }

        #[test]
        fn should_create_a_private_dir_and_file_when_none_exist() {
            let tmp = tempfile::tempdir().unwrap();
            let db = tmp.path().join("data").join("app.db");
            prepare(&db);
            assert_eq!(mode(db.parent().unwrap()), 0o700);
            assert_eq!(mode(&db), 0o600);
            assert_eq!(fs::metadata(&db).unwrap().len(), 0);
        }

        #[test]
        fn should_tighten_existing_files_and_keep_their_content_when_they_are_open_to_others() {
            let tmp = tempfile::tempdir().unwrap();
            let dir = tmp.path().join("data");
            fs::create_dir(&dir).unwrap();
            fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
            let db = dir.join("app.db");
            for name in ["app.db", "app.db-wal", "app.db-shm"] {
                let p = dir.join(name);
                fs::write(&p, b"secret").unwrap();
                fs::set_permissions(&p, fs::Permissions::from_mode(0o644)).unwrap();
            }
            prepare(&db);
            assert_eq!(mode(&dir), 0o700);
            for name in ["app.db", "app.db-wal", "app.db-shm"] {
                let p = dir.join(name);
                assert_eq!(mode(&p), 0o600, "{name}");
                assert_eq!(fs::read(&p).unwrap(), b"secret");
            }
        }

        #[test]
        fn should_leave_ancestor_dirs_alone_when_creating_the_data_dir() {
            let tmp = tempfile::tempdir().unwrap();
            let outer = tmp.path().join("a");
            let inner = outer.join("b");
            fs::create_dir_all(&inner).unwrap();
            // Explicit modes, so the test does not depend on the process umask.
            fs::set_permissions(&outer, fs::Permissions::from_mode(0o755)).unwrap();
            fs::set_permissions(&inner, fs::Permissions::from_mode(0o755)).unwrap();
            let db = inner.join("data").join("app.db");
            prepare(&db);
            assert_eq!(mode(db.parent().unwrap()), 0o700);
            assert_eq!(mode(&inner), 0o755);
            assert_eq!(mode(&outer), 0o755);
        }

        #[test]
        fn should_not_panic_when_the_parent_is_not_a_directory() {
            let tmp = tempfile::tempdir().unwrap();
            // The "directory" is a regular file, so every step fails.
            let blocker = tmp.path().join("blocker");
            fs::write(&blocker, b"x").unwrap();
            prepare(&blocker.join("data").join("app.db"));
            assert_eq!(fs::read(&blocker).unwrap(), b"x");
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[tokio::test]
    async fn should_open_a_private_db_when_the_path_is_fresh() {
        let tmp = tempfile::tempdir().unwrap();
        let db = tmp.path().join("data").join("app.db");
        let pool = create_pool(db.clone()).await.unwrap();
        drop(pool.acquire().await.unwrap());
        pool.close().await;
        let mode = std::fs::metadata(&db).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}
