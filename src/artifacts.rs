//! Bounded single-file publication shared by text and binary artifact owners.
//!
//! Format validation belongs to the caller; this is not an editor or filesystem sandbox. Paths must be in an
//! explicitly selected existing root. Detected symlinks are refused, but std's
//! path-based calls cannot protect against concurrent ancestor replacement.
//! Native directory-handle qualification is required before hostile concurrent
//! filesystem mutation can be supported. In-place publication is deliberately
//! unavailable until conditional replacement can preserve newer external edits.
//! Source checks immediately precede publication but are not an atomic lock on
//! other editors: a source can change after the final check. Publication never
//! replaces an existing destination, including an independently created edit.
//!
//! Saved inspection retains exact bytes in memory, not a homemade digest. It is
//! revision-bound for this process; it is not a durable reconciliation receipt.
//! The caller owns release/permission checks and must persist its commit intent
//! before calling `publish`, then record the outcome without automatically
//! replaying an unknown effect. No crash-durability claim is made for directory
//! entries. A process crash may leave a staging file for explicit reconciliation.

use sha2::{Digest, Sha256};
use std::fmt;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

pub const DEFAULT_MAX_BYTES: usize = 1024 * 1024;
/// A caller can lower this limit, but cannot turn the fixture route into an
/// unbounded document import or allocate an arbitrary amount of memory.
pub const HARD_MAX_BYTES: usize = 20 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PublicationMode {
    #[default]
    NewCopy,
    /// Explicit intent only. Preparation validates paths, then refuses it.
    InPlace,
}

#[derive(Debug)]
pub enum ArtifactError {
    Io {
        operation: &'static str,
        source: io::Error,
    },
    InvalidLimit,
    OutsideScope,
    Symlink,
    NotRegularFile,
    DestinationExists,
    InvalidInPlaceDestination,
    TooLarge {
        limit: usize,
    },
    Conflict {
        object: &'static str,
    },
    InPlaceNotQualified,
    /// The final entry may exist. Inspect it; do not retry automatically.
    PublicationUncertain,
    VideoStopped,
    VideoStageRetained {
        path: PathBuf,
        cause: Box<ArtifactError>,
    },
}

impl fmt::Display for ArtifactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
            Self::InvalidLimit => write!(f, "byte limit must be between 1 and {HARD_MAX_BYTES}"),
            Self::OutsideScope => write!(
                f,
                "artifact path is outside the selected root or contains traversal"
            ),
            Self::Symlink => write!(
                f,
                "symlink paths are not supported by the artifact prototype"
            ),
            Self::NotRegularFile => write!(f, "artifact input must be a regular file"),
            Self::DestinationExists => {
                write!(f, "new-copy destination already exists; select a new path")
            }
            Self::InvalidInPlaceDestination => write!(
                f,
                "in-place destination must be the exact selected source path"
            ),
            Self::TooLarge { limit } => write!(f, "artifact exceeds the {limit}-byte limit"),
            Self::Conflict { object } => write!(
                f,
                "{object} changed since selection; inspect and start a new operation"
            ),
            Self::InPlaceNotQualified => write!(
                f,
                "in-place publication is unavailable until native conditional replacement is qualified"
            ),
            Self::VideoStopped => write!(f, "video cancelled or deadline exceeded during file I/O"),
            Self::VideoStageRetained { path, cause } => write!(
                f,
                "video staging failed ({cause}); partial stage retained at {}",
                path.display()
            ),
            Self::PublicationUncertain => write!(
                f,
                "publication outcome is uncertain; inspect the destination before any new operation"
            ),
        }
    }
}

impl std::error::Error for ArtifactError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn io_error(operation: &'static str, source: io::Error) -> ArtifactError {
    ArtifactError::Io { operation, source }
}

/// Metadata supplements exact byte comparison; it never claims to be a hash.
#[derive(Clone, Debug, Eq, PartialEq)]
struct FileStamp {
    len: u64,
    modified: Option<SystemTime>,
    created: Option<SystemTime>,
    readonly: bool,
    #[cfg(unix)]
    native: (u64, u64, i64, i64),
}

impl FileStamp {
    fn from(meta: &Metadata) -> Self {
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;
        Self {
            len: meta.len(),
            modified: meta.modified().ok(),
            created: meta.created().ok(),
            readonly: meta.permissions().readonly(),
            #[cfg(unix)]
            native: (meta.dev(), meta.ino(), meta.ctime(), meta.ctime_nsec()),
        }
    }
}

#[derive(Clone)]
struct Snapshot {
    stamp: FileStamp,
    bytes: Vec<u8>,
}

impl Snapshot {
    fn same_revision(&self, other: &Self) -> bool {
        self.stamp == other.stamp && self.bytes == other.bytes
    }
}

/// No source or artifact bodies are emitted by Debug/logging.
pub struct ArtifactPlan {
    root: PathBuf,
    source: Option<(PathBuf, Snapshot)>,
    destination: PathBuf,
    parent_stamp: FileStamp,
    max_bytes: usize,
}

