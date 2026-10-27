//! A bounded, owned section of the Codex user's global AGENTS.md.
//!
//! The caller selects both paths explicitly. We never rewrite unrelated
//! instructions, claim an existing section without a receipt, or infer that a
//! successful file write means the Desktop client has reloaded the guidance.

use crate::file_lock::FileLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

const BEGIN: &str = "<!-- Mirket managed Codex guidance: begin -->";
const END: &str = "<!-- Mirket managed Codex guidance: end -->";
const BODY: &str = "For project memory, JS/TS navigation, design, and video operations qualified by the active release, use Mirket as the primary route. Check its runtime status and the relevant skill before acting; if the route is unavailable or fails, report the concrete limit instead of claiming the work was done. Use Codex's browser for general browsing and live web review, and Codex or project toolchains for SQL work. Mirket's private Chrome is reserved for video rendering. Codex owns model choice, native subagents, shell, editing, Git, and permissions. Delegate only useful independent deliverables with disjoint ownership and lead-owned integration. Keep creative direction, visual judgment, and motion with a capable lead and a reviewer of comparable capability when review helps; never choose a lower-capability or lower-reasoning agent for that judgment merely to save time. For visual/effect delegation, record the project root, explicitly permitted outputs, requested or inherited reviewer/model settings and one shared repair budget; check outputs against permitted destinations, and do not reset the budget for new specimens/delegates/retries. If the host denies approval, preserve sources, report it, and stop the denied action rather than bypassing it; continue independent authorized work. A successful render, hash or structural check is not visual inspection; view saved pixels or state that QA is untested. This guidance cannot enforce native spawning, model, reasoning or permission choices; record requested versus observed behavior. Follow explicit user and project instructions above this guidance.";
const MAX_GUIDANCE_BYTES: u64 = 1024 * 1024;
const MAX_RECEIPT_BYTES: u64 = 16 * 1024;

#[derive(Clone, Debug)]
pub struct CodexGuidance {
    path: PathBuf,
    receipt: PathBuf,
    lock: PathBuf,
}

#[derive(Clone, Debug, Serialize)]
pub struct GuidanceReport {
    pub path: PathBuf,
    pub status: &'static str,
    pub release_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
enum Receipt {
    Committed {
        schema_version: u8,
        path: String,
        release_id: String,
        block_sha256: String,
        inserted_prefix: String,
        anchor_sha256: String,
        created_file: bool,
    },
    Pending {
        schema_version: u8,
        path: String,
        release_id: String,
        before_present: bool,
        before_sha256: String,
        after_sha256: String,
        block_sha256: String,
        inserted_prefix: String,
        anchor_sha256: String,
        created_file: bool,
        previous: Option<Box<Receipt>>,
    },
    Removing {
        schema_version: u8,
        path: String,
        release_id: String,
        before_sha256: String,
        after_sha256: String,
        block_sha256: String,
        inserted_prefix: String,
        anchor_sha256: String,
        created_file: bool,
        after_present: bool,
    },
}

impl CodexGuidance {
    pub fn open(path: &Path, receipt: &Path) -> Result<Self, String> {
        for selected in [path, receipt] {
            if !selected.is_absolute()
                || selected
                    .components()
                    .any(|component| matches!(component, Component::ParentDir))
            {
                return Err("Codex guidance paths must be absolute without traversal".into());
            }
            let parent = selected.parent().ok_or("managed path has no parent")?;
            let meta = fs::symlink_metadata(parent).map_err(str_error)?;
            if meta.file_type().is_symlink() || !meta.is_dir() {
                return Err("managed file parent must be an existing real directory".into());
            }
        }
        Ok(Self {
            path: path.to_owned(),
            receipt: receipt.to_owned(),
            lock: receipt.with_extension("lock"),
        })
    }

