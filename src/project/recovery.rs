//! Durable directory publication. This owns only its journal and validated skill trees;
//! historical anonymous temp directories are reported, never swept.
use super::*;

const TRANSACTION: &str = ".mirket/project-sync-transaction";
const COMMITTED: &[u8] = b"mirket project sync committed\n";

pub(super) struct Publication {
    pub path: PathBuf,
    pub before: Option<Receipt>,
    pub payload: Option<(String, Payload)>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    format: u32,
    entries: Vec<Entry>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    path: String,
    before: Option<Receipt>,
    after: Option<Receipt>,
}

fn optional_receipt(path: &Path) -> Result<Option<Receipt>> {
    reject_symlink_components(path)?;
    if path.try_exists()? {
        Ok(Some(inspect_receipt(path)?))
    } else {
        Ok(None)
    }
}
fn matches(path: &Path, expected: &Option<Receipt>) -> Result<()> {
    ensure!(
        optional_receipt(path)? == *expected,
        "project recovery conflicts with a changed skill at {}; changes and journal preserved",
        path.display()
    );
    Ok(())
}
fn stage(directory: &Path, index: usize, name: &str) -> PathBuf {
    directory.join(index.to_string()).join(name)
}
fn validate_entry(entry: &Entry) -> Result<()> {
    crate::util::safe_relative(Path::new(&entry.path))?;
    let parts: Vec<_> = entry.path.split('/').collect();
    ensure!(
        parts.len() == 3 && [".agents", ".claude"].contains(&parts[0]) && parts[1] == "skills",
        "project recovery journal claims an unmanaged path"
    );
    safe_path(parts[2])?;
    for receipt in [&entry.before, &entry.after].into_iter().flatten() {
        crate::catalog::validate_id(&receipt.skill)?;
        ensure!(
            receipt.schema_version == 1
                && receipt.sha256.len() == 64
                && receipt.sha256.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid recovery receipt"
        );
    }
    ensure!(
        entry.before.is_some() || entry.after.is_some(),
        "empty recovery operation"
    );
    Ok(())
}

// Validate *all* retained trees before removing any of them. Unexpected entries,
// drift and partial cleanup are retained for inspection rather than guessed away.
fn validate_staging(directory: &Path, journal: &Journal) -> Result<()> {
    for item in fs::read_dir(directory)? {
        let item = item?;
        let name = item.file_name().to_string_lossy().into_owned();
        if name == "journal.json" || name == "committed" {
            ensure!(
                item.file_type()?.is_file(),
                "unexpected project recovery entry"
            );
            continue;
        }
        let index: usize = name
            .parse()
            .context("unknown project recovery entry; preserved")?;
        ensure!(
            index < journal.entries.len()
                && name == index.to_string()
                && item.file_type()?.is_dir(),
            "unknown project recovery directory; preserved"
        );
        let entry = &journal.entries[index];
        for tree in fs::read_dir(item.path())? {
            let tree = tree?;
            let expected = match tree.file_name().to_str() {
                Some("before") => &entry.before,
                Some("next") => &entry.after,
                _ => bail!("unknown project recovery tree; preserved"),
            };
            ensure!(
                tree.file_type()?.is_dir() && expected.is_some(),
                "unexpected project recovery tree; preserved"
            );
            matches(&tree.path(), expected)?;
        }
    }
    Ok(())
}
fn cleanup(directory: &Path, journal: &Journal) -> Result<()> {
    validate_staging(directory, journal)?;
    // Remove images first, retain metadata until all image cleanup succeeds.
    for index in 0..journal.entries.len() {
        let path = directory.join(index.to_string());
        if path.try_exists()? {
            fs::remove_dir_all(path)?;
        }
    }
    // A crash during image removal is safely diagnosed as partial cleanup.
    // Once images are gone, a retained committed marker is sufficient ownership.
    fs::remove_file(directory.join("journal.json"))?;
    if directory.join("committed").try_exists()? {
        fs::remove_file(directory.join("committed"))?;
    }
    fs::remove_dir(directory)?;
    Ok(())
}

