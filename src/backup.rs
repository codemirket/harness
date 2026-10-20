//! Offline-consistent durable-state images, not logical exports or installation copies.
use crate::{maintenance, paths::Paths, util};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

const MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_FILES: usize = 20_000;
const MANIFEST_LIMIT: u64 = 8 * 1024 * 1024;
const ROOTS: [&str; 3] = ["state.db", "runs", "runtime"];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Entry {
    bytes: u64,
    sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: u32,
    home: PathBuf,
    version: String,
    source_sha256: String,
    runtime_recipe: String,
    roots: Vec<String>,
    files: BTreeMap<String, Entry>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pending {
    sha256: String,
    stage: String,
    manifest: Manifest,
}

fn destination(paths: &Paths, name: &str) -> PathBuf {
    match name {
        "runtime" => crate::packages::agent::data(paths),
        _ => paths.root.join(name),
    }
}
fn digest(path: &Path) -> Result<Entry> {
    util::reject_symlink_components(path)?;
    let meta = fs::symlink_metadata(path)?;
    ensure!(
        meta.is_file() && meta.len() <= MAX_BYTES,
        "backup requires bounded regular files"
    );
    let mut file = fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut bytes = 0;
    let mut buffer = [0; 64 * 1024];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        bytes += n as u64;
        ensure!(bytes <= MAX_BYTES, "backup file grew beyond limit");
        hash.update(&buffer[..n]);
    }
    ensure!(bytes == meta.len(), "file changed while reading backup");
    Ok(Entry {
        bytes,
        sha256: format!("{:x}", hash.finalize()),
    })
}
fn inventory(base: &Path, name: &str, files: &mut BTreeMap<String, Entry>) -> Result<()> {
    let mut bytes = files.values().map(|f| f.bytes).sum();
    inventory_inner(base, name, files, &mut bytes)
}
fn inventory_inner(
    base: &Path,
    name: &str,
    files: &mut BTreeMap<String, Entry>,
    bytes: &mut u64,
) -> Result<()> {
    let path = base.join(name);
    util::reject_symlink_components(&path)?;
    let meta = fs::symlink_metadata(&path)?;
    if meta.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let child = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("backup paths must be UTF-8"))?;
            inventory_inner(base, &format!("{name}/{child}"), files, bytes)?;
        }
    } else {
        ensure!(meta.is_file(), "backup refuses links and special files");
        ensure!(files.len() < MAX_FILES, "backup exceeds file limit");
        let entry = digest(&path)?;
        *bytes = bytes
            .checked_add(entry.bytes)
            .context("backup size overflow")?;
        ensure!(*bytes <= MAX_BYTES, "backup exceeds 2 GiB");
        files.insert(name.into(), entry);
    }
    Ok(())
}
fn copy_tree(source: &Path, target: &Path) -> Result<()> {
    util::reject_symlink_components(source)?;
    let meta = fs::symlink_metadata(source)?;
    if meta.is_dir() {
        private_directory(target)?;
        for child in fs::read_dir(source)? {
            let child = child?;
            copy_tree(&child.path(), &target.join(child.file_name()))?;
        }
    } else {
        ensure!(meta.is_file(), "backup refuses links and special files");
        ensure!(meta.len() <= MAX_BYTES, "backup file exceeds limit");
        let mut input = fs::File::open(source)?;
        let mut output = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(target)?;
        private_file(target)?;
        let bytes = std::io::copy(&mut (&mut input).take(MAX_BYTES + 1), &mut output)?;
        ensure!(bytes == meta.len(), "backup source changed");
        output.sync_all()?;
    }
    Ok(())
}
fn private_directory(path: &Path) -> Result<()> {
    util::ensure_directory(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}
fn private_file(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}
fn sync_dir(path: &Path) -> Result<()> {
    #[cfg(unix)]
    fs::File::open(path)?.sync_all()?;
    Ok(())
}

fn sync_tree(path: &Path) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            sync_tree(&entry.path())?;
        }
    }
    sync_dir(path)
}