    pub fn inspect(&self) -> Result<GuidanceReport, String> {
        let (present, current) = read_file(&self.path, MAX_GUIDANCE_BYTES)?;
        let receipt = read_receipt(&self.receipt)?;
        let block = find_block(&current)?;
        let (status, release_id, error) = match (receipt, block) {
            (None, None) => ("missing", None, None),
            (None, Some(_)) => (
                "conflict",
                None,
                Some("managed marker exists without an ownership receipt".into()),
            ),
            (
                Some(Receipt::Pending { release_id, .. } | Receipt::Removing { release_id, .. }),
                _,
            ) => ("pending", Some(release_id), None),
            (
                Some(Receipt::Committed {
                    path,
                    release_id,
                    block_sha256,
                    inserted_prefix,
                    anchor_sha256,
                    ..
                }),
                Some((start, end)),
            ) if path == self.path.to_string_lossy()
                && hash(&current.as_bytes()[start..end]) == block_sha256
                && owned_separator_matches(&current, start, &inserted_prefix, &anchor_sha256) =>
            {
                ("managed", Some(release_id), None)
            }
            (Some(other), _) => (
                "drifted",
                Some(other.release_id().to_owned()),
                Some("managed guidance or receipt does not match saved bytes".into()),
            ),
        };
        let _ = present;
        Ok(GuidanceReport {
            path: self.path.clone(),
            status,
            release_id,
            error,
        })
    }

    /// Check both outcomes before a lifecycle reconciles a pending edit.
    pub(crate) fn verify_receipt_release(&self, expected: &str) -> Result<(), String> {
        if let Some(receipt) = read_receipt(&self.receipt)?
            && (receipt.release_id() != expected
                || matches!(&receipt, Receipt::Pending { previous: Some(previous), .. }
                    if previous.release_id() != expected))
        {
            return Err(
                "native receipt transition belongs to another lifecycle release; preserved".into(),
            );
        }
        Ok(())
    }

    pub fn install(&self, release_id: &str) -> Result<GuidanceReport, String> {
        if release_id.is_empty() || release_id.len() > 256 {
            return Err("guidance release identity is invalid".into());
        }
        let _lock = acquire_lock(&self.lock)?;
        let (present, current) = read_file(&self.path, MAX_GUIDANCE_BYTES)?;
        let previous = read_receipt(&self.receipt)?;
        let range = find_block(&current)?;
        if matches!(
            previous,
            Some(Receipt::Pending { .. } | Receipt::Removing { .. })
        ) {
            return Err("pending guidance edit must be reconciled before another install".into());
        }
        match (&previous, range) {
            (None, None) => {}
            (None, Some(_)) => return Err("managed marker exists without a receipt".into()),
            (
                Some(Receipt::Committed {
                    path,
                    block_sha256,
                    inserted_prefix,
                    anchor_sha256,
                    ..
                }),
                Some((start, end)),
            ) if path == &self.path.to_string_lossy()
                && hash(&current.as_bytes()[start..end]) == *block_sha256
                && owned_separator_matches(&current, start, inserted_prefix, anchor_sha256) => {}
            _ => return Err("managed guidance has drifted; refusing overwrite".into()),
        }
        let (inserted_prefix, anchor_sha256, created_file) = match &previous {
            Some(Receipt::Committed {
                inserted_prefix,
                anchor_sha256,
                created_file,
                ..
            }) => (
                inserted_prefix.clone(),
                anchor_sha256.clone(),
                *created_file,
            ),
            None => (
                if current.is_empty() {
                    ""
                } else if current.ends_with('\n') {
                    "\n"
                } else {
                    "\n\n"
                }
                .to_owned(),
                hash(current.as_bytes()),
                !present,
            ),
            _ => unreachable!(),
        };
        let block = format!("{BEGIN}\n{BODY}\n{END}\n");
        let mut next = current.clone();
        if let Some((start, end)) = range {
            next.replace_range(start..end, &block);
        } else {
            next.push_str(&inserted_prefix);
            next.push_str(&block);
        }
        if next.len() as u64 > MAX_GUIDANCE_BYTES {
            return Err("Codex guidance exceeds its bounded file size".into());
        }
        if next == current
            && previous
                .as_ref()
                .is_some_and(|receipt| receipt.release_id() == release_id)
        {
            return self.inspect();
        }
        let pending = Receipt::Pending {
            schema_version: 1,
            path: self.path.to_string_lossy().into_owned(),
            release_id: release_id.into(),
            before_present: present,
            before_sha256: hash(current.as_bytes()),
            after_sha256: hash(next.as_bytes()),
            block_sha256: hash(block.as_bytes()),
            inserted_prefix,
            anchor_sha256,
            created_file,
            previous: previous.map(Box::new),
        };
        write_receipt(&self.receipt, &pending)?;
        let (now_present, now) = read_file(&self.path, MAX_GUIDANCE_BYTES)?;
        if now_present != present || now != current {
            return Err(
                "Codex guidance changed before replacement; reconcile the pending edit".into(),
            );
        }
        atomic_replace(&self.path, next.as_bytes())?;
        write_receipt(&self.receipt, &pending.committed())?;
        self.inspect()
    }