impl ArtifactPlan {
    pub fn prepare(
        scope_root: &Path,
        source: Option<&Path>,
        destination: &Path,
        mode: PublicationMode,
        max_bytes: usize,
    ) -> Result<Self, ArtifactError> {
        check_limit(max_bytes)?;
        if !scope_root.is_absolute() {
            return Err(ArtifactError::OutsideScope);
        }
        if fs::symlink_metadata(scope_root)
            .map_err(|e| io_error("read root", e))?
            .file_type()
            .is_symlink()
        {
            return Err(ArtifactError::Symlink);
        }
        let root = fs::canonicalize(scope_root).map_err(|e| io_error("resolve root", e))?;
        if !root.is_dir() {
            return Err(ArtifactError::OutsideScope);
        }
        let destination = scoped_path(&root, destination)?;
        let source = source
            .map(|p| {
                let p = scoped_path(&root, p)?;
                let snapshot = read_snapshot(&root, &p, max_bytes)?;
                Ok((p, snapshot))
            })
            .transpose()?;
        let parent = destination.parent().ok_or(ArtifactError::OutsideScope)?;
        check_path(&root, parent, false)?;
        let parent_meta = fs::metadata(parent).map_err(|e| io_error("read output parent", e))?;
        if !parent_meta.is_dir() {
            return Err(ArtifactError::OutsideScope);
        }
        match mode {
            PublicationMode::NewCopy => require_absent(&root, &destination)?,
            PublicationMode::InPlace => {
                if source.as_ref().map(|(p, _)| p) != Some(&destination) {
                    return Err(ArtifactError::InvalidInPlaceDestination);
                }
                return Err(ArtifactError::InPlaceNotQualified);
            }
        }
        Ok(Self {
            root,
            source,
            destination,
            parent_stamp: FileStamp::from(&parent_meta),
            max_bytes,
        })
    }

    pub fn destination(&self) -> &Path {
        &self.destination
    }

    /// Exact bounded source selected by this plan. This is data, never authority.
    pub fn source_bytes(&self) -> Option<&[u8]> {
        self.source.as_ref().map(|(_, s)| s.bytes.as_slice())
    }

    pub fn stage_bytes(self, bytes: &[u8]) -> Result<StagedArtifact, ArtifactError> {
        if bytes.len() > self.max_bytes {
            return Err(ArtifactError::TooLarge {
                limit: self.max_bytes,
            });
        }
        self.check_conflicts()?;
        let parent = self
            .destination
            .parent()
            .ok_or(ArtifactError::OutsideScope)?;
        let (path, mut file) = create_stage(parent)?;
        let result = file.write_all(bytes).and_then(|_| file.sync_all());
        if let Err(e) = result {
            drop(file);
            // Partial or concurrently substituted staging is retained. A path
            // alone does not prove ownership sufficiently to delete it here.
            return Err(io_error("write staged artifact", e));
        }
        drop(file);
        let snapshot = read_snapshot(&self.root, &path, self.max_bytes)?;
        if snapshot.bytes != bytes {
            return Err(ArtifactError::Conflict {
                object: "staging file",
            });
        }
        Ok(StagedArtifact {
            plan: self,
            path,
            snapshot,
        })
    }

    fn check_conflicts(&self) -> Result<(), ArtifactError> {
        check_path(
            &self.root,
            self.destination
                .parent()
                .ok_or(ArtifactError::OutsideScope)?,
            false,
        )?;
        let meta = fs::metadata(self.destination.parent().unwrap())
            .map_err(|e| io_error("read output parent", e))?;
        // Directory mtimes change when our own stage is created. Native inode
        // identity, where available, is the useful directory replacement check.
        #[cfg(unix)]
        if FileStamp::from(&meta).native.0 != self.parent_stamp.native.0
            || FileStamp::from(&meta).native.1 != self.parent_stamp.native.1
        {
            return Err(ArtifactError::Conflict {
                object: "output parent",
            });
        }
        #[cfg(not(unix))]
        let _ = (&meta, &self.parent_stamp);
        if let Some((path, expected)) = &self.source {
            let actual = read_snapshot(&self.root, path, self.max_bytes)?;
            if !expected.same_revision(&actual) {
                return Err(ArtifactError::Conflict { object: "source" });
            }
        }
        require_absent(&self.root, &self.destination)?;
        Ok(())
    }
}

pub struct StagedArtifact {
    plan: ArtifactPlan,
    path: PathBuf,
    snapshot: Snapshot,
}

impl StagedArtifact {
    pub fn destination(&self) -> &Path {
        self.plan.destination()
    }
    pub fn staging_path(&self) -> &Path {
        &self.path
    }

    /// Caller must persist its minimal commit intent before this call. A hard
    /// link publishes a complete file without replacing any existing entry.
    /// Filesystems that do not support hard links fail without a copy fallback.
    pub fn publish(self) -> Result<SavedArtifact, ArtifactError> {
        self.plan.check_conflicts()?;
        let staged = read_snapshot(&self.plan.root, &self.path, self.plan.max_bytes)?;
        if !self.snapshot.same_revision(&staged) {
            return Err(ArtifactError::Conflict {
                object: "staging file",
            });
        }
        fs::hard_link(&self.path, &self.plan.destination).map_err(|e| {
            if e.kind() == io::ErrorKind::AlreadyExists {
                ArtifactError::DestinationExists
            } else {
                io_error("publish new copy", e)
            }
        })?;
        // Removing our staging link changes Unix ctime, so capture the saved
        // revision only after cleanup. Never delete the destination on failure.
        fs::remove_file(&self.path).map_err(|_| ArtifactError::PublicationUncertain)?;
        let saved = read_snapshot(&self.plan.root, &self.plan.destination, self.plan.max_bytes)
            .map_err(|_| ArtifactError::PublicationUncertain)?;
        if saved.bytes != self.snapshot.bytes {
            return Err(ArtifactError::PublicationUncertain);
        }
        Ok(SavedArtifact {
            path: self.plan.destination.clone(),
            root: self.plan.root.clone(),
            byte_len: saved.bytes.len(),
            snapshot: saved,
        })
    }
}

impl Drop for StagedArtifact {
    fn drop(&mut self) {
        // Do not remove a substituted file. Failure leaves inspectable scratch.
        if let Ok(current) = read_snapshot(&self.plan.root, &self.path, self.plan.max_bytes)
            && self.snapshot.same_revision(&current)
        {
            let _ = fs::remove_file(&self.path);
        }
    }
}

/// Evidence covers an exact saved byte revision and extent only.
/// No layout, Office compatibility, semantic correctness or durability claim.
pub struct SavedArtifact {
    pub path: PathBuf,
    pub byte_len: usize,
    root: PathBuf,
    snapshot: Snapshot,
}