pub(super) fn recover(root: &Path) -> Result<()> {
    let directory = root.join(TRANSACTION);
    reject_symlink_components(&directory)?;
    if !directory.try_exists()? {
        return Ok(());
    }
    let journal_path = directory.join("journal.json");
    if !journal_path.try_exists()? {
        // No destination was changed before the complete journal existed. A
        // completed cleanup may have removed it already; only empty/marker-only
        // directories are unambiguous. Other interrupted preparation is retained.
        let names: Vec<_> = fs::read_dir(&directory)?.collect::<std::io::Result<Vec<_>>>()?;
        if names.is_empty() {
            fs::remove_dir(&directory)?;
            return Ok(());
        }
        if names.len() == 1
            && names[0].file_name() == "committed"
            && read_bounded(&directory.join("committed"), 64)? == COMMITTED
        {
            fs::remove_file(directory.join("committed"))?;
            fs::remove_dir(directory)?;
            return Ok(());
        }
        bail!(
            "incomplete project sync preparation at {}; no destination was published; preserve and inspect these files before retry",
            directory.display()
        );
    }
    let journal: Journal = serde_json::from_slice(&read_bounded(&journal_path, 4 * 1024 * 1024)?)?;
    ensure!(
        journal.format == 1 && journal.entries.len() <= 2048,
        "invalid project recovery journal"
    );
    let mut seen = BTreeSet::new();
    for entry in &journal.entries {
        validate_entry(entry)?;
        ensure!(seen.insert(&entry.path), "duplicate recovery destination");
    }
    validate_staging(&directory, &journal)?;
    if directory.join("committed").try_exists()? {
        ensure!(
            read_bounded(&directory.join("committed"), 64)? == COMMITTED,
            "invalid project commit marker"
        );
        return cleanup(&directory, &journal);
    }
    // Preflight every destination before rolling back any. Then repeat each
    // comparison at its mutation boundary to preserve intervening user edits.
    for (index, entry) in journal.entries.iter().enumerate() {
        validate_current(root, &directory, index, entry)?;
    }
    for (index, entry) in journal.entries.iter().enumerate().rev() {
        validate_current(root, &directory, index, entry)?;
        let path = root.join(&entry.path);
        let backup = stage(&directory, index, "before");
        if backup.try_exists()? {
            if path.try_exists()? {
                withdraw(&path, &directory, index, entry)?;
                #[cfg(test)]
                pause_at_interruption(root, "rollback")?;
            }
            matches(&backup, &entry.before)?;
            fs::rename(&backup, &path)?;
        } else if entry.before.is_none() && path.try_exists()? {
            withdraw(&path, &directory, index, entry)?;
        }
    }
    // Restored destinations are now independent of the journal. Mark cleanup so
    // a subsequent restart does not require vanished before-images.
    atomic_write(&directory.join("committed"), COMMITTED)?;
    cleanup(&directory, &journal)
}
// Reverse publication using renames too: a crash cannot leave a half-deleted
// visible copy that obscures an otherwise intact before-image.
fn withdraw(path: &Path, directory: &Path, index: usize, entry: &Entry) -> Result<()> {
    matches(path, &entry.after)?;
    let next = stage(directory, index, "next");
    ensure!(
        !next.try_exists()?,
        "recovery staging already occupied; preserved"
    );
    fs::rename(path, next)?;
    Ok(())
}

fn validate_current(root: &Path, directory: &Path, index: usize, entry: &Entry) -> Result<()> {
    let current = optional_receipt(&root.join(&entry.path))?;
    let backup = optional_receipt(&stage(directory, index, "before"))?;
    if backup.is_some() {
        ensure!(
            backup == entry.before && (current.is_none() || current == entry.after),
            "project recovery conflict; destination or before-image changed; preserved"
        );
    } else {
        ensure!(
            current == entry.before || (entry.before.is_none() && current == entry.after),
            "project recovery cannot restore missing before-image or changed destination; preserved"
        );
    }
    Ok(())
}