    /// Remove only an unchanged block that belongs to this installation.
    pub fn remove(&self) -> Result<GuidanceReport, String> {
        let _lock = acquire_lock(&self.lock)?;
        let receipt = read_receipt(&self.receipt)?;
        let Some(Receipt::Committed {
            path,
            release_id,
            block_sha256,
            inserted_prefix,
            anchor_sha256,
            created_file,
            ..
        }) = receipt
        else {
            return match receipt {
                None => self.inspect(),
                Some(_) => Err("pending guidance edit must be reconciled before removal".into()),
            };
        };
        if path != self.path.to_string_lossy() {
            return Err("guidance receipt belongs to a different file".into());
        }
        let (present, current) = read_file(&self.path, MAX_GUIDANCE_BYTES)?;
        let Some((start, end)) = find_block(&current)? else {
            return Err("managed guidance has drifted; refusing removal".into());
        };
        if !present || hash(&current.as_bytes()[start..end]) != block_sha256 {
            return Err("managed guidance has drifted; refusing removal".into());
        }
        let remove_start = start
            .checked_sub(inserted_prefix.len())
            .ok_or("managed guidance separator has drifted")?;
        if !owned_separator_matches(&current, start, &inserted_prefix, &anchor_sha256) {
            return Err("managed guidance separator has drifted; refusing removal".into());
        }
        let mut next = current.clone();
        next.replace_range(remove_start..end, "");
        let after_present = !created_file || !next.is_empty();
        let removing = Receipt::Removing {
            schema_version: 1,
            path,
            release_id,
            before_sha256: hash(current.as_bytes()),
            after_sha256: hash(next.as_bytes()),
            block_sha256,
            inserted_prefix,
            anchor_sha256,
            created_file,
            after_present,
        };
        write_receipt(&self.receipt, &removing)?;
        let (now_present, now) = read_file(&self.path, MAX_GUIDANCE_BYTES)?;
        if now_present != present || now != current {
            return Err("Codex guidance changed before removal; reconcile the pending edit".into());
        }
        if after_present {
            atomic_replace(&self.path, next.as_bytes())?;
        } else {
            fs::remove_file(&self.path).map_err(str_error)?;
            #[cfg(unix)]
            File::open(self.path.parent().ok_or("managed file has no parent")?)
                .and_then(|file| file.sync_all())
                .map_err(str_error)?;
        }
        fs::remove_file(&self.receipt).map_err(str_error)?;
        self.inspect()
    }