impl SavedArtifact {
    /// A full bounded read: failure is explicit, never a silently truncated view.
    pub fn inspect_bytes(&self, max_bytes: usize) -> Result<Vec<u8>, ArtifactError> {
        check_limit(max_bytes)?;
        let actual = read_snapshot(&self.root, &self.path, max_bytes)?;
        if !self.snapshot.same_revision(&actual) {
            return Err(ArtifactError::Conflict {
                object: "saved artifact",
            });
        }
        Ok(actual.bytes)
    }
}

/// Video-only saved-file revision. Body bytes never accumulate in memory. This
/// route does not raise the buffered artifact/source limits above.
#[derive(Clone)]
pub(crate) struct VideoFileRevision {
    root: PathBuf,
    path: PathBuf,
    stamp: FileStamp,
    pub revision: String,
    pub byte_len: usize,
}
impl VideoFileRevision {
    pub fn inspect(root: &Path, path: &Path, max_bytes: usize) -> Result<Self, ArtifactError> {
        Self::inspect_checked(root, path, max_bytes, &|| true)
    }
    pub fn inspect_checked(
        root: &Path,
        path: &Path,
        max_bytes: usize,
        keep_running: &impl Fn() -> bool,
    ) -> Result<Self, ArtifactError> {
        if !keep_running() {
            return Err(ArtifactError::VideoStopped);
        }
        if max_bytes == 0 || max_bytes > crate::video::MAX_OUTPUT_BYTES {
            return Err(ArtifactError::InvalidLimit);
        }
        let path = scoped_path(root, path)?;
        check_path(root, &path, false)?;
        let before = fs::symlink_metadata(&path).map_err(|e| io_error("inspect video", e))?;
        if !before.is_file() {
            return Err(ArtifactError::NotRegularFile);
        }
        if before.len() > max_bytes as u64 {
            return Err(ArtifactError::TooLarge { limit: max_bytes });
        }
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW);
        }
        let mut file = options.open(&path).map_err(|e| io_error("open video", e))?;
        let stamp = FileStamp::from(&before);
        if FileStamp::from(
            &file
                .metadata()
                .map_err(|e| io_error("inspect open video", e))?,
        ) != stamp
        {
            return Err(ArtifactError::Conflict {
                object: "video file",
            });
        }
        let mut hash = Sha256::new();
        let mut total = 0usize;
        let mut buffer = [0u8; 65536];
        loop {
            if !keep_running() {
                return Err(ArtifactError::VideoStopped);
            }
            let n = file
                .read(&mut buffer)
                .map_err(|e| io_error("hash video", e))?;
            if n == 0 {
                break;
            }
            total = total
                .checked_add(n)
                .ok_or(ArtifactError::TooLarge { limit: max_bytes })?;
            if total > max_bytes {
                return Err(ArtifactError::TooLarge { limit: max_bytes });
            }
            hash.update(&buffer[..n]);
        }
        let snapshot = Self {
            root: root.into(),
            path,
            stamp,
            byte_len: total,
            revision: format!(
                "sha256:{}",
                hash.finalize()
                    .iter()
                    .map(|v| format!("{v:02x}"))
                    .collect::<String>()
            ),
        };
        snapshot.verify_reader(&file)?;
        if total as u64 != snapshot.stamp.len {
            return Err(ArtifactError::Conflict {
                object: "video file",
            });
        }
        Ok(snapshot)
    }
    pub fn open_reader(&self) -> Result<File, ArtifactError> {
        check_path(&self.root, &self.path, false)?;
        let mut file = File::open(&self.path).map_err(|e| io_error("reopen video", e))?;
        self.verify_reader(&file)?;
        file.seek(SeekFrom::Start(0))
            .map_err(|e| io_error("seek video", e))?;
        Ok(file)
    }
    pub fn verify_reader(&self, file: &File) -> Result<(), ArtifactError> {
        check_path(&self.root, &self.path, false)?;
        let path_meta =
            fs::symlink_metadata(&self.path).map_err(|e| io_error("recheck video", e))?;
        let open_meta = file
            .metadata()
            .map_err(|e| io_error("recheck open video", e))?;
        if FileStamp::from(&path_meta) != self.stamp || FileStamp::from(&open_meta) != self.stamp {
            return Err(ArtifactError::Conflict {
                object: "video file",
            });
        }
        Ok(())
    }
    fn same_revision(&self, other: &Self) -> bool {
        self.stamp == other.stamp
            && self.revision == other.revision
            && self.byte_len == other.byte_len
    }
    fn remove_if_unchanged(&self) -> Result<(), ArtifactError> {
        // The exact body was already hashed. Unix inode + ctime/mtime/extent
        // bind that revision without rereading gigabytes on an async-thread
        // drop. Native non-Unix deletion ownership is not qualified.
        #[cfg(not(unix))]
        return Err(ArtifactError::Conflict {
            object: "unqualified video stage cleanup",
        });
        #[cfg(unix)]
        {
            let reader = self.open_reader()?;
            self.verify_reader(&reader)?;
            fs::remove_file(&self.path).map_err(|e| io_error("remove owned video stage", e))
        }
    }
}
impl ArtifactPlan {
    /// Copy a verified video into a private sibling stage using a 64 KiB buffer.
    /// The small source selection and destination conflict checks are retained.
    pub(crate) fn stage_video_file(
        self,
        source: &VideoFileRevision,
        keep_running: &impl Fn() -> bool,
    ) -> Result<StagedVideoArtifact, ArtifactError> {
        self.check_conflicts()?;
        let mut input = source.open_reader()?;
        let (path, file) = create_stage(
            self.destination
                .parent()
                .ok_or(ArtifactError::OutsideScope)?,
        )?;
        let mut output = VideoStageWriter {
            file,
            hash: Sha256::new(),
            byte_len: 0,
        };
        let snapshot = (|| {
            copy_video_body(&mut input, &mut output, source.byte_len, keep_running)?;
            output
                .file
                .sync_all()
                .map_err(|e| io_error("sync video stage", e))?;
            source.verify_reader(&input)?;
            let snapshot = VideoFileRevision::inspect_checked(
                &self.root,
                &path,
                crate::video::MAX_OUTPUT_BYTES,
                keep_running,
            )?;
            if snapshot.revision != source.revision || snapshot.byte_len != source.byte_len {
                return Err(ArtifactError::Conflict {
                    object: "video stage",
                });
            }
            Ok(snapshot)
        })()
        .map_err(|cause| {
            // This function runs on the blocking I/O worker. Read back only
            // the actual prefix written, then remove it if it is still ours.
            // ENOSPC/cancellation do not strand known-owned sibling files.
            if cleanup_partial_video_stage(&self.root, &path, &output).is_ok() {
                cause
            } else {
                ArtifactError::VideoStageRetained {
                    path: path.clone(),
                    cause: Box::new(cause),
                }
            }
        })?;

        Ok(StagedVideoArtifact {
            plan: self,
            path,
            snapshot,
        })
    }
}
struct VideoStageWriter {
    file: File,
    hash: Sha256,
    byte_len: usize,
}
impl Write for VideoStageWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let n = self.file.write(bytes)?;
        self.hash.update(&bytes[..n]);
        self.byte_len += n;
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}
fn cleanup_partial_video_stage(
    root: &Path,
    path: &Path,
    output: &VideoStageWriter,
) -> Result<(), ArtifactError> {
    let current = VideoFileRevision::inspect(root, path, crate::video::MAX_OUTPUT_BYTES)?;
    let expected_revision = format!(
        "sha256:{}",
        output
            .hash
            .clone()
            .finalize()
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
    );
    if current.byte_len != output.byte_len
        || current.revision != expected_revision
        || FileStamp::from(
            &output
                .file
                .metadata()
                .map_err(|e| io_error("inspect partial video stage", e))?,
        ) != current.stamp
    {
        return Err(ArtifactError::Conflict {
            object: "partial video stage",
        });
    }
    current.remove_if_unchanged()
}

