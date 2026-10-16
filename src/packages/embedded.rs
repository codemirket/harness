//! Reviewed first-party package files, carried by the CLI into immutable generations.
use super::{Member, archive};
use crate::util;
use anyhow::{Context, Result, bail};
use include_dir::{Dir, include_dir};
use std::{collections::BTreeSet, fs, io::Write, path::Path};

static PAYLOAD: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/runtime/packages");
const MAX_FILES: usize = 64;
const MAX_FILE_BYTES: usize = 2 * 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 8 * 1024 * 1024;

fn content<'a>(payload: &'a Dir<'a>, name: &str) -> Result<&'a [u8]> {
    payload
        .get_file(name)
        .map(|file| file.contents())
        .with_context(|| format!("missing embedded package source {name}"))
}

pub(super) fn validate(members: &[Member]) -> Result<()> {
    validate_from(members, &PAYLOAD)
}

fn validate_from(members: &[Member], payload: &Dir<'_>) -> Result<()> {
    if members.len() > MAX_FILES {
        bail!("embedded package payload exceeds {MAX_FILES} files");
    }
    let mut sources = BTreeSet::new();
    let mut destinations = BTreeSet::new();
    let mut total = 0usize;
    for member in members {
        archive::normal_path(&member.path)?;
        archive::normal_path(&member.destination)?;
        let destination = member.destination.to_lowercase();
        if !sources.insert(&member.path)
            || !destinations.insert(destination.clone())
            || destination.split('/').next() == Some(archive::RECEIPT)
        {
            bail!("duplicate or reserved embedded package mapping");
        }
        let bytes = content(payload, &member.path)?;
        if bytes.is_empty() || bytes.len() > MAX_FILE_BYTES {
            bail!("embedded package source must be nonempty and at most 2 MiB");
        }
        total = total
            .checked_add(bytes.len())
            .context("embedded size overflow")?;
        if total > MAX_TOTAL_BYTES {
            bail!("embedded package payload exceeds 8 MiB");
        }
    }
    for destination in &destinations {
        let mut prefix = destination.as_str();
        while let Some((parent, _)) = prefix.rsplit_once('/') {
            if destinations.contains(parent) {
                bail!("embedded package output is also another output's parent");
            }
            prefix = parent;
        }
    }
    Ok(())
}

pub(super) fn append_identity(identity: &mut Vec<u8>, members: &[Member]) -> Result<()> {
    append_identity_from(identity, members, &PAYLOAD)
}

fn append_identity_from(
    identity: &mut Vec<u8>,
    members: &[Member],
    payload: &Dir<'_>,
) -> Result<()> {
    validate_from(members, payload)?;
    for member in members {
        let bytes = content(payload, &member.path)?;
        // Length-framed metadata and content hashes ensure changing a runner,
        // its destination or executable mode selects a new generation.
        let entry = serde_json::to_vec(&(member, bytes.len(), util::sha256(bytes)))?;
        identity.extend_from_slice(&(entry.len() as u64).to_le_bytes());
        identity.extend_from_slice(&entry);
    }
    Ok(())
}

fn preflight(root: &Path, destination: &str) -> Result<()> {
    util::reject_symlink_components(root)?;
    let mut parent = root.to_path_buf();
    let components = destination.split('/').collect::<Vec<_>>();
    for (index, component) in components.iter().enumerate() {
        if !parent.exists() {
            break;
        }
        if !parent.is_dir() {
            bail!("embedded package parent is not a directory");
        }
        let folded = component.to_lowercase();
        for entry in fs::read_dir(&parent)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_str().context("non-UTF8 staged package path")?;
            if name.to_lowercase() == folded {
                if name != *component || index + 1 == components.len() {
                    bail!("embedded package output collides with staged payload: {destination}");
                }
                if !entry.file_type()?.is_dir() {
                    bail!("embedded package parent is not a regular directory");
                }
            }
        }
        parent.push(component);
    }
    util::reject_symlink_components(&root.join(destination))?;
    Ok(())
}

pub(super) fn stage(root: &Path, members: &[Member]) -> Result<()> {
    stage_from(root, members, &PAYLOAD)
}