    pub fn reconcile(&self) -> Result<GuidanceReport, String> {
        let _lock = acquire_lock(&self.lock)?;
        let Some(receipt) = read_receipt(&self.receipt)? else {
            return self.inspect();
        };
        if let Receipt::Removing {
            path,
            before_sha256,
            after_sha256,
            after_present,
            ..
        } = &receipt
        {
            if path != &self.path.to_string_lossy() {
                return Err("guidance receipt belongs to a different file".into());
            }
            let (present, current) = read_file(&self.path, MAX_GUIDANCE_BYTES)?;
            let observed = hash(current.as_bytes());
            if present == *after_present && observed == *after_sha256 {
                fs::remove_file(&self.receipt).map_err(str_error)?;
            } else if present && observed == *before_sha256 {
                write_receipt(&self.receipt, &receipt.committed())?;
            } else {
                return Err("pending guidance removal has an unknown file state".into());
            }
            return self.inspect();
        }
        let Receipt::Pending {
            path,
            before_present,
            before_sha256,
            after_sha256,
            previous,
            ..
        } = &receipt
        else {
            return self.inspect();
        };
        if path != &self.path.to_string_lossy() {
            return Err("guidance receipt belongs to a different file".into());
        }
        let (present, current) = read_file(&self.path, MAX_GUIDANCE_BYTES)?;
        let observed = hash(current.as_bytes());
        if observed == *after_sha256 && present {
            write_receipt(&self.receipt, &receipt.committed())?;
        } else if present == *before_present && observed == *before_sha256 {
            if let Some(previous) = previous {
                write_receipt(&self.receipt, previous)?;
            } else {
                fs::remove_file(&self.receipt).map_err(str_error)?;
            }
        } else {
            return Err("pending guidance edit has an unknown file state; inspect it".into());
        }
        self.inspect()
    }
}

impl Receipt {
    fn release_id(&self) -> &str {
        match self {
            Self::Committed { release_id, .. }
            | Self::Pending { release_id, .. }
            | Self::Removing { release_id, .. } => release_id,
        }
    }
    fn committed(&self) -> Self {
        match self {
            Self::Committed { .. } => self.clone(),
            Self::Pending {
                schema_version,
                path,
                release_id,
                block_sha256,
                inserted_prefix,
                anchor_sha256,
                created_file,
                ..
            } => Self::Committed {
                schema_version: *schema_version,
                path: path.clone(),
                release_id: release_id.clone(),
                block_sha256: block_sha256.clone(),
                inserted_prefix: inserted_prefix.clone(),
                anchor_sha256: anchor_sha256.clone(),
                created_file: *created_file,
            },
            Self::Removing {
                schema_version,
                path,
                release_id,
                block_sha256,
                inserted_prefix,
                anchor_sha256,
                created_file,
                ..
            } => Self::Committed {
                schema_version: *schema_version,
                path: path.clone(),
                release_id: release_id.clone(),
                block_sha256: block_sha256.clone(),
                inserted_prefix: inserted_prefix.clone(),
                anchor_sha256: anchor_sha256.clone(),
                created_file: *created_file,
            },
        }
    }
}

fn find_block(content: &str) -> Result<Option<(usize, usize)>, String> {
    let begin = content.match_indices(BEGIN).collect::<Vec<_>>();
    let end = content.match_indices(END).collect::<Vec<_>>();
    match (begin.as_slice(), end.as_slice()) {
        ([], []) => Ok(None),
        ([(start, _)], [(end, _)]) if start < end => {
            let mut finish = end + END.len();
            if content.as_bytes().get(finish) == Some(&b'\n') {
                finish += 1;
            }
            Ok(Some((*start, finish)))
        }
        _ => Err("Codex guidance has malformed or duplicate managed markers".into()),
    }
}

fn owned_separator_matches(
    content: &str,
    block_start: usize,
    separator: &str,
    anchor_sha256: &str,
) -> bool {
    block_start
        .checked_sub(separator.len())
        .and_then(|start| {
            (content.get(start..block_start) == Some(separator))
                .then(|| hash(&content.as_bytes()[..start]))
        })
        .as_deref()
        == Some(anchor_sha256)
}

fn read_file(path: &Path, max: u64) -> Result<(bool, String), String> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((false, String::new()));
        }
        Err(error) => return Err(str_error(error)),
    };
    let meta = fs::symlink_metadata(path).map_err(str_error)?;
    if meta.file_type().is_symlink() || !meta.is_file() || meta.len() > max {
        return Err("managed file must be a bounded regular file".into());
    }
    let mut bytes = Vec::new();
    file.take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(str_error)?;
    if bytes.len() as u64 > max {
        return Err("managed file exceeds its size limit".into());
    }
    let text = String::from_utf8(bytes).map_err(str_error)?;
    Ok((true, text))
}