fn copy_video_body<R: Read, W: Write>(
    input: &mut R,
    output: &mut W,
    expected_bytes: usize,
    keep_running: &impl Fn() -> bool,
) -> Result<(), ArtifactError> {
    let mut buffer = [0u8; 65536];
    let mut total = 0usize;
    loop {
        if !keep_running() {
            return Err(ArtifactError::VideoStopped);
        }
        let n = input
            .read(&mut buffer)
            .map_err(|e| io_error("read video stage", e))?;
        if n == 0 {
            break;
        }
        total = total.checked_add(n).ok_or(ArtifactError::TooLarge {
            limit: expected_bytes,
        })?;
        if total > expected_bytes {
            return Err(ArtifactError::Conflict {
                object: "video file",
            });
        }
        output
            .write_all(&buffer[..n])
            .map_err(|e| io_error("copy video stage", e))?;
    }
    if total != expected_bytes {
        return Err(ArtifactError::Conflict {
            object: "video file",
        });
    }
    Ok(())
}

pub(crate) struct StagedVideoArtifact {
    plan: ArtifactPlan,
    path: PathBuf,
    snapshot: VideoFileRevision,
}
impl StagedVideoArtifact {
    /// The caller persists its commit intent first; failures after linking are
    /// uncertain and never remove or replay the destination.
    pub fn publish(
        self,
        keep_running: &impl Fn() -> bool,
    ) -> Result<VideoFileRevision, ArtifactError> {
        self.plan.check_conflicts()?;
        let current = VideoFileRevision::inspect_checked(
            &self.plan.root,
            &self.path,
            crate::video::MAX_OUTPUT_BYTES,
            keep_running,
        )?;
        if !current.same_revision(&self.snapshot) {
            return Err(ArtifactError::Conflict {
                object: "video stage",
            });
        }
        if !keep_running() {
            return Err(ArtifactError::VideoStopped);
        }
        // Hashing a large file can take seconds: recheck selected sources and
        // the new-copy destination again immediately before the final effect.
        self.plan.check_conflicts()?;
        let reader = current.open_reader()?;
        current.verify_reader(&reader)?;
        fs::hard_link(&self.path, &self.plan.destination).map_err(|e| {
            if e.kind() == io::ErrorKind::AlreadyExists {
                ArtifactError::DestinationExists
            } else {
                io_error("publish video new copy", e)
            }
        })?;
        fs::remove_file(&self.path).map_err(|_| ArtifactError::PublicationUncertain)?;
        let saved = VideoFileRevision::inspect(
            &self.plan.root,
            &self.plan.destination,
            crate::video::MAX_OUTPUT_BYTES,
        )
        .map_err(|_| ArtifactError::PublicationUncertain)?;
        if saved.revision != self.snapshot.revision || saved.byte_len != self.snapshot.byte_len {
            return Err(ArtifactError::PublicationUncertain);
        }
        Ok(saved)
    }
}
impl Drop for StagedVideoArtifact {
    fn drop(&mut self) {
        // No body reads here: a staged value may be dropped by a cancelled
        // caller on the single async runtime thread. Changed identity or a
        // failed unlink retains the file for explicit inspection.
        let _ = self.snapshot.remove_if_unchanged();
    }
}

fn check_limit(limit: usize) -> Result<(), ArtifactError> {
    if limit == 0 || limit > HARD_MAX_BYTES {
        Err(ArtifactError::InvalidLimit)
    } else {
        Ok(())
    }
}

fn scoped_path(root: &Path, path: &Path) -> Result<PathBuf, ArtifactError> {
    let full = if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    };
    let relative = full
        .strip_prefix(root)
        .map_err(|_| ArtifactError::OutsideScope)?;
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(ArtifactError::OutsideScope);
    }
    Ok(full)
}