pub(super) fn publish(root: &Path, publications: Vec<Publication>) -> Result<()> {
    if publications.is_empty() {
        return Ok(());
    }
    let directory = root.join(TRANSACTION);
    reject_symlink_components(&directory)?;
    fs::create_dir(&directory).context("create project recovery directory")?;
    let mut journal = Journal {
        format: 1,
        entries: Vec::new(),
    };
    for (index, publication) in publications.iter().enumerate() {
        matches(&publication.path, &publication.before)?;
        ensure_directory(publication.path.parent().context("skill has no parent")?)?;
        let after = publication
            .payload
            .as_ref()
            .map(|(skill, payload)| Receipt {
                schema_version: 1,
                skill: skill.clone(),
                sha256: payload.sha256.clone(),
                executable_files: payload.executables.clone(),
            });
        let entry = Entry {
            path: publication
                .path
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/"),
            before: publication.before.clone(),
            after,
        };
        validate_entry(&entry)?;
        ensure_directory(&directory.join(index.to_string()))?;
        if let Some((_, payload)) = &publication.payload {
            let next = stage(&directory, index, "next");
            ensure_directory(&next)?;
            for (relative, bytes) in &payload.files {
                let path = next.join(relative);
                atomic_write(&path, bytes)?;
                set_executable(&path, payload.executables.contains(relative))?;
            }
            atomic_write(
                &next.join(RECEIPT),
                &serde_json::to_vec_pretty(entry.after.as_ref().unwrap())?,
            )?;
            matches(&next, &entry.after)?;
        }
        journal.entries.push(entry);
    }
    atomic_write(
        &directory.join("journal.json"),
        &serde_json::to_vec(&journal)?,
    )?;
    let result = (|| -> Result<()> {
        for entry in &journal.entries {
            matches(&root.join(&entry.path), &entry.before)?;
        }
        for (index, entry) in journal.entries.iter().enumerate() {
            let path = root.join(&entry.path);
            matches(&path, &entry.before)?;
            if entry.before.is_some() {
                fs::rename(&path, stage(&directory, index, "before"))?;
                #[cfg(test)]
                pause_at_interruption(root, "publication")?;
            }
            if entry.after.is_some() {
                fs::rename(stage(&directory, index, "next"), &path)?;
            }
        }
        atomic_write(&directory.join("committed"), COMMITTED)?;
        Ok(())
    })();
    if let Err(error) = result {
        return match recover(root) {
            Ok(()) => Err(error.context("project publication failed; previous copies restored")),
            Err(recovery) => {
                Err(error.context(format!("project recovery needs attention: {recovery:#}")))
            }
        };
    }
    cleanup(&directory, &journal).context("project synchronized; recovery cleanup needs attention")
}

#[cfg(test)]
fn pause_at_interruption(root: &Path, phase: &str) -> Result<()> {
    if std::env::var("MIRKET_TEST_SYNC_INTERRUPTION").as_deref() == Ok(phase) {
        fs::write(root.join("interruption-ready"), phase)?;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }
    Ok(())
}