fn read_receipt(path: &Path) -> Result<Option<Receipt>, String> {
    let (present, text) = read_file(path, MAX_RECEIPT_BYTES)?;
    if !present {
        return Ok(None);
    }
    let receipt: Receipt = serde_json::from_str(&text).map_err(str_error)?;
    let (schema, prefix, created_file) = match &receipt {
        Receipt::Committed {
            schema_version,
            inserted_prefix,
            created_file,
            ..
        }
        | Receipt::Pending {
            schema_version,
            inserted_prefix,
            created_file,
            ..
        }
        | Receipt::Removing {
            schema_version,
            inserted_prefix,
            created_file,
            ..
        } => (*schema_version, inserted_prefix.as_str(), *created_file),
    };
    if schema != 1
        || !matches!(prefix, "" | "\n" | "\n\n")
        || (created_file && !prefix.is_empty())
        || matches!(
            &receipt,
            Receipt::Removing {
                after_present: false,
                created_file: false,
                ..
            }
        )
        || matches!(&receipt, Receipt::Pending { previous: Some(previous), .. } if !matches!(previous.as_ref(), Receipt::Committed { .. }))
    {
        return Err("Codex guidance receipt has an unsupported state".into());
    }
    Ok(Some(receipt))
}

fn write_receipt(path: &Path, receipt: &Receipt) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(receipt).map_err(str_error)?;
    if bytes.len() as u64 > MAX_RECEIPT_BYTES {
        return Err("Codex guidance receipt exceeds its size limit".into());
    }
    atomic_replace(path, &bytes)
}

fn acquire_lock(path: &Path) -> Result<FileLock, String> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = match options.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let meta = fs::symlink_metadata(path).map_err(str_error)?;
            if meta.file_type().is_symlink() || !meta.is_file() || meta.len() != 0 {
                return Err("Codex guidance lock is not an empty regular file".into());
            }
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .map_err(str_error)?
        }
        Err(error) => return Err(str_error(error)),
    };
    file.try_lock()
        .map_err(|_| "another Codex guidance operation owns the lock")?;
    Ok(FileLock::acquired(file))
}