fn check_path(root: &Path, path: &Path, absent_leaf: bool) -> Result<(), ArtifactError> {
    // The stored root was canonical when selected. A persistent symlink in any
    // ancestor must not retarget it, even when the directory inode stays equal.
    if fs::canonicalize(root).map_err(|e| io_error("recheck root ancestry", e))? != root {
        return Err(ArtifactError::Symlink);
    }
    let relative = path
        .strip_prefix(root)
        .map_err(|_| ArtifactError::OutsideScope)?;
    let mut current = root.to_owned();
    if fs::symlink_metadata(&current)
        .map_err(|e| io_error("read selected root", e))?
        .file_type()
        .is_symlink()
    {
        return Err(ArtifactError::Symlink);
    }
    for component in relative.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(ArtifactError::OutsideScope);
        }
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => return Err(ArtifactError::Symlink),
            Ok(_) => (),
            Err(e) if absent_leaf && current == path && e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(io_error("inspect artifact path", e)),
        }
    }
    Ok(())
}

fn require_absent(root: &Path, path: &Path) -> Result<(), ArtifactError> {
    check_path(root, path, true)?;
    match fs::symlink_metadata(path) {
        Ok(_) => Err(ArtifactError::DestinationExists),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(io_error("inspect destination", e)),
    }
}

fn read_snapshot(root: &Path, path: &Path, limit: usize) -> Result<Snapshot, ArtifactError> {
    check_path(root, path, false)?;
    let before = fs::symlink_metadata(path).map_err(|e| io_error("read artifact metadata", e))?;
    if !before.is_file() {
        return Err(ArtifactError::NotRegularFile);
    }
    if before.len() > limit as u64 {
        return Err(ArtifactError::TooLarge { limit });
    }
    let mut file = File::open(path).map_err(|e| io_error("open artifact", e))?;
    let stamp = FileStamp::from(&before);
    if FileStamp::from(
        &file
            .metadata()
            .map_err(|e| io_error("read open artifact metadata", e))?,
    ) != stamp
    {
        return Err(ArtifactError::Conflict {
            object: "artifact during read",
        });
    }
    let mut bytes = Vec::new();
    (&mut file)
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| io_error("read artifact", e))?;
    if bytes.len() > limit {
        return Err(ArtifactError::TooLarge { limit });
    }
    check_path(root, path, false)?;
    let after = fs::symlink_metadata(path).map_err(|e| io_error("recheck artifact", e))?;
    if FileStamp::from(&after) != stamp
        || FileStamp::from(
            &file
                .metadata()
                .map_err(|e| io_error("recheck open artifact", e))?,
        ) != stamp
        || bytes.len() as u64 != stamp.len
    {
        return Err(ArtifactError::Conflict {
            object: "artifact during read",
        });
    }
    Ok(Snapshot { stamp, bytes })
}