fn stage_from(root: &Path, members: &[Member], payload: &Dir<'_>) -> Result<()> {
    validate_from(members, payload)?;
    // Validate every destination before adding any first-party file. The caller
    // owns a private staging directory and discards it on any failure.
    for member in members {
        preflight(root, &member.destination)?;
    }
    for member in members {
        let path = root.join(&member.destination);
        fs::create_dir_all(path.parent().context("embedded output parent")?)?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        file.write_all(content(payload, &member.path)?)?;
        file.sync_all()?;
        util::set_executable_mode(&path, member.executable)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use include_dir::{DirEntry, File};

    static FIXTURE: Dir<'_> = Dir::new(
        "",
        &[
            DirEntry::File(File::new("runner.mjs", b"export const version = 1;\n")),
            DirEntry::File(File::new("helper.mjs", b"export const helper = true;\n")),
        ],
    );
    static REVISED: Dir<'_> = Dir::new(
        "",
        &[DirEntry::File(File::new(
            "runner.mjs",
            b"export const version = 2;\n",
        ))],
    );

    fn mapping(source: &str, destination: &str) -> Member {
        Member {
            path: source.into(),
            destination: destination.into(),
            executable: false,
        }
    }

    #[test]
    fn embedded_sources_and_portable_destinations_fail_closed() {
        for (source, destination) in [
            ("missing.mjs", "runner.mjs"),
            ("../runner.mjs", "runner.mjs"),
            ("runner.mjs", "../runner.mjs"),
            ("runner.mjs", "/runner.mjs"),
            ("runner.mjs", "a\\runner.mjs"),
            ("runner.mjs", ".MIRKET-PACKAGE.JSON"),
            ("runner.mjs", ".mirket-package.json/nested"),
        ] {
            assert!(validate_from(&[mapping(source, destination)], &FIXTURE).is_err());
        }
        for second in ["run.mjs", "RUN.mjs", "run.mjs/child"] {
            assert!(
                validate_from(
                    &[
                        mapping("runner.mjs", "run.mjs"),
                        mapping("helper.mjs", second)
                    ],
                    &FIXTURE
                )
                .is_err()
            );
        }
        assert!(
            validate_from(
                &vec![mapping("runner.mjs", "run.mjs"); MAX_FILES + 1],
                &FIXTURE
            )
            .is_err()
        );
    }

    #[test]
    fn embedded_bytes_destinations_and_modes_change_identity() {
        let member = mapping("runner.mjs", "mirket/runner.mjs");
        let identity = |members: &[Member], payload| {
            let mut result = Vec::new();
            append_identity_from(&mut result, members, payload).unwrap();
            util::sha256(&result)
        };
        let baseline = identity(std::slice::from_ref(&member), &FIXTURE);
        assert_ne!(baseline, identity(std::slice::from_ref(&member), &REVISED));
        let mut moved = member.clone();
        moved.destination = "mirket/other.mjs".into();
        assert_ne!(baseline, identity(&[moved], &FIXTURE));
        let mut executable = member;
        executable.executable = true;
        assert_ne!(baseline, identity(&[executable], &FIXTURE));
        let mut empty = b"unchanged".to_vec();
        append_identity_from(&mut empty, &[], &FIXTURE).unwrap();
        assert_eq!(empty, b"unchanged");
    }

    #[test]
    fn embedded_payload_byte_limits_reject_empty_oversized_and_excessive_content() {
        for bytes in [Vec::new(), vec![b'x'; MAX_FILE_BYTES + 1]] {
            let entries = [DirEntry::File(File::new("runner.mjs", &bytes))];
            let payload = Dir::new("", &entries);
            assert!(validate_from(&[mapping("runner.mjs", "runner.mjs")], &payload).is_err());
        }
        let bytes = vec![b'x'; MAX_FILE_BYTES];
        let names = ["one.mjs", "two.mjs", "three.mjs", "four.mjs", "five.mjs"];
        let entries = names.map(|name| DirEntry::File(File::new(name, &bytes)));
        let payload = Dir::new("", &entries);
        let members = names.map(|name| mapping(name, name));
        assert!(validate_from(&members[..4], &payload).is_ok());
        assert!(validate_from(&members, &payload).is_err());
    }

    #[test]
    fn embedded_staging_preserves_existing_files_and_refuses_case_aliases() {
        let root = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let members = [
            mapping("runner.mjs", "mirket/runner.mjs"),
            mapping("helper.mjs", "mirket/helper.mjs"),
        ];
        fs::create_dir(root.path().join("mirket")).unwrap();
        fs::write(root.path().join("mirket/helper.mjs"), b"upstream owned").unwrap();
        assert!(stage_from(root.path(), &members, &FIXTURE).is_err());
        assert!(!root.path().join("mirket/runner.mjs").exists());
        assert_eq!(
            fs::read(root.path().join("mirket/helper.mjs")).unwrap(),
            b"upstream owned"
        );
        fs::remove_file(root.path().join("mirket/helper.mjs")).unwrap();
        stage_from(root.path(), &members, &FIXTURE).unwrap();
        assert_eq!(
            fs::read(root.path().join("mirket/runner.mjs")).unwrap(),
            content(&FIXTURE, "runner.mjs").unwrap()
        );
        assert!(
            stage_from(
                root.path(),
                &[mapping("helper.mjs", "MIRKET/new.mjs")],
                &FIXTURE
            )
            .is_err()
        );
        assert!(!root.path().join("mirket/new.mjs").exists());
    }

    #[cfg(unix)]
    #[test]
    fn embedded_staging_refuses_symlink_parents() {
        let root = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let outside = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("link")).unwrap();
        assert!(
            stage_from(
                root.path(),
                &[mapping("runner.mjs", "link/runner.mjs")],
                &FIXTURE
            )
            .is_err()
        );
        assert!(!outside.path().join("runner.mjs").exists());
    }
}