pub fn create(paths: &Paths, output: &Path) -> Result<Value> {
    ensure!(output.is_absolute(), "backup output must be absolute");
    util::reject_symlink_components(output)?;
    ensure!(!output.exists(), "backup destination already exists");
    // Complete state migrations before taking the exclusive lease.
    crate::state::State::open(paths)?;
    let _lease = maintenance::exclusive(paths)?;
    ensure!(
        !paths.root.join("restore-pending.json").exists(),
        "restore is incomplete"
    );
    let parent = output.parent().context("backup parent")?;
    for name in ROOTS {
        ensure!(
            !output.starts_with(destination(paths, name)),
            "backup cannot be written inside durable data"
        );
    }
    let stage = tempfile::Builder::new()
        .prefix("backup-")
        .tempdir_in(&paths.root)?;
    let mut manifest = Manifest {
        format: 1,
        home: paths.home.clone(),
        version: env!("CARGO_PKG_VERSION").into(),
        source_sha256: env!("MIRKET_SOURCE_SHA256").into(),
        runtime_recipe: crate::packages::agent::recipe(),
        roots: vec!["state.db".into()],
        files: BTreeMap::new(),
    };
    // SQLite's own snapshot includes committed WAL pages; never copy a live DB file.
    util::reject_symlink_components(&paths.database())?;
    let conn = rusqlite::Connection::open(paths.database())?;
    conn.execute(
        "VACUUM INTO ?1",
        [stage
            .path()
            .join("state.db")
            .to_str()
            .context("UTF-8 backup path")?],
    )?;
    drop(conn);
    // VACUUM INTO normalizes to DELETE mode; restore must retain the owned WAL contract.
    let snapshot = rusqlite::Connection::open(stage.path().join("state.db"))?;
    snapshot.pragma_update(None, "journal_mode", "WAL")?;
    drop(snapshot);
    private_file(&stage.path().join("state.db"))?;
    for name in ["runs", "runtime"] {
        let source = destination(paths, name);
        util::reject_symlink_components(&source)?;
        if source.exists() {
            let mut check = BTreeMap::new();
            inventory(
                source.parent().context("state parent")?,
                source
                    .file_name()
                    .and_then(|s| s.to_str())
                    .context("state name")?,
                &mut check,
            )?;
            copy_tree(&source, &stage.path().join(name))?;
            manifest.roots.push(name.into());
        }
    }
    for name in &manifest.roots {
        inventory(stage.path(), name, &mut manifest.files)?;
    }
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    {
        let mut tar = tar::Builder::new(temporary.as_file_mut());
        let bytes = serde_json::to_vec(&manifest)?;
        ensure!(
            bytes.len() as u64 <= MANIFEST_LIMIT,
            "backup manifest exceeds bound"
        );
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o600);
        header.set_cksum();
        tar.append_data(&mut header, "manifest.json", bytes.as_slice())?;
        for name in manifest.files.keys() {
            tar.append_path_with_name(stage.path().join(name), name)?;
        }
        tar.finish()?;
    }
    temporary.as_file().sync_all()?;
    let hash = archive_digest(temporary.path())?;
    temporary
        .persist_noclobber(output)
        .context("publish backup without overwriting")?;
    sync_dir(parent)?;
    Ok(
        json!({"created":true,"path":output,"sha256":hash,"files":manifest.files.len(),"bytes":manifest.files.values().map(|f| f.bytes).sum::<u64>(),"coverage":"Task database, runtime facts/history/all checkpoints, and run traces. Project and design files, installation, credentials and host settings are excluded."}),
    )
}