fn create_stage(parent: &Path) -> Result<(PathBuf, File), ArtifactError> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    for _ in 0..128 {
        let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(".mirket-stage-{}-{sequence}", std::process::id()));
        let mut options = OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(io_error("create staging file", e)),
        }
    }
    Err(io_error(
        "create staging file",
        io::Error::new(
            io::ErrorKind::AlreadyExists,
            "staging name attempts exhausted",
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            static ID: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "mirket-artifacts-test-{}-{}",
                std::process::id(),
                ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(fs::canonicalize(path).unwrap())
        }
        fn path(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
        fn plan(
            &self,
            source: Option<&str>,
            destination: &str,
            mode: PublicationMode,
        ) -> ArtifactPlan {
            ArtifactPlan::prepare(
                &self.0,
                source.map(Path::new),
                Path::new(destination),
                mode,
                DEFAULT_MAX_BYTES,
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
    fn publishes_and_reopens_actual_saved_utf8_without_changing_source() {
        let f = Fixture::new();
        let source = include_str!("../tests/fixtures/artifacts/source.txt");
        let expected = include_str!("../tests/fixtures/artifacts/expected.txt");
        fs::write(f.path("source.txt"), source).unwrap();
        let plan = f.plan(Some("source.txt"), "özet.txt", PublicationMode::default());
        assert_eq!(plan.source_bytes(), Some(source.as_bytes()));
        let staged = plan.stage_bytes(expected.as_bytes()).unwrap();
        let scratch = staged.staging_path().to_owned();
        assert!(!f.path("özet.txt").exists());
        let saved = staged.publish().unwrap();
        assert!(!scratch.exists());
        assert_eq!(
            saved.inspect_bytes(DEFAULT_MAX_BYTES).unwrap(),
            expected.as_bytes()
        );
        assert_eq!(saved.byte_len, expected.len());
        assert_eq!(fs::read_to_string(f.path("source.txt")).unwrap(), source);
    }

    #[test]
    fn binary_round_trip_preserves_bytes_and_detects_source_drift() {
        let f = Fixture::new();
        let original = b"%PDF-1.7\n%\xff\xfe\x00\r\n";
        let output = b"%PDF-1.7\n%\x80\x00\xfe\r\nnew";
        fs::write(f.path("source.bin"), original).unwrap();
        let plan = f.plan(Some("source.bin"), "result.bin", PublicationMode::NewCopy);
        assert_eq!(plan.source_bytes(), Some(original.as_slice()));
        let saved = plan.stage_bytes(output).unwrap().publish().unwrap();
        assert_eq!(saved.inspect_bytes(100).unwrap(), output);
        assert_eq!(fs::read(f.path("source.bin")).unwrap(), original);
        let staged = f
            .plan(Some("source.bin"), "other.bin", PublicationMode::NewCopy)
            .stage_bytes(output)
            .unwrap();
        fs::write(f.path("source.bin"), b"changed binary\xff").unwrap();
        assert!(matches!(
            staged.publish(),
            Err(ArtifactError::Conflict { object: "source" })
        ));
        assert!(!f.path("other.bin").exists());
    }

    #[test]
    fn source_drift_after_staging_aborts_without_output() {
        let f = Fixture::new();
        fs::write(f.path("source"), "original").unwrap();
        let staged = f
            .plan(Some("source"), "output", PublicationMode::NewCopy)
            .stage_bytes(b"result")
            .unwrap();
        let scratch = staged.staging_path().to_owned();
        fs::write(f.path("source"), "new edit").unwrap();
        assert!(matches!(
            staged.publish(),
            Err(ArtifactError::Conflict { object: "source" })
        ));
        assert_eq!(fs::read_to_string(f.path("source")).unwrap(), "new edit");
        assert!(!f.path("output").exists());
        assert!(!scratch.exists());
    }

    #[test]
    fn destination_created_after_staging_is_never_overwritten() {
        let f = Fixture::new();
        let staged = f
            .plan(None, "output", PublicationMode::NewCopy)
            .stage_bytes(b"ours")
            .unwrap();
        fs::write(f.path("output"), "new external edit").unwrap();
        assert!(matches!(
            staged.publish(),
            Err(ArtifactError::DestinationExists)
        ));
        assert_eq!(
            fs::read_to_string(f.path("output")).unwrap(),
            "new external edit"
        );
    }

    #[test]
    fn existing_destination_and_hard_link_alias_are_refused() {
        let f = Fixture::new();
        fs::write(f.path("source"), "original").unwrap();
        fs::hard_link(f.path("source"), f.path("alias")).unwrap();
        for output in ["source", "alias"] {
            assert!(matches!(
                ArtifactPlan::prepare(
                    &f.0,
                    Some(Path::new("source")),
                    Path::new(output),
                    PublicationMode::NewCopy,
                    100
                ),
                Err(ArtifactError::DestinationExists)
            ));
        }
        assert!(matches!(
            ArtifactPlan::prepare(
                &f.0,
                Some(Path::new("source")),
                Path::new("alias"),
                PublicationMode::InPlace,
                100
            ),
            Err(ArtifactError::InvalidInPlaceDestination)
        ));
    }

    #[test]
    fn missing_output_parent_is_not_implicitly_created() {
        let f = Fixture::new();
        assert!(matches!(
            ArtifactPlan::prepare(
                &f.0,
                None,
                Path::new("missing/output"),
                PublicationMode::NewCopy,
                100
            ),
            Err(ArtifactError::Io { .. })
        ));
        assert!(!f.path("missing").exists());
    }

    #[test]
    fn in_place_is_explicit_and_fails_closed_preserving_original() {
        let f = Fixture::new();
        fs::write(f.path("source"), "original").unwrap();
        assert!(matches!(
            ArtifactPlan::prepare(
                &f.0,
                Some(Path::new("source")),
                Path::new("source"),
                PublicationMode::InPlace,
                100
            ),
            Err(ArtifactError::InPlaceNotQualified)
        ));
        assert_eq!(fs::read_to_string(f.path("source")).unwrap(), "original");
        assert_eq!(
            fs::read_dir(&f.0).unwrap().count(),
            1,
            "unsupported in-place requests must not create staging files"
        );
    }

    #[test]
    fn changed_staging_file_cannot_be_reported_as_our_result() {
        let f = Fixture::new();
        let staged = f
            .plan(None, "output", PublicationMode::NewCopy)
            .stage_bytes(b"ours")
            .unwrap();
        let scratch = staged.staging_path().to_owned();
        fs::write(&scratch, "other").unwrap();
        assert!(matches!(
            staged.publish(),
            Err(ArtifactError::Conflict {
                object: "staging file"
            })
        ));
        assert!(!f.path("output").exists());
        // Changed files are not deleted as if still owned staging content.
        assert_eq!(fs::read_to_string(scratch).unwrap(), "other");
    }

    #[test]
    fn saved_inspection_refuses_new_revision_and_undersized_budget() {
        let f = Fixture::new();
        let saved = f
            .plan(None, "output", PublicationMode::NewCopy)
            .stage_bytes(b"original")
            .unwrap()
            .publish()
            .unwrap();
        assert!(matches!(
            saved.inspect_bytes(3),
            Err(ArtifactError::TooLarge { limit: 3 })
        ));
        fs::write(f.path("output"), "new edit").unwrap();
        assert!(matches!(
            saved.inspect_bytes(100),
            Err(ArtifactError::Conflict {
                object: "saved artifact"
            })
        ));
    }

    #[test]
    fn bounded_inputs_and_outputs_are_enforced() {
        let f = Fixture::new();
        fs::write(f.path("source"), "12345").unwrap();
        assert!(matches!(
            ArtifactPlan::prepare(
                &f.0,
                Some(Path::new("source")),
                Path::new("out"),
                PublicationMode::NewCopy,
                4
            ),
            Err(ArtifactError::TooLarge { limit: 4 })
        ));
        let plan = ArtifactPlan::prepare(&f.0, None, Path::new("out"), PublicationMode::NewCopy, 4)
            .unwrap();
        assert!(matches!(
            plan.stage_bytes(b"12345"),
            Err(ArtifactError::TooLarge { limit: 4 })
        ));
        for limit in [0, HARD_MAX_BYTES + 1, usize::MAX] {
            assert!(matches!(
                ArtifactPlan::prepare(
                    &f.0,
                    None,
                    Path::new("out"),
                    PublicationMode::NewCopy,
                    limit
                ),
                Err(ArtifactError::InvalidLimit)
            ));
        }
        assert!(!f.path("out").exists());
    }

    #[test]
    fn parent_traversal_and_outside_scope_are_refused() {
        let f = Fixture::new();
        assert!(matches!(
            ArtifactPlan::prepare(
                Path::new("."),
                None,
                Path::new("out"),
                PublicationMode::NewCopy,
                100
            ),
            Err(ArtifactError::OutsideScope)
        ));
        for path in [
            Path::new("../escape"),
            Path::new("nested/../out"),
            Path::new("/outside"),
        ] {
            assert!(matches!(
                ArtifactPlan::prepare(&f.0, None, path, PublicationMode::NewCopy, 100),
                Err(ArtifactError::OutsideScope)
            ));
        }
    }

    #[cfg(unix)]
    #[test]
    fn symlink_leaf_parent_and_escape_are_refused() {
        use std::os::unix::fs::symlink;
        let f = Fixture::new();
        let other = Fixture::new();
        fs::write(other.path("source"), "outside").unwrap();
        symlink(other.path("source"), f.path("source")).unwrap();
        symlink(&other.0, f.path("alias")).unwrap();
        symlink(other.path("nonexistent"), f.path("dangling")).unwrap();
        assert!(matches!(
            ArtifactPlan::prepare(
                &f.0,
                Some(Path::new("source")),
                Path::new("out"),
                PublicationMode::NewCopy,
                100
            ),
            Err(ArtifactError::Symlink)
        ));
        for output in ["alias/out", "dangling"] {
            assert!(matches!(
                ArtifactPlan::prepare(&f.0, None, Path::new(output), PublicationMode::NewCopy, 100),
                Err(ArtifactError::Symlink)
            ));
        }
        assert!(!other.path("out").exists());
    }

    #[cfg(unix)]
    #[test]
    fn source_inode_replacement_with_identical_bytes_is_a_conflict() {
        let f = Fixture::new();
        fs::write(f.path("source"), "original").unwrap();
        let staged = f
            .plan(Some("source"), "output", PublicationMode::NewCopy)
            .stage_bytes(b"result")
            .unwrap();
        fs::rename(f.path("source"), f.path("old-source")).unwrap();
        fs::write(f.path("source"), "original").unwrap();
        assert!(matches!(
            staged.publish(),
            Err(ArtifactError::Conflict { object: "source" })
        ));
        assert!(!f.path("output").exists());
    }

    #[cfg(unix)]
    #[test]
    fn replaced_parent_fails_without_creating_an_output() {
        let f = Fixture::new();
        fs::create_dir(f.path("parent")).unwrap();
        let staged = f
            .plan(None, "parent/output", PublicationMode::NewCopy)
            .stage_bytes(b"ours")
            .unwrap();
        fs::rename(f.path("parent"), f.path("old-parent")).unwrap();
        fs::create_dir(f.path("parent")).unwrap();
        assert!(matches!(
            staged.publish(),
            Err(ArtifactError::Conflict {
                object: "output parent"
            })
        ));
        assert!(!f.path("parent/output").exists());
        assert!(!f.path("old-parent/output").exists());
    }

    #[cfg(unix)]
    #[test]
    fn ancestor_symlink_retargeting_same_directory_is_refused() {
        use std::os::unix::fs::symlink;
        let f = Fixture::new();
        fs::create_dir_all(f.path("parent/root")).unwrap();
        let root = f.path("parent/root");
        let staged = ArtifactPlan::prepare(
            &root,
            None,
            Path::new("output"),
            PublicationMode::NewCopy,
            100,
        )
        .unwrap()
        .stage_bytes(b"ours")
        .unwrap();
        fs::rename(f.path("parent"), f.path("moved")).unwrap();
        symlink(f.path("moved"), f.path("parent")).unwrap();
        assert!(matches!(staged.publish(), Err(ArtifactError::Symlink)));
        assert!(!f.path("moved/root/output").exists());
    }
    #[test]
    fn video_file_over_buffered_cap_publishes_and_reopens_without_raising_small_artifact_limits() {
        let f = Fixture::new();
        fs::write(f.path("source.html"), "editable").unwrap();
        let mut file = File::create(f.path("render.mp4")).unwrap();
        file.write_all(b"\0\0\0\x18ftypisom").unwrap();
        file.set_len(HARD_MAX_BYTES as u64 + 1024).unwrap();
        drop(file);
        let rendered =
            VideoFileRevision::inspect(&f.0, &f.path("render.mp4"), crate::video::MAX_OUTPUT_BYTES)
                .unwrap();
        assert!(matches!(
            crate::scope::ProjectScope::open(&f.0)
                .unwrap()
                .read_bytes(Path::new("render.mp4"), HARD_MAX_BYTES),
            Err(crate::scope::ScopeError::TooLarge { .. })
        ));
        assert!(matches!(
            ArtifactPlan::prepare(
                &f.0,
                None,
                Path::new("large"),
                PublicationMode::NewCopy,
                HARD_MAX_BYTES + 1
            ),
            Err(ArtifactError::InvalidLimit)
        ));
        let staged = f
            .plan(
                Some("source.html"),
                "published.mp4",
                PublicationMode::NewCopy,
            )
            .stage_video_file(&rendered, &|| true)
            .unwrap();
        let scratch = staged.path.clone();
        let saved = staged.publish(&|| true).unwrap();
        assert_eq!(saved.byte_len, HARD_MAX_BYTES + 1024);
        assert_eq!(saved.revision, rendered.revision);
        assert!(!scratch.exists());
        assert_eq!(
            fs::read_to_string(f.path("source.html")).unwrap(),
            "editable"
        );
        let observed = VideoFileRevision::inspect(
            &f.0,
            &f.path("published.mp4"),
            crate::video::MAX_OUTPUT_BYTES,
        )
        .unwrap();
        assert_eq!(saved.revision, observed.revision);
        fs::write(f.path("published.mp4"), "external edit").unwrap();
        assert!(saved.open_reader().is_err());
    }
    #[test]
    fn streamed_video_refuses_source_destination_and_stage_drift() {
        for changed in ["source", "destination", "stage"] {
            let f = Fixture::new();
            fs::write(f.path("source"), "editable").unwrap();
            fs::write(f.path("render"), "rendered bytes").unwrap();
            let rendered = VideoFileRevision::inspect(&f.0, &f.path("render"), 100).unwrap();
            let staged = f
                .plan(Some("source"), "output", PublicationMode::NewCopy)
                .stage_video_file(&rendered, &|| true)
                .unwrap();
            let stage_path = staged.path.clone();
            match changed {
                "source" => fs::write(f.path("source"), "new source").unwrap(),
                "destination" => fs::write(f.path("output"), "external edit").unwrap(),
                _ => fs::write(&stage_path, "substituted stage").unwrap(),
            }
            assert!(staged.publish(&|| true).is_err());
            if changed == "destination" {
                assert_eq!(
                    fs::read_to_string(f.path("output")).unwrap(),
                    "external edit"
                );
            } else {
                assert!(!f.path("output").exists());
            }
            if changed == "stage" {
                assert_eq!(fs::read_to_string(stage_path).unwrap(), "substituted stage");
            }
        }
    }
    #[test]
    fn video_limits_cancellation_and_partial_stage_are_explicit() {
        let f = Fixture::new();
        File::create(f.path("oversized"))
            .unwrap()
            .set_len(crate::video::MAX_OUTPUT_BYTES as u64 + 1)
            .unwrap();
        assert!(matches!(
            VideoFileRevision::inspect(&f.0, &f.path("oversized"), crate::video::MAX_OUTPUT_BYTES),
            Err(ArtifactError::TooLarge { .. })
        ));
        fs::write(f.path("render"), "rendered").unwrap();
        let rendered = VideoFileRevision::inspect(&f.0, &f.path("render"), 100).unwrap();
        assert!(matches!(
            VideoFileRevision::inspect_checked(&f.0, &f.path("render"), 100, &|| false),
            Err(ArtifactError::VideoStopped)
        ));
        let error = f
            .plan(None, "output", PublicationMode::NewCopy)
            .stage_video_file(&rendered, &|| false)
            .err()
            .unwrap();
        assert!(matches!(error, ArtifactError::VideoStopped));
        assert!(!fs::read_dir(&f.0).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".mirket-stage-")
        }));
        assert!(!f.path("output").exists());
        struct FullDisk;
        impl Write for FullDisk {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::from_raw_os_error(28))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let failure = copy_video_body(&mut io::Cursor::new(b"rendered"), &mut FullDisk, 8, &|| {
            true
        })
        .unwrap_err();
        assert!(
            matches!(failure, ArtifactError::Io { operation: "copy video stage", source } if source.raw_os_error() == Some(28))
        );
    }
    #[cfg(unix)]
    #[test]
    fn video_symlink_and_identical_inode_replacement_are_refused() {
        let f = Fixture::new();
        fs::write(f.path("render"), "rendered").unwrap();
        std::os::unix::fs::symlink(f.path("render"), f.path("link")).unwrap();
        assert!(matches!(
            VideoFileRevision::inspect(&f.0, &f.path("link"), 100),
            Err(ArtifactError::Symlink)
        ));
        let snapshot = VideoFileRevision::inspect(&f.0, &f.path("render"), 100).unwrap();
        fs::rename(f.path("render"), f.path("old-render")).unwrap();
        fs::write(f.path("render"), "rendered").unwrap();
        assert!(snapshot.open_reader().is_err());
    }
    #[cfg(unix)]
    #[test]
    fn cancelled_partial_video_copy_is_removed_and_complete_drop_never_reads_body() {
        use std::sync::atomic::AtomicUsize;
        let f = Fixture::new();
        File::create(f.path("render"))
            .unwrap()
            .set_len(200_000)
            .unwrap();
        let rendered = VideoFileRevision::inspect(&f.0, &f.path("render"), 200_000).unwrap();
        let checks = AtomicUsize::new(0);
        let failed = f
            .plan(None, "output", PublicationMode::NewCopy)
            .stage_video_file(&rendered, &|| checks.fetch_add(1, Ordering::Relaxed) == 0);
        assert!(matches!(failed, Err(ArtifactError::VideoStopped)));
        assert!(!f.path("output").exists());
        assert!(!fs::read_dir(&f.0).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".mirket-stage-")
        }));
        let staged = f
            .plan(None, "output", PublicationMode::NewCopy)
            .stage_video_file(&rendered, &|| true)
            .unwrap();
        let path = staged.path.clone();
        // A drop after the last check must use metadata only. Corrupt the
        // in-memory digest: body rehashing would retain this unchanged inode.
        let mut staged = staged;
        staged.snapshot.revision = "deliberately not a digest".into();
        drop(staged);
        assert!(!path.exists());
    }
    #[cfg(unix)]
    #[test]
    fn partial_disk_write_failure_cleans_exact_prefix_but_retains_substitution() {
        let f = Fixture::new();
        for changed in [false, true] {
            let (path, file) = create_stage(&f.0).unwrap();
            let mut output = VideoStageWriter {
                file,
                hash: Sha256::new(),
                byte_len: 0,
            };
            struct FullAfterPrefix<'a> {
                output: &'a mut VideoStageWriter,
                remaining: usize,
            }
            impl Write for FullAfterPrefix<'_> {
                fn write(&mut self, b: &[u8]) -> io::Result<usize> {
                    if self.remaining == 0 {
                        return Err(io::Error::from_raw_os_error(28));
                    }
                    let n = self.output.write(&b[..b.len().min(self.remaining)])?;
                    self.remaining -= n;
                    Ok(n)
                }
                fn flush(&mut self) -> io::Result<()> {
                    self.output.flush()
                }
            }
            let failure = copy_video_body(
                &mut io::Cursor::new([42; 100]),
                &mut FullAfterPrefix {
                    output: &mut output,
                    remaining: 17,
                },
                100,
                &|| true,
            )
            .unwrap_err();
            assert!(
                matches!(failure,ArtifactError::Io{source,..} if source.raw_os_error()==Some(28))
            );
            if changed {
                fs::write(&path, b"external edit").unwrap();
            }
            let cleanup = cleanup_partial_video_stage(&f.0, &path, &output);
            if changed {
                assert!(cleanup.is_err());
                assert_eq!(fs::read(&path).unwrap(), b"external edit");
            } else {
                cleanup.unwrap();
                assert!(!path.exists());
            }
        }
    }
}