pub(super) fn remnants(root: &Path) -> Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    let transaction = root.join(TRANSACTION);
    if transaction.try_exists()? {
        result.push(transaction);
    }
    for target in ["codex", "claude"] {
        let directory = root.join(target_skills_dir(target)?);
        reject_symlink_components(&directory)?;
        if !directory.try_exists()? {
            continue;
        }
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            if entry.file_type()?.is_dir()
                && entry.file_name().to_string_lossy().starts_with(".tmp")
                && ["before", "next"]
                    .iter()
                    .any(|name| entry.path().join(name).symlink_metadata().is_ok())
            {
                result.push(entry.path());
            }
        }
    }
    result.sort();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (tempfile::TempDir, Catalog, PathBuf) {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().canonicalize().unwrap();
        let catalog = Catalog::embedded().unwrap();
        init(&catalog, &root, &["codex".into()]).unwrap();
        sync(&catalog, &root, &root.join("cache")).unwrap();
        (temporary, catalog, root)
    }
    fn old_revision(root: &Path) -> Receipt {
        let path = root.join(".agents/skills/debugging");
        fs::write(path.join("SKILL.md"), b"previous verified revision").unwrap();
        let mut receipt = inspect_unchecked_receipt(&path);
        receipt.sha256 = payload_hash(&read_payload_directory(&path, &[RECEIPT]).unwrap());
        fs::write(path.join(RECEIPT), serde_json::to_vec(&receipt).unwrap()).unwrap();
        inspect_receipt(&path).unwrap()
    }
    fn inspect_unchecked_receipt(path: &Path) -> Receipt {
        serde_json::from_slice(&fs::read(path.join(RECEIPT)).unwrap()).unwrap()
    }
    fn pending(root: &Path) -> PathBuf {
        let path = root.join(".agents/skills/debugging");
        let receipt = inspect_receipt(&path).unwrap();
        let directory = root.join(TRANSACTION);
        ensure_directory(&directory.join("0")).unwrap();
        let journal = Journal {
            format: 1,
            entries: vec![Entry {
                path: ".agents/skills/debugging".into(),
                before: Some(receipt),
                after: None,
            }],
        };
        atomic_write(
            &directory.join("journal.json"),
            &serde_json::to_vec(&journal).unwrap(),
        )
        .unwrap();
        fs::rename(&path, stage(&directory, 0, "before")).unwrap();
        directory
    }

    #[test]
    fn interrupted_prune_recovers_and_unknown_remnants_are_reported() {
        let (_temp, catalog, root) = setup();
        let directory = pending(&root);
        assert_eq!(doctor(&catalog, &root).unwrap()["ok"], false);
        sync(&catalog, &root, &root.join("cache")).unwrap();
        assert!(!directory.exists());
        assert_eq!(doctor(&catalog, &root).unwrap()["ok"], true);
        let unknown = root.join(".agents/skills/.tmpHistorical/before");
        ensure_directory(&unknown).unwrap();
        fs::write(unknown.join("user-notes"), "preserve me").unwrap();
        let report = sync(&catalog, &root, &root.join("cache")).unwrap();
        assert_eq!(report["recovery_remnants"].as_array().unwrap().len(), 1);
        assert_eq!(doctor(&catalog, &root).unwrap()["ok"], false);
        assert_eq!(
            fs::read_to_string(unknown.join("user-notes")).unwrap(),
            "preserve me"
        );
        let next_only = root.join(".agents/skills/.tmpUnpublished/next");
        ensure_directory(&next_only).unwrap();
        assert_eq!(remnants(&root).unwrap().len(), 2);
        assert!(next_only.exists());
    }

    #[test]
    fn recovery_preserves_changed_destination_backup_and_unknown_files() {
        for damage in ["destination", "backup", "unknown"] {
            let (_temp, _catalog, root) = setup();
            let directory = pending(&root);
            let changed = match damage {
                "destination" => root.join(".agents/skills/debugging/user-note"),
                "backup" => stage(&directory, 0, "before").join("SKILL.md"),
                _ => directory.join("unexpected-note"),
            };
            ensure_directory(changed.parent().unwrap()).unwrap();
            fs::write(&changed, "user change").unwrap();
            assert!(recover(&root).is_err());
            assert_eq!(fs::read_to_string(&changed).unwrap(), "user change");
            assert!(directory.exists());
            assert!(stage(&directory, 0, "before").exists());
        }
    }

    fn published_pending(root: &Path) -> (PathBuf, Journal) {
        let directory = pending(root);
        let journal_path = directory.join("journal.json");
        let mut journal: Journal =
            serde_json::from_slice(&fs::read(&journal_path).unwrap()).unwrap();
        let path = root.join(".agents/skills/debugging");
        ensure_directory(&path).unwrap();
        fs::write(path.join("SKILL.md"), "new revision").unwrap();
        let after = Receipt {
            schema_version: 1,
            skill: "debugging".into(),
            sha256: payload_hash(&read_payload_directory(&path, &[RECEIPT]).unwrap()),
            executable_files: BTreeSet::new(),
        };
        atomic_write(&path.join(RECEIPT), &serde_json::to_vec(&after).unwrap()).unwrap();
        journal.entries[0].after = Some(after);
        atomic_write(&journal_path, &serde_json::to_vec(&journal).unwrap()).unwrap();
        (directory, journal)
    }

    #[test]
    fn rollback_can_resume_after_withdrawal_before_restoration() {
        let (_temp, _catalog, root) = setup();
        let (directory, journal) = published_pending(&root);
        let path = root.join(".agents/skills/debugging");
        // Exact interruption boundary: published tree withdrawn atomically,
        // original tree has not yet moved back into the visible destination.
        withdraw(&path, &directory, 0, &journal.entries[0]).unwrap();
        assert!(!path.exists());
        recover(&root).unwrap();
        assert_eq!(optional_receipt(&path).unwrap(), journal.entries[0].before);
        assert!(!directory.exists());
    }

    #[test]
    fn incomplete_preparation_and_partial_cleanup_are_preserved() {
        let (_temp, _catalog, root) = setup();
        let directory = root.join(TRANSACTION);
        ensure_directory(&directory).unwrap();
        fs::write(directory.join("partial-output"), "unknown completion").unwrap();
        assert!(
            recover(&root)
                .unwrap_err()
                .to_string()
                .contains("no destination was published")
        );
        assert!(root.join(".agents/skills/debugging/SKILL.md").exists());
        fs::remove_dir_all(&directory).unwrap();
        let directory = pending(&root);
        atomic_write(&directory.join("committed"), COMMITTED).unwrap();
        fs::remove_file(stage(&directory, 0, "before").join(RECEIPT)).unwrap();
        assert!(recover(&root).is_err());
        assert!(directory.exists());
    }

    #[test]
    fn committed_cleanup_preserves_post_commit_destination_edits() {
        let (_temp, _catalog, root) = setup();
        let directory = pending(&root);
        atomic_write(&directory.join("committed"), COMMITTED).unwrap();
        let note = root.join(".agents/skills/debugging/user-note");
        ensure_directory(note.parent().unwrap()).unwrap();
        fs::write(&note, "post-commit change").unwrap();
        recover(&root).unwrap();
        assert!(!directory.exists());
        assert_eq!(fs::read_to_string(note).unwrap(), "post-commit change");
    }

    #[test]
    #[ignore = "subprocess helper invoked only by actual_process_interruption_recovers"]
    fn interruption_child() {
        let root = PathBuf::from(
            std::env::var_os("MIRKET_TEST_SYNC_PROJECT").expect("test subprocess project"),
        );
        assert!(std::env::var_os("MIRKET_TEST_SYNC_INTERRUPTION").is_some());
        sync(&Catalog::embedded().unwrap(), &root, &root.join("cache")).unwrap();
    }

    #[test]
    fn actual_process_interruption_recovers() {
        use std::{
            process::{Command, Stdio},
            time::{Duration, Instant},
        };
        for phase in ["publication", "rollback"] {
            let (_temp, catalog, root) = setup();
            if phase == "publication" {
                old_revision(&root);
            } else {
                published_pending(&root);
            }
            let mut child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "project::recovery::tests::interruption_child",
                    "--ignored",
                ])
                .env("MIRKET_TEST_SYNC_PROJECT", &root)
                .env("MIRKET_TEST_SYNC_INTERRUPTION", phase)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            let started = Instant::now();
            while !root.join("interruption-ready").exists()
                && started.elapsed() < Duration::from_secs(30)
            {
                if child.try_wait().unwrap().is_some() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            let ready = root.join("interruption-ready").exists();
            let _ = child.kill();
            child.wait().unwrap();
            assert!(ready, "child must reach the {phase} interruption point");
            assert!(!root.join(".agents/skills/debugging").exists());
            assert_eq!(doctor(&catalog, &root).unwrap()["ok"], false);
            sync(&catalog, &root, &root.join("cache")).unwrap();
            assert_eq!(doctor(&catalog, &root).unwrap()["ok"], true);
            assert!(!root.join(TRANSACTION).exists());
            assert!(remnants(&root).unwrap().is_empty());
        }
    }
}
