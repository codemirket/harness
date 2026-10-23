//! Fixed owned-file accounting: no recursive hot-path traversal or cached size authority.
use crate::util;
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use std::{fs, path::Path};
const LIMIT: u64 = 512 * 1024 * 1024;
const RECOVERY: u64 = 64 * 1024 * 1024;

pub(super) fn usage(directory: &Path) -> Result<u64> {
    let mut bytes = 0u64;
    for database in ["facts.sqlite", "execution.sqlite"] {
        for suffix in ["", "-wal", "-shm", "-journal"] {
            let path = directory.join(format!("{database}{suffix}"));
            util::reject_symlink_components(&path)?;
            match fs::symlink_metadata(&path) {
                Ok(meta) => {
                    ensure!(
                        meta.is_file(),
                        "native storage requires regular owned files"
                    );
                    bytes = bytes
                        .checked_add(meta.len())
                        .ok_or_else(|| anyhow::anyhow!("storage overflow"))?;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
    }
    Ok(bytes)
}
pub(super) fn admit(directory: &Path, operation: &str, args: &Value) -> Result<Value> {
    let used = usage(directory)?;
    let recovery = matches!(
        operation,
        "memory_forget" | "workflow_forget" | "data_apply"
    ) || (operation == "workflow_resume"
        && matches!(
            args["reply"]["decision"].as_str(),
            Some("cancelled" | "blocked")
        ));
    let mutation = matches!(
        operation,
        "memory_put"
            | "memory_forget"
            | "workflow_start"
            | "workflow_resume"
            | "workflow_hostcheck"
            | "workflow_forget"
            | "data_apply"
    );
    // Maximum 4 MiB state/delta plus SQLite indexes, WAL and page splits.
    // Reads/exports remain available above the write envelope.
    let reservation = if mutation { 32 * 1024 * 1024 } else { 0 };
    let ceiling = if recovery { LIMIT } else { LIMIT - RECOVERY };
    if mutation {
        ensure!(
            used.saturating_add(reservation) <= ceiling,
            "native project storage admission exhausted; read/export and reserved recovery remain available"
        );
    }
    Ok(
        json!({"owned_bytes":used,"write_reservation_bytes":reservation,"limit_bytes":LIMIT,"recovery_reserve_bytes":RECOVERY,"method":"fixed_database_and_sidecar_metadata_v1"}),
    )
}

pub(super) fn doctor(paths: &crate::paths::Paths) -> Result<Value> {
    let root = crate::packages::agent::data(paths);
    util::reject_symlink_components(&root)?;
    if !root.exists() {
        return Ok(json!({"ok":true,"projects":0,"owned_bytes":0}));
    }
    let _lease = crate::maintenance::exclusive(paths)?;
    let mut projects = 0usize;
    let mut total = 0u64;
    for entry in fs::read_dir(&root)? {
        let entry = entry?;
        let directory = entry.path();
        util::reject_symlink_components(&directory)?;
        let scope = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow::anyhow!("invalid native scope"))?;
        ensure!(
            scope.len() == 64
                && scope.bytes().all(|b| b.is_ascii_hexdigit())
                && entry.file_type()?.is_dir(),
            "invalid native project directory"
        );
        projects += 1;
        ensure!(projects <= 4096, "native doctor project scan exceeds bound");
        let used = usage(&directory)?;
        total = total.saturating_add(used);
        ensure!(used <= LIMIT, "native project storage quota exceeded");
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            util::reject_symlink_components(&entry.path())?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("invalid storage filename"))?;
            ensure!(
                entry.file_type()?.is_file()
                    && (name == "operation.lock"
                        || ["facts.sqlite", "execution.sqlite"].iter().any(|base| [
                            "",
                            "-wal",
                            "-shm",
                            "-journal",
                            ".init.lock"
                        ]
                        .iter()
                        .any(|suffix| name == format!("{base}{suffix}")))),
                "unexpected native storage entry"
            );
        }
        for (name, app) in [
            ("facts.sqlite", 0x4d464143),
            ("execution.sqlite", 0x4d52554e),
        ] {
            let path = directory.join(name);
            if !path.exists() {
                continue;
            }
            let db = rusqlite::Connection::open_with_flags(
                path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )?;
            let actual: i64 = db.pragma_query_value(None, "application_id", |r| r.get(0))?;
            let version: i64 = db.pragma_query_value(None, "user_version", |r| r.get(0))?;
            ensure!(
                actual == app && version == 1,
                "unsupported native storage identity"
            );
            let stored: String = db.query_row("SELECT scope FROM identity", [], |r| r.get(0))?;
            ensure!(stored == scope, "native storage scope mismatch");
            let integrity: String = db.pragma_query_value(None, "integrity_check", |r| r.get(0))?;
            ensure!(integrity == "ok", "native SQLite integrity failure");
            ensure!(
                db.prepare("PRAGMA foreign_key_check")?
                    .query([])?
                    .next()?
                    .is_none(),
                "native foreign key integrity failure"
            );
            if name == "facts.sqlite" {
                crate::memory::store::FactStore::validate_backup(&db, &scope)?;
                let model: String = db.query_row("SELECT model FROM identity", [], |r| r.get(0))?;
                ensure!(
                    model == super::native::model_identity(),
                    "native embedding identity mismatch"
                );
            } else {
                super::journal::Journal::validate_backup(&db, &scope)?;
            }
        }
    }
    Ok(
        json!({"ok":true,"projects":projects,"owned_bytes":total,"method":"full_owned_paths_sqlite_integrity_and_foreign_keys"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accounting_includes_wal_and_preserves_recovery_at_normal_write_limit() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let file = fs::File::create(root.join("facts.sqlite-wal")).unwrap();
        file.set_len(LIMIT - RECOVERY).unwrap();
        assert_eq!(usage(&root).unwrap(), LIMIT - RECOVERY);
        assert!(admit(&root, "memory_put", &json!({})).is_err());
        assert!(admit(&root, "memory_forget", &json!({})).is_ok());
        assert!(admit(&root, "memory_read", &json!({})).is_ok());
        file.set_len(LIMIT).unwrap();
        assert!(admit(&root, "memory_forget", &json!({})).is_err());
        assert!(admit(&root, "data_export", &json!({})).is_ok());
    }
    #[cfg(unix)]
    #[test]
    fn accounting_never_follows_sidecar_links() {
        let dir = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink("/tmp", dir.path().join("execution.sqlite-wal")).unwrap();
        assert!(usage(dir.path()).is_err());
    }
}