fn archive_digest(path: &Path) -> Result<String> {
    util::reject_symlink_components(path)?;
    let meta = fs::metadata(path)?;
    ensure!(
        meta.is_file() && meta.len() <= MAX_BYTES + 64 * 1024 * 1024,
        "backup archive exceeds bound"
    );
    let mut hash = Sha256::new();
    let mut file = fs::File::open(path)?;
    let mut buffer = [0; 64 * 1024];
    let mut total = 0;
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        total += n as u64;
        ensure!(
            total <= MAX_BYTES + 64 * 1024 * 1024,
            "archive grew beyond limit"
        );
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn unpack(archive: &Path, stage: &Path, expected: &str) -> Result<Manifest> {
    ensure!(
        expected.len() == 64 && expected.bytes().all(|b| b.is_ascii_hexdigit()),
        "expected SHA-256 required"
    );
    ensure!(
        archive_digest(archive)? == expected,
        "backup archive SHA-256 mismatch"
    );
    let mut tar = tar::Archive::new(fs::File::open(archive)?);
    let mut entries = tar.entries()?;
    let mut first = entries.next().context("missing backup manifest")??;
    ensure!(
        first.header().entry_type().is_file()
            && first.path()?.as_ref() == Path::new("manifest.json"),
        "backup manifest must be first"
    );
    ensure!(
        first.size() <= MANIFEST_LIMIT,
        "backup manifest exceeds bound"
    );
    let manifest: Manifest = serde_json::from_reader(&mut first)?;
    ensure!(
        manifest.format == 1 && manifest.files.len() <= MAX_FILES,
        "unsupported backup format or size"
    );
    ensure!(
        !manifest.roots.is_empty() && manifest.roots.contains(&"state.db".into()),
        "backup database missing"
    );
    let mut roots = std::collections::BTreeSet::new();
    for name in &manifest.roots {
        ensure!(
            ROOTS.contains(&name.as_str()) && roots.insert(name),
            "unknown or duplicate backup root"
        );
        if name != "state.db" {
            private_directory(&stage.join(name))?;
        }
    }
    let mut seen = BTreeMap::new();
    let mut total: u64 = 0;
    for entry in entries {
        let mut entry = entry?;
        ensure!(
            entry.header().entry_type().is_file(),
            "backup refuses links and non-files"
        );
        let path = entry.path()?.into_owned();
        util::safe_relative(&path)?;
        let name = path.to_str().context("UTF-8 archive path required")?;
        let root = name.split('/').next().context("archive root")?;
        ensure!(
            roots.contains(&root.to_string()) && (root != "state.db" || name == "state.db"),
            "unknown backup path"
        );
        let expected = manifest
            .files
            .get(name)
            .context("undeclared backup entry")?;
        ensure!(
            !seen.contains_key(name) && entry.size() == expected.bytes,
            "duplicate or wrong-size backup entry"
        );
        total = total
            .checked_add(entry.size())
            .context("backup size overflow")?;
        ensure!(total <= MAX_BYTES, "backup exceeds 2 GiB");
        let target = stage.join(&path);
        private_directory(target.parent().context("archive parent")?)?;
        let mut output = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&target)?;
        private_file(&target)?;
        std::io::copy(&mut entry, &mut output)?;
        output.sync_all()?;
        let actual = digest(&target)?;
        ensure!(&actual == expected, "backup entry digest mismatch");
        seen.insert(name.to_string(), actual);
    }
    ensure!(seen == manifest.files, "incomplete backup archive");
    // Check the copy, never execute restored content or trust a healthy tar as a database check.
    crate::state::State::validate_backup(&stage.join("state.db"))?;
    ensure!(
        archive_digest(archive)? == expected,
        "backup changed while validating"
    );
    Ok(manifest)
}

pub fn inspect(paths: &Paths, archive: &Path, expected: &str) -> Result<Value> {
    paths.ensure()?;
    let stage = tempfile::tempdir_in(&paths.root)?;
    let manifest = unpack(archive, stage.path(), expected)?;
    Ok(
        json!({"valid":true,"manifest":manifest,"scope":"Integrity and format checks; no restore or execution."}),
    )
}

