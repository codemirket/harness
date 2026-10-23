//! One initialization path for private native databases. Publication is atomic.
use crate::util;
use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, Transaction};
use std::{
    path::Path,
    time::{Duration, Instant},
};

pub(crate) fn open(
    path: &Path,
    application: i64,
    initialize: impl FnOnce(&Transaction<'_>) -> Result<()>,
) -> Result<Connection> {
    util::reject_symlink_components(path)?;
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .context("database filename")?;
    for suffix in ["-wal", "-shm", "-journal"] {
        util::reject_symlink_components(&path.with_file_name(format!("{name}{suffix}")))?;
    }
    let lock_path = path.with_file_name(format!("{name}.init.lock"));
    let started = Instant::now();
    let _lock = loop {
        match util::FileLock::acquire(&lock_path) {
            Ok(lock) => break lock,
            Err(error)
                if matches!(
                    error.downcast_ref::<std::fs::TryLockError>(),
                    Some(std::fs::TryLockError::WouldBlock)
                ) && started.elapsed() < Duration::from_secs(10) =>
            {
                std::thread::sleep(Duration::from_millis(10))
            }
            Err(error) => return Err(error.context("database initialization lock")),
        }
    };
    if !path.try_exists()? {
        // Only this fixed, unpublished initialization generation is recoverable.
        // The initialization lease proves no writer can still own it; no
        // acknowledged database is discarded after a crash before publication.
        let staging = path.with_file_name(format!(".{name}.initializing"));
        for suffix in ["", "-journal", "-wal", "-shm"] {
            let stale = path.with_file_name(format!(".{name}.initializing{suffix}"));
            util::reject_symlink_components(&stale)?;
            if stale.try_exists()? {
                ensure!(
                    stale.is_file(),
                    "initialization staging is not a regular file"
                );
                std::fs::remove_file(stale)?;
            }
        }
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&staging)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }
        let temp =
            tempfile::NamedTempFile::from_parts(file, tempfile::TempPath::try_from_path(staging)?);
        let mut conn = Connection::open(temp.path())?;
        let tx = conn.transaction()?;
        initialize(&tx)?;
        tx.pragma_update(None, "application_id", application)?;
        tx.pragma_update(None, "user_version", 1)?;
        tx.commit()?;
        drop(conn);
        temp.as_file().sync_all()?;
        temp.persist_noclobber(path)
            .context("publish initialized database")?;
        #[cfg(unix)]
        std::fs::File::open(path.parent().context("database parent")?)?.sync_all()?;
    }
    ensure!(path.is_file(), "database is not a regular file");
    let conn = Connection::open(path)?;
    conn.busy_timeout(Duration::from_secs(10))?;
    let app: i64 = conn.pragma_query_value(None, "application_id", |r| r.get(0))?;
    let version: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    ensure!(
        app == application && version == 1,
        "unsupported native database identity/version; no legacy conversion"
    );
    conn.pragma_update(None, "foreign_keys", true)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "FULL")?;
    conn.pragma_update(None, "max_page_count", 131072)?;
    conn.pragma_update(None, "wal_autocheckpoint", 256)?;
    conn.pragma_update(None, "journal_size_limit", 8 * 1024 * 1024)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interrupted_unpublished_initialization_is_rebuilt_without_adopting_partial_state() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let path = root.join("facts.sqlite");
        std::fs::write(
            root.join(".facts.sqlite.initializing"),
            b"incomplete sqlite transaction",
        )
        .unwrap();
        std::fs::write(
            root.join(".facts.sqlite.initializing-journal"),
            b"partial journal",
        )
        .unwrap();
        let conn = open(&path, 1234, |tx| {
            tx.execute_batch("CREATE TABLE proof(value INTEGER); INSERT INTO proof VALUES(42)")?;
            Ok(())
        })
        .unwrap();
        assert_eq!(
            conn.query_row("SELECT value FROM proof", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            42
        );
        assert!(!root.join(".facts.sqlite.initializing").exists());
        assert!(!root.join(".facts.sqlite.initializing-journal").exists());
        drop(conn);
        assert!(open(&path, 5678, |_| Ok(())).is_err());
        let conn = open(&path, 1234, |_| {
            panic!("published database must not reinitialize")
        })
        .unwrap();
        assert_eq!(
            conn.query_row("SELECT value FROM proof", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            42
        );
    }
}