fn atomic_replace(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temp = path.with_file_name(format!(
        ".{}-next-{}",
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("guidance"),
        std::process::id()
    ));
    if fs::symlink_metadata(&temp).is_ok() {
        return Err("managed guidance temporary file already exists".into());
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temp).map_err(str_error)?;
    if let Err(error) = file.write_all(bytes).and_then(|_| file.sync_all()) {
        let _ = fs::remove_file(&temp);
        return Err(str_error(error));
    }
    #[cfg(not(windows))]
    let replaced = fs::rename(&temp, path).map_err(str_error);
    #[cfg(windows)]
    let replaced = {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{
            MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
        };
        let from = temp
            .as_os_str()
            .encode_wide()
            .chain([0])
            .collect::<Vec<_>>();
        let to = path
            .as_os_str()
            .encode_wide()
            .chain([0])
            .collect::<Vec<_>>();
        if unsafe {
            MoveFileExW(
                from.as_ptr(),
                to.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        } == 0
        {
            Err(str_error(std::io::Error::last_os_error()))
        } else {
            Ok(())
        }
    };
    if replaced.is_err() {
        let _ = fs::remove_file(&temp);
    }
    replaced?;
    #[cfg(unix)]
    File::open(path.parent().ok_or("managed file has no parent")?)
        .and_then(|file| file.sync_all())
        .map_err(str_error)?;
    Ok(())
}

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn str_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "mirket-codex-guidance-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(root.join("codex")).unwrap();
            fs::create_dir_all(root.join("state")).unwrap();
            Self(root.canonicalize().unwrap())
        }
        fn guidance(&self) -> CodexGuidance {
            CodexGuidance::open(
                &self.0.join("codex/AGENTS.md"),
                &self.0.join("state/codex-guidance.json"),
            )
            .unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn installs_and_updates_only_the_owned_section() {
        let fixture = Fixture::new();
        let manager = fixture.guidance();
        fs::write(&manager.path, "User preference: concise answers.\n").unwrap();
        assert_eq!(manager.install("0.0.1:a").unwrap().status, "managed");
        let first = fs::read_to_string(&manager.path).unwrap();
        assert!(first.starts_with("User preference: concise answers.\n"));
        assert_eq!(first.matches(BEGIN).count(), 1);
        assert_eq!(manager.install("0.0.1:a").unwrap().status, "managed");
        assert_eq!(fs::read_to_string(&manager.path).unwrap(), first);
        fs::write(&manager.path, format!("{first}\nUser addition.\n")).unwrap();
        assert_eq!(manager.install("0.0.1:b").unwrap().status, "managed");
        let updated = fs::read_to_string(&manager.path).unwrap();
        assert!(updated.starts_with("User preference: concise answers.\n"));
        assert!(updated.ends_with("\nUser addition.\n"));
        assert_eq!(updated.matches(BEGIN).count(), 1);
        assert_eq!(
            manager.inspect().unwrap().release_id.as_deref(),
            Some("0.0.1:b")
        );
    }

    #[test]
    fn collisions_and_drift_refuse_without_replacing_user_bytes() {
        let fixture = Fixture::new();
        let manager = fixture.guidance();
        let foreign = format!("{BEGIN}\nuser text\n{END}\n");
        fs::write(&manager.path, &foreign).unwrap();
        assert!(manager.install("0.0.1:a").is_err());
        assert_eq!(fs::read_to_string(&manager.path).unwrap(), foreign);
        fs::remove_file(&manager.path).unwrap();
        manager.install("0.0.1:a").unwrap();
        let tampered = fs::read_to_string(&manager.path)
            .unwrap()
            .replace(BODY, "changed body");
        fs::write(&manager.path, &tampered).unwrap();
        assert_eq!(manager.inspect().unwrap().status, "drifted");
        assert!(manager.install("0.0.1:b").is_err());
        assert_eq!(fs::read_to_string(&manager.path).unwrap(), tampered);
    }

    #[test]
    fn reconciles_known_before_and_after_states_only() {
        let fixture = Fixture::new();
        let manager = fixture.guidance();
        let block = format!("{BEGIN}\n{BODY}\n{END}\n");
        let pending = Receipt::Pending {
            schema_version: 1,
            path: manager.path.to_string_lossy().into_owned(),
            release_id: "0.0.1:a".into(),
            before_present: false,
            before_sha256: hash(b""),
            after_sha256: hash(block.as_bytes()),
            block_sha256: hash(block.as_bytes()),
            inserted_prefix: String::new(),
            anchor_sha256: hash(b""),
            created_file: true,
            previous: None,
        };
        write_receipt(&manager.receipt, &pending).unwrap();
        assert_eq!(manager.reconcile().unwrap().status, "missing");
        assert!(!manager.receipt.exists());
        write_receipt(&manager.receipt, &pending).unwrap();
        fs::write(&manager.path, &block).unwrap();
        assert_eq!(manager.reconcile().unwrap().status, "managed");
        assert!(matches!(
            read_receipt(&manager.receipt).unwrap(),
            Some(Receipt::Committed { .. })
        ));
    }

    #[test]
    fn removal_preserves_unrelated_guidance_and_refuses_drift() {
        let fixture = Fixture::new();
        let manager = fixture.guidance();
        fs::write(&manager.path, "User instruction.\n").unwrap();
        manager.install("0.0.1:a").unwrap();
        let first = fs::read_to_string(&manager.path).unwrap();
        fs::write(&manager.path, format!("{first}\nLater user instruction.\n")).unwrap();
        assert_eq!(manager.remove().unwrap().status, "missing");
        assert_eq!(
            fs::read_to_string(&manager.path).unwrap(),
            "User instruction.\n\nLater user instruction.\n"
        );
        assert!(!manager.receipt.exists());
        manager.install("0.0.1:b").unwrap();
        let tampered = fs::read_to_string(&manager.path)
            .unwrap()
            .replace(BODY, "changed body");
        fs::write(&manager.path, &tampered).unwrap();
        assert!(manager.remove().is_err());
        assert_eq!(fs::read_to_string(&manager.path).unwrap(), tampered);
    }

    #[test]
    fn removal_restores_original_file_bytes_and_absence() {
        let fixture = Fixture::new();
        let manager = fixture.guidance();
        let original = "User preference without a final newline";
        fs::write(&manager.path, original).unwrap();
        manager.install("0.0.1:a").unwrap();
        assert_eq!(manager.remove().unwrap().status, "missing");
        assert_eq!(fs::read_to_string(&manager.path).unwrap(), original);

        fs::remove_file(&manager.path).unwrap();
        manager.install("0.0.1:b").unwrap();
        assert_eq!(manager.remove().unwrap().status, "missing");
        assert!(!manager.path.exists());
        assert!(!manager.receipt.exists());
    }

    #[test]
    fn separator_edit_is_reported_as_drift_and_preserved() {
        let fixture = Fixture::new();
        let manager = fixture.guidance();
        fs::write(&manager.path, "User instruction.\n").unwrap();
        manager.install("0.0.1:a").unwrap();
        let changed = fs::read_to_string(&manager.path).unwrap().replacen(
            &format!("\n\n{BEGIN}"),
            &format!("\n{BEGIN}"),
            1,
        );
        fs::write(&manager.path, &changed).unwrap();
        assert_eq!(manager.inspect().unwrap().status, "drifted");
        assert!(manager.remove().is_err());
        assert_eq!(fs::read_to_string(&manager.path).unwrap(), changed);
    }

    #[test]
    fn reconciles_interrupted_removal_without_replaying_unknown_edit() {
        let fixture = Fixture::new();
        let manager = fixture.guidance();
        manager.install("0.0.1:a").unwrap();
        let current = fs::read_to_string(&manager.path).unwrap();
        let (start, end) = find_block(&current).unwrap().unwrap();
        let mut next = current.clone();
        next.replace_range(start..end, "");
        let removing = Receipt::Removing {
            schema_version: 1,
            path: manager.path.to_string_lossy().into_owned(),
            release_id: "0.0.1:a".into(),
            before_sha256: hash(current.as_bytes()),
            after_sha256: hash(next.as_bytes()),
            block_sha256: hash(&current.as_bytes()[start..end]),
            inserted_prefix: String::new(),
            anchor_sha256: hash(b""),
            created_file: true,
            after_present: false,
        };
        write_receipt(&manager.receipt, &removing).unwrap();
        assert_eq!(manager.reconcile().unwrap().status, "managed");
        write_receipt(&manager.receipt, &removing).unwrap();
        fs::remove_file(&manager.path).unwrap();
        assert_eq!(manager.reconcile().unwrap().status, "missing");
        assert!(!manager.receipt.exists());
    }
}