pub fn restore(paths: &Paths, archive: &Path, expected: &str) -> Result<Value> {
    let _lease = maintenance::exclusive(paths)?;
    let journal = paths.root.join("restore-pending.json");
    let pending = if journal.exists() {
        let pending: Pending =
            serde_json::from_slice(&util::read_bounded(&journal, MANIFEST_LIMIT)?)?;
        ensure!(
            pending.sha256 == expected,
            "another restore is incomplete; repeat its exact archive and SHA-256"
        );
        ensure!(
            archive_digest(archive)? == expected,
            "backup archive SHA-256 mismatch"
        );
        util::safe_relative(Path::new(&pending.stage))?;
        ensure!(
            Path::new(&pending.stage).components().count() == 1
                && pending.stage.starts_with("restore-"),
            "invalid restore stage"
        );
        let validation = tempfile::tempdir_in(&paths.root)?;
        ensure!(
            unpack(archive, validation.path(), expected)? == pending.manifest,
            "restore journal differs from archive"
        );
        ensure!(
            pending.manifest.home == paths.home
                && pending.manifest.runtime_recipe == crate::packages::agent::recipe(),
            "restore binding changed"
        );
        pending
    } else {
        let stage = tempfile::Builder::new()
            .prefix("restore-")
            .tempdir_in(&paths.root)?;
        let manifest = unpack(archive, stage.path(), expected)?;
        ensure!(
            manifest.home == paths.home,
            "backup belongs to a different home; absolute worker bindings cannot be remapped"
        );
        ensure!(
            manifest.runtime_recipe == crate::packages::agent::recipe(),
            "backup runtime recipe differs; use its matching runtime before recovery"
        );
        for name in ROOTS {
            let dest = destination(paths, name);
            util::reject_symlink_components(&dest)?;
            ensure!(
                !dest.exists() || (dest.is_dir() && fs::read_dir(&dest)?.next().is_none()),
                "restore requires empty durable destinations; existing data is never overwritten"
            );
        }
        for suffix in ["state.db-wal", "state.db-shm"] {
            ensure!(
                !paths.root.join(suffix).exists(),
                "existing SQLite sidecar; restore refused"
            );
        }
        sync_tree(stage.path())?;
        let stage = stage.keep();
        let pending = Pending {
            sha256: expected.into(),
            stage: stage
                .file_name()
                .and_then(|s| s.to_str())
                .context("restore stage name")?
                .into(),
            manifest,
        };
        util::atomic_write(&journal, &serde_json::to_vec(&pending)?)?;
        sync_dir(&paths.root)?;
        pending
    };
    let stage = paths.root.join(&pending.stage);
    // Validate every staged or already-published root before continuing a crash recovery.
    for name in &pending.manifest.roots {
        ensure!(ROOTS.contains(&name.as_str()), "invalid restore root");
        let source = stage.join(name);
        let dest = destination(paths, name);
        let observed = if source.exists() { &source } else { &dest };
        let mut actual = BTreeMap::new();
        inventory(
            observed.parent().context("restore parent")?,
            observed
                .file_name()
                .and_then(|s| s.to_str())
                .context("restore name")?,
            &mut actual,
        )?;
        // Runtime destination basename is data, while the archive root is runtime.
        let basename = observed.file_name().unwrap().to_str().unwrap();
        let actual: BTreeMap<_, _> = actual
            .into_iter()
            .map(|(p, v)| (format!("{name}{}", &p[basename.len()..]), v))
            .collect();
        let expected: BTreeMap<_, _> = pending
            .manifest
            .files
            .iter()
            .filter(|(p, _)| *p == name || p.starts_with(&format!("{name}/")))
            .map(|(p, v)| (p.clone(), v.clone()))
            .collect();
        ensure!(
            actual == expected,
            "restore staging or published data changed; retained for inspection"
        );
    }
    for name in &pending.manifest.roots {
        let source = stage.join(name);
        let dest = destination(paths, name);
        if source.exists() {
            util::reject_symlink_components(&dest)?;
            if dest.exists() {
                ensure!(
                    dest.is_dir() && fs::read_dir(&dest)?.next().is_none(),
                    "restore destination became occupied"
                );
                fs::remove_dir(&dest)?;
            }
            util::ensure_directory(dest.parent().context("restore parent")?)?;
            fs::rename(&source, &dest)?;
            sync_dir(dest.parent().unwrap())?;
        }
    }
    fs::remove_file(&journal)?;
    sync_dir(&paths.root)?;
    if stage.exists() {
        fs::remove_dir_all(stage)?;
    }
    Ok(
        json!({"restored":true,"sha256":expected,"files":pending.manifest.files.len(),"assurance":"Durable records restored. Restore project/design files separately and recheck evidence. Historical source/path/deadline bindings still govern worker resumption."}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (tempfile::TempDir, Paths, PathBuf, String) {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
        crate::state::State::open(&paths).unwrap();
        let run = paths.root.join("runs/routed-history");
        fs::create_dir_all(&run).unwrap();
        fs::write(run.join("trace.json"), b"historical trace").unwrap();
        let archive = paths.home.join("backup.tar");
        let hash = create(&paths, &archive).unwrap()["sha256"]
            .as_str()
            .unwrap()
            .to_owned();
        (home, paths, archive, hash)
    }
    fn interrupted(paths: &Paths, archive: &Path, hash: &str) {
        let stage = tempfile::Builder::new()
            .prefix("restore-")
            .tempdir_in(&paths.root)
            .unwrap();
        let manifest = unpack(archive, stage.path(), hash).unwrap();
        let stage = stage.keep();
        for name in ROOTS {
            let p = destination(paths, name);
            if p.is_dir() {
                fs::remove_dir_all(p).unwrap();
            } else if p.exists() {
                fs::remove_file(p).unwrap();
            }
        }
        let pending = Pending {
            sha256: hash.into(),
            stage: stage.file_name().unwrap().to_str().unwrap().into(),
            manifest,
        };
        util::atomic_write(
            &paths.root.join("restore-pending.json"),
            &serde_json::to_vec(&pending).unwrap(),
        )
        .unwrap();
        fs::rename(stage.join("state.db"), paths.database()).unwrap();
    }
    #[test]
    fn interrupted_restore_blocks_calls_then_reconciles_published_root() {
        let (_home, paths, archive, hash) = fixture();
        interrupted(&paths, &archive, &hash);
        assert!(crate::state::State::open(&paths).is_err());
        assert!(restore(&paths, &archive, &"0".repeat(64)).is_err());
        restore(&paths, &archive, &hash).unwrap();
        assert!(!paths.root.join("restore-pending.json").exists());
        crate::state::State::open(&paths).unwrap();
        assert_eq!(
            fs::read(paths.root.join("runs/routed-history/trace.json")).unwrap(),
            b"historical trace"
        );
    }
    #[test]
    fn interrupted_restore_preserves_unexpected_destination_changes() {
        let (_home, paths, archive, hash) = fixture();
        interrupted(&paths, &archive, &hash);
        fs::write(paths.database(), b"external edit").unwrap();
        assert!(
            restore(&paths, &archive, &hash)
                .unwrap_err()
                .to_string()
                .contains("changed")
        );
        assert_eq!(fs::read(paths.database()).unwrap(), b"external edit");
        assert!(paths.root.join("restore-pending.json").exists());
    }
    #[test]
    fn backup_rejects_links_duplicates_missing_and_modified_entries_even_with_new_archive_hash() {
        let (_home, paths, archive, hash) = fixture();
        let stage = tempfile::tempdir_in(&paths.root).unwrap();
        let manifest = unpack(&archive, stage.path(), &hash).unwrap();
        for mode in ["link", "duplicate", "missing", "modified", "undeclared"] {
            let bad = paths.home.join(format!("{mode}.tar"));
            let mut tar = tar::Builder::new(fs::File::create(&bad).unwrap());
            let bytes = serde_json::to_vec(&manifest).unwrap();
            let mut header = tar::Header::new_gnu();
            header.set_mode(0o600);
            header.set_size(bytes.len() as u64);
            header.set_cksum();
            tar.append_data(&mut header, "manifest.json", bytes.as_slice())
                .unwrap();
            if mode != "missing" {
                for name in manifest.files.keys() {
                    tar.append_path_with_name(stage.path().join(name), name)
                        .unwrap();
                }
            }
            match mode {
                "link" => {
                    let mut h = tar::Header::new_gnu();
                    h.set_entry_type(tar::EntryType::Symlink);
                    h.set_size(0);
                    h.set_mode(0o600);
                    h.set_link_name("/tmp/escape").unwrap();
                    h.set_cksum();
                    tar.append_data(&mut h, "runs/link", std::io::empty())
                        .unwrap();
                }
                "duplicate" => {
                    tar.append_path_with_name(stage.path().join("state.db"), "state.db")
                        .unwrap();
                }
                "undeclared" => {
                    tar.append_path_with_name(stage.path().join("state.db"), "unknown.db")
                        .unwrap();
                }
                _ => {}
            }
            tar.finish().unwrap();
            drop(tar);
            if mode == "modified" {
                let mut bytes = fs::read(&bad).unwrap();
                let index = bytes
                    .windows(b"historical trace".len())
                    .position(|v| v == b"historical trace")
                    .unwrap();
                bytes[index] = b'X';
                fs::write(&bad, bytes).unwrap();
            }
            let digest = archive_digest(&bad).unwrap();
            assert!(inspect(&paths, &bad, &digest).is_err(), "{mode}");
        }
    }
}
