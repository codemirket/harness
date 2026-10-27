//! Installation-wide soft resident-memory admission, qualified on macOS only.
//!
//! Registered servers and independently registered supervisors are sampled with
//! their descendants; overlapping trees are deduplicated by PID and native birth.
//! RSS sums may double-count shared pages and are neither unique physical RAM nor
//! a reservation of future growth. The finite snapshot is not a hard memory cap.
//! Active work is never killed by this module.
//!
//! The permanent resources/gate.lock serializes registration, observations and
//! heavy acquisition. Two fixed slot bindings identify measurement membership,
//! never cleanup authority. A held slot needs a live lock-backed registration;
//! a free slot with an unresolved marker is unknown even if its old PID is gone.
//! Callers retain ResourceSession at least as long as every lease and cleanup.
//! Private-directory ownership is assumed; checks detect drift, not hostile
//! ancestor swaps. Unknown/stale registration cleanup never clears heavy markers.

use crate::file_lock::FileLock;
use crate::leases::{Admission, AdmissionError, HEAVY_JOB_SLOTS, SlotLease};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, Metadata, OpenOptions, TryLockError};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(target_os = "macos")]
#[path = "resource_admission_macos.rs"]
mod native;
#[cfg(not(target_os = "macos"))]
mod native {
    use super::*;
    pub(super) struct Native;
    pub(super) fn current() -> Result<Identity, Error> {
        unsupported()
    }
    pub(super) fn pressure() -> Result<Pressure, Error> {
        unsupported()
    }
    fn unsupported<T>() -> Result<T, Error> {
        Err(Error::Unknown(
            "native process-tree accounting is not qualified on this platform".into(),
        ))
    }
    impl Sampler for Native {
        fn sample(&self, _: i32) -> Result<NativeSample, SampleError> {
            unsupported().map_err(SampleError::Unknown)
        }
        fn children(&self, _: i32) -> Result<Vec<i32>, SampleError> {
            unsupported().map_err(SampleError::Unknown)
        }
    }
}

pub(crate) const SOFT_LIMIT_BYTES: u64 = 8 * 1024 * 1024 * 1024;
const MAX_REGISTRATIONS: usize = 256;
const MAX_PROCESSES: usize = 4096;
const MAX_ROW_BYTES: u64 = 512;
static NEXT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) enum Pressure {
    Normal,
    Warning,
    Critical,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct Snapshot {
    pub bytes: u64,
    pub process_count: usize,
    pub pressure: Pressure,
}
impl Snapshot {
    pub fn defers(&self) -> bool {
        self.bytes >= SOFT_LIMIT_BYTES || self.pressure != Pressure::Normal
    }
}
#[derive(Debug)]
pub(crate) enum Error {
    Busy,
    Deferred(Snapshot),
    Unknown(String),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Busy => write!(f, "resource admission is busy"),
            Self::Deferred(snapshot) => write!(
                f,
                "heavy admission deferred: {} resident bytes, {:?} pressure",
                snapshot.bytes, snapshot.pressure
            ),
            Self::Unknown(reason) => write!(f, "resource accounting unknown: {reason}"),
        }
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Unknown(error.to_string())
    }
}
fn unknown(message: &str) -> Error {
    Error::Unknown(message.into())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Identity {
    pid: i32,
    seconds: u64,
    micros: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Registration {
    identity: Identity,
    name: String,
}

#[derive(Debug)]
pub(crate) struct ResourceSession {
    state_root: PathBuf,
    registration: Registration,
    lock: FileLock,
}
impl ResourceSession {
    pub fn register(state_root: &Path) -> Result<Self, Error> {
        let identity = native::current()?; // Unsupported platforms cause no effects.
        let until = std::time::Instant::now() + std::time::Duration::from_secs(1);
        let gate = loop {
            match Gate::open(state_root) {
                Err(Error::Busy) if std::time::Instant::now() < until => {
                    std::thread::sleep(std::time::Duration::from_millis(5))
                }
                result => break result?,
            }
        };
        let registrations = gate.registrations()?;
        if registrations.len() >= MAX_REGISTRATIONS {
            return Err(unknown("too many live resource registrations"));
        }
        let name = format!(
            "root-{}-{}-{}-{}.json",
            identity.pid,
            identity.seconds,
            identity.micros,
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let registration = Registration { identity, name };
        let path = gate.resources.join(&registration.name);
        let file = create_private(&path)?;
        file.try_lock()
            .map_err(|error| unknown(&format!("lock new resource registration: {error}")))?;
        let lock = FileLock::acquired(file);
        let mut writer = lock.as_file();
        writer.write_all(&encode(&registration)?)?;
        validate_open(&path, lock.as_file(), MAX_ROW_BYTES)?;
        gate.recheck()?;
        Ok(Self {
            state_root: state_root.to_owned(),
            registration,
            lock,
        })
    }
}
impl Drop for ResourceSession {
    fn drop(&mut self) {
        // Failure leaves an unlocked observation row, ignored/pruned next time.
        // The resource gate never waits for a registration lock, avoiding cycles.
        if let Ok(gate) = Gate::open(&self.state_root) {
            let path = gate.resources.join(&self.registration.name);
            if validate_open(&path, self.lock.as_file(), MAX_ROW_BYTES).is_ok()
                && read_row::<Registration>(self.lock.as_file()).ok().as_ref()
                    == Some(&self.registration)
                && gate.recheck().is_ok()
            {
                let _ = fs::remove_file(path);
            }
        }
    }
}

pub(crate) fn snapshot(state_root: &Path) -> Result<Snapshot, Error> {
    native::current()?;
    let gate = Gate::open(state_root)?;
    let (result, _) = gate.observe(&native::Native, native::pressure()?)?;
    gate.recheck()?;
    Ok(result)
}

pub(crate) fn admit(state_root: &Path) -> Result<SlotLease, Error> {
    let current = native::current()?;
    let gate = Gate::open(state_root)?;
    let rows = gate.registrations()?;
    if !rows.values().any(|row| row.identity == current) {
        return Err(unknown("heavy owner has no live resource registration"));
    }
    // Full slots cannot admit work. Validate ownership first, but don't make
    // every queued 25ms poll resample all process trees while capacity is absent.
    if !gate.check_slots(&rows)? {
        return Err(Error::Busy);
    }

    #[cfg(not(test))]
    let (observed, registrations) = gate.observe(&native::Native, native::pressure()?)?;
    #[cfg(test)]
    let (observed, registrations) = match test_support::observation(state_root) {
        Some(sample) => gate.observe(&sample, sample.pressure)?,
        None => gate.observe(&native::Native, native::pressure()?)?,
    };
    let own = registrations
        .values()
        .find(|row| row.identity == current)
        .ok_or_else(|| unknown("heavy owner has no live resource registration"))?;
    if observed.defers() {
        return Err(Error::Deferred(observed));
    }
    let lease = Admission::open(state_root)
        .map_err(|e| unknown(&e.to_string()))?
        .try_acquire()
        .map_err(|e| match e {
            AdmissionError::Busy => Error::Busy,
            other => unknown(&other.to_string()),
        })?;
    gate.bind(lease.slot_index(), own)?;
    gate.recheck()?;
    Ok(lease)
}

struct Gate {
    root: PathBuf,
    root_metadata: Metadata,
    resources: PathBuf,
    resources_metadata: Metadata,
    lock: FileLock,
}
impl Gate {
    fn open(root: &Path) -> Result<Self, Error> {
        if !root.is_absolute() {
            return Err(unknown("state root must be explicit and absolute"));
        }
        let root_metadata = checked_directory(root)?;
        let resources = root.join("resources");
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(&resources) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e.into()),
        }
        let resources_metadata = checked_directory(&resources)?;
        let path = resources.join("gate.lock");
        let file = open_private(&path, 0)?;
        // Immediate native acquisition. Registration alone retries for <=1s;
        // snapshots/admission return Busy and Drop leaves a safe stale row.
        match file.try_lock() {
            Ok(()) => {}
            Err(TryLockError::WouldBlock) => return Err(Error::Busy),
            Err(TryLockError::Error(error)) => return Err(error.into()),
        }
        let gate = Self {
            root: root.to_owned(),
            root_metadata,
            resources,
            resources_metadata,
            lock: FileLock::acquired(file),
        };
        gate.recheck()?;
        Ok(gate)
    }
    fn recheck(&self) -> Result<(), Error> {
        if !same_file(&self.root_metadata, &checked_directory(&self.root)?)
            || !same_file(
                &self.resources_metadata,
                &checked_directory(&self.resources)?,
            )
        {
            return Err(unknown("resource directory identity changed"));
        }
        validate_open(&self.resources.join("gate.lock"), self.lock.as_file(), 0)
    }
    fn observe(
        &self,
        sampler: &impl Sampler,
        pressure: Pressure,
    ) -> Result<(Snapshot, BTreeMap<String, Registration>), Error> {
        // One bounded retry handles a normal short-lived descendant exiting or
        // a completed supervisor dropping its registration during observation.
        // Never treat an unreadable process as zero; re-enumerate the whole tree.
        for attempt in 0..2 {
            let rows = self.registrations()?;
            self.check_slots(&rows)?;
            let identities: BTreeSet<_> = rows.values().map(|row| row.identity).collect();
            match measure_once(
                sampler,
                &identities.iter().copied().collect::<Vec<_>>(),
                pressure,
            ) {
                Ok(observed) => {
                    let current = self.registrations()?;
                    if current
                        .values()
                        .map(|row| row.identity)
                        .collect::<BTreeSet<_>>()
                        != identities
                    {
                        if attempt == 0 {
                            continue;
                        }
                        return Err(Error::Busy);
                    }
                    self.check_slots(&current)?;
                    self.recheck()?;
                    return Ok((observed, current));
                }
                Err(SampleError::Gone | SampleError::Changed) if attempt == 0 => continue,
                // Churn is an unavailable observation, not an estimate of zero
                // or a claim of bad cleanup. Callers keep their existing finite
                // wait/deadline; no snapshot or heavy admission is returned.
                Err(SampleError::Gone | SampleError::Changed) => return Err(Error::Busy),
                Err(error) => return Err(error.into_error()),
            }
        }
        unreachable!("bounded observation returns on its final attempt")
    }
    fn registrations(&self) -> Result<BTreeMap<String, Registration>, Error> {
        let mut rows = BTreeMap::new();
        let mut count = 0;
        for entry in fs::read_dir(&self.resources)? {
            count += 1;
            if count > MAX_REGISTRATIONS + HEAVY_JOB_SLOTS + 1 {
                return Err(unknown("resource inventory exceeds bound"));
            }
            let entry = entry?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| unknown("invalid resource filename"))?;
            if name == "gate.lock"
                || (0..HEAVY_JOB_SLOTS).any(|slot| name == format!("slot-{slot}.json"))
            {
                continue;
            }
            if !valid_registration_name(&name) {
                return Err(unknown("unknown resource entry"));
            }
            let path = self.resources.join(&name);
            let file = open_existing(&path, MAX_ROW_BYTES)?;
            // Decode and bind even unlocked rows before deleting anything. An
            // unknown or interrupted write is preserved, not guessed stale.
            let row: Registration = read_row(&file)?;
            let prefix = format!(
                "root-{}-{}-{}-",
                row.identity.pid, row.identity.seconds, row.identity.micros
            );
            let suffix = name
                .strip_prefix(&prefix)
                .and_then(|value| value.strip_suffix(".json"));
            if row.name != name
                || row.identity.pid <= 0
                || row.identity.seconds == 0
                || row.identity.micros >= 1_000_000
                || !suffix.is_some_and(|value| {
                    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
                })
            {
                return Err(unknown("invalid resource registration"));
            }
            validate_open(&path, &file, MAX_ROW_BYTES)?;
            match file.try_lock() {
                Ok(()) => {
                    let stale = FileLock::acquired(file);
                    validate_open(&path, stale.as_file(), MAX_ROW_BYTES)?;
                    // Unlocked rows cannot authorize measurement membership. The
                    // exact owned file can be removed without touching a marker.
                    fs::remove_file(path)?;
                }
                Err(TryLockError::WouldBlock) => {
                    rows.insert(name, row);
                }
                Err(TryLockError::Error(error)) => return Err(error.into()),
            }
        }
        Ok(rows)
    }
    fn check_slots(&self, rows: &BTreeMap<String, Registration>) -> Result<bool, Error> {
        self.check_slots_before_lock(rows, |_| {})
    }
    // The internal callback controls the release/acquisition boundary in tests;
    // production supplies only the no-op above and still uses real native locks.
    fn check_slots_before_lock(
        &self,
        rows: &BTreeMap<String, Registration>,
        mut before_lock: impl FnMut(usize),
    ) -> Result<bool, Error> {
        let mut has_free = false;
        for slot in 0..HEAVY_JOB_SLOTS {
            let marker = self.root.join(format!("heavy-slot-{slot}.unresolved"));
            let path = self.root.join(format!("heavy-slot-{slot}.lock"));
            let file = open_private(&path, 0)?;
            before_lock(slot);
            match file.try_lock() {
                Ok(()) => {
                    let lock = FileLock::acquired(file);
                    validate_open(&path, lock.as_file(), 0)?;
                    // Read only after acquiring the free slot: a prior owner
                    // can legitimately clear its marker immediately before
                    // unlocking. Historical marker existence proves nothing.
                    match fs::symlink_metadata(&marker) {
                        Ok(metadata) => {
                            validate_metadata(&metadata, MAX_ROW_BYTES)?;
                            return Err(unknown(
                                "unresolved heavy ownership has no held native slot",
                            ));
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                        Err(error) => return Err(error.into()),
                    }
                    has_free = true;
                }
                Err(TryLockError::WouldBlock) => {
                    // Pre-launch owners may not have a marker yet. An existing
                    // marker must still have the private regular-file shape.
                    match fs::symlink_metadata(&marker) {
                        Ok(metadata) => validate_metadata(&metadata, MAX_ROW_BYTES)?,
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                        Err(error) => return Err(error.into()),
                    }
                    let binding = open_existing(
                        &self.resources.join(format!("slot-{slot}.json")),
                        MAX_ROW_BYTES,
                    )?;
                    let identity: Identity = read_row(&binding)?;
                    if !rows.values().any(|row| row.identity == identity) {
                        return Err(unknown("held heavy slot lacks a live registered owner"));
                    }
                }
                Err(TryLockError::Error(error)) => return Err(error.into()),
            }
        }
        Ok(has_free)
    }
    fn bind(&self, slot: usize, row: &Registration) -> Result<(), Error> {
        let path = self.resources.join(format!("slot-{slot}.json"));
        let mut file = open_private(&path, MAX_ROW_BYTES)?;
        let bytes = serde_json::to_vec(&row.identity).map_err(|e| unknown(&e.to_string()))?;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(&bytes)?;
        file.set_len(bytes.len() as u64)?;
        validate_open(&path, &file, MAX_ROW_BYTES)
    }
}

fn valid_registration_name(name: &str) -> bool {
    name.starts_with("root-")
        && name.ends_with(".json")
        && name.len() <= 160
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.')
}
fn encode(row: &Registration) -> Result<Vec<u8>, Error> {
    let bytes = serde_json::to_vec(row).map_err(|e| unknown(&e.to_string()))?;
    if bytes.len() as u64 > MAX_ROW_BYTES {
        return Err(unknown("resource row exceeds bound"));
    }
    Ok(bytes)
}
fn read_row<T: serde::de::DeserializeOwned>(file: &File) -> Result<T, Error> {
    let mut reader = file;
    reader.seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::new();
    reader.take(MAX_ROW_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_ROW_BYTES {
        return Err(unknown("resource row exceeds bound"));
    }
    serde_json::from_slice(&bytes).map_err(|e| unknown(&format!("invalid resource row: {e}")))
}
fn create_private(path: &Path) -> Result<File, Error> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    Ok(options.open(path)?)
}
fn open_private(path: &Path, max: u64) -> Result<File, Error> {
    let file = match create_private(path) {
        Ok(file) => file,
        Err(Error::Unknown(_)) if path.try_exists()? => return open_existing(path, max),
        Err(error) => return Err(error),
    };
    validate_open(path, &file, max)?;
    Ok(file)
}
fn open_existing(path: &Path, max: u64) -> Result<File, Error> {
    validate_metadata(&fs::symlink_metadata(path)?, max)?;
    let file = OpenOptions::new().read(true).write(true).open(path)?;
    validate_open(path, &file, max)?;
    Ok(file)
}
fn checked_directory(path: &Path) -> Result<Metadata, Error> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(unknown("resource directory is not a regular directory"));
    }
    private_owner(&metadata)?;
    Ok(metadata)
}
fn private_owner(metadata: &Metadata) -> Result<(), Error> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.uid() != unsafe { libc::geteuid() } || metadata.mode() & 0o077 != 0 {
            return Err(unknown("resource state requires private user ownership"));
        }
    }
    #[cfg(not(unix))]
    let _ = metadata;
    Ok(())
}
fn validate_metadata(metadata: &Metadata, max: u64) -> Result<(), Error> {
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > max {
        return Err(unknown("invalid resource file type or size"));
    }
    private_owner(metadata)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 {
            return Err(unknown("aliased resource file"));
        }
    }
    Ok(())
}
fn validate_open(path: &Path, file: &File, max: u64) -> Result<(), Error> {
    let saved = fs::symlink_metadata(path)?;
    let opened = file.metadata()?;
    validate_metadata(&saved, max)?;
    validate_metadata(&opened, max)?;
    if !same_file(&saved, &opened) {
        return Err(unknown("resource file identity changed"));
    }
    Ok(())
}
fn same_file(a: &Metadata, b: &Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        (a.dev(), a.ino()) == (b.dev(), b.ino())
    }
    #[cfg(not(unix))]
    {
        let _ = (a, b);
        false
    }
}

#[derive(Clone, Copy, Debug)]
struct NativeSample {
    identity: Identity,
    parent: i32,
    bytes: u64,
}
#[derive(Debug)]
enum SampleError {
    Gone,
    Changed,
    Unknown(Error),
}
impl SampleError {
    fn into_error(self) -> Error {
        match self {
            Self::Gone => unknown("a process disappeared during bounded observation"),
            Self::Changed => unknown("process tree changed during bounded observation"),
            Self::Unknown(error) => error,
        }
    }
}
impl From<Error> for SampleError {
    fn from(error: Error) -> Self {
        Self::Unknown(error)
    }
}
trait Sampler {
    fn sample(&self, pid: i32) -> Result<NativeSample, SampleError>;
    fn children(&self, pid: i32) -> Result<Vec<i32>, SampleError>;
}

/// Protocol/lifecycle fixtures keep real registration, gate and lease checks,
/// while replacing only the external process-tree/pressure observation. The
/// canonical root and guard lifetime confine each override to its own fixture;
/// no environment variable or process-wide default affects concurrent tests.
#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static OBSERVATIONS: OnceLock<Mutex<BTreeMap<PathBuf, Sample>>> = OnceLock::new();

    #[derive(Clone, Copy)]
    pub(super) struct Sample {
        identity: Identity,
        bytes: u64,
        pub(super) pressure: Pressure,
    }
    impl Sampler for Sample {
        fn sample(&self, pid: i32) -> Result<NativeSample, SampleError> {
            if pid != self.identity.pid {
                return Err(SampleError::Gone);
            }
            Ok(NativeSample {
                identity: self.identity,
                parent: 0,
                bytes: self.bytes,
            })
        }
        fn children(&self, pid: i32) -> Result<Vec<i32>, SampleError> {
            if pid != self.identity.pid {
                return Err(SampleError::Gone);
            }
            Ok(vec![])
        }
    }

    pub(crate) struct Observation {
        root: PathBuf,
    }
    impl Observation {
        pub(crate) fn normal(state_root: &Path) -> Self {
            Self::new(state_root, 1024 * 1024, Pressure::Normal)
        }
        pub(crate) fn new(state_root: &Path, bytes: u64, pressure: Pressure) -> Self {
            let root = state_root.canonicalize().unwrap();
            let sample = Sample {
                identity: native::current().unwrap(),
                bytes,
                pressure,
            };
            let mut observations = OBSERVATIONS.get_or_init(Default::default).lock().unwrap();
            let inserted = match observations.entry(root.clone()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(sample);
                    true
                }
                std::collections::btree_map::Entry::Occupied(_) => false,
            };
            drop(observations);
            assert!(inserted, "resource observation already owned by a fixture");
            Self { root }
        }
    }
    impl Drop for Observation {
        fn drop(&mut self) {
            OBSERVATIONS
                .get()
                .unwrap()
                .lock()
                .unwrap()
                .remove(&self.root);
        }
    }

    pub(super) fn observation(state_root: &Path) -> Option<Sample> {
        let root = state_root.canonicalize().ok()?;
        OBSERVATIONS.get()?.lock().unwrap().get(&root).copied()
    }
}

#[cfg(test)]
fn measure(
    sampler: &impl Sampler,
    roots: &[Identity],
    pressure: Pressure,
) -> Result<Snapshot, Error> {
    measure_once(sampler, roots, pressure).map_err(SampleError::into_error)
}
fn measure_once(
    sampler: &impl Sampler,
    roots: &[Identity],
    pressure: Pressure,
) -> Result<Snapshot, SampleError> {
    let mut root_map = BTreeMap::new();
    for identity in roots {
        if root_map
            .insert(identity.pid, *identity)
            .is_some_and(|previous| previous != *identity)
        {
            return Err(unknown("registered PID has conflicting birth identities").into());
        }
    }
    let roots = root_map;
    let mut pending: Vec<_> = roots
        .values()
        .map(|identity| (identity.pid, None))
        .collect();
    let mut samples = BTreeMap::<i32, (NativeSample, Vec<i32>)>::new();
    while let Some((pid, parent)) = pending.pop() {
        let sample = sampler.sample(pid)?;
        if sample.identity.pid != pid
            || roots
                .get(&pid)
                .is_some_and(|expected| *expected != sample.identity)
        {
            return Err(unknown("registered process birth identity differs").into());
        }
        if parent.is_some_and(|expected| sample.parent != expected) {
            return Err(SampleError::Changed);
        }
        if let Some((previous, _)) = samples.get(&pid) {
            if previous.identity != sample.identity {
                return Err(unknown("PID birth changed during tree observation").into());
            }
            continue;
        }
        if samples.len() >= MAX_PROCESSES {
            return Err(unknown("process tree exceeds observation bound").into());
        }
        let children = sampler.children(pid)?;
        if children.len() > MAX_PROCESSES
            || children.iter().any(|child| *child <= 0 || *child == pid)
        {
            return Err(unknown("invalid child inventory").into());
        }
        pending.extend(children.iter().map(|child| (*child, Some(pid))));
        samples.insert(pid, (sample, children));
    }
    let mut bytes = 0u64;
    for (pid, (before, children)) in &samples {
        let after = sampler.sample(*pid)?;
        let after_children = sampler.children(*pid)?;
        if after.identity != before.identity
            || after.parent != before.parent
            || children.iter().collect::<BTreeSet<_>>()
                != after_children.iter().collect::<BTreeSet<_>>()
        {
            return Err(SampleError::Changed);
        }
        bytes = bytes
            .checked_add(after.bytes)
            .ok_or_else(|| unknown("resident memory sum overflow"))?;
    }
    Ok(Snapshot {
        bytes,
        process_count: samples.len(),
        pressure,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct Fake {
        samples: BTreeMap<i32, NativeSample>,
        children: BTreeMap<i32, Vec<i32>>,
        change: Option<NativeSample>,
        calls: Cell<usize>,
    }
    fn identity(pid: i32) -> Identity {
        Identity {
            pid,
            seconds: 100,
            micros: 1,
        }
    }
    fn fake() -> Fake {
        Fake {
            samples: [
                (
                    1,
                    NativeSample {
                        identity: identity(1),
                        parent: 0,
                        bytes: SOFT_LIMIT_BYTES - 101,
                    },
                ),
                (
                    2,
                    NativeSample {
                        identity: identity(2),
                        parent: 1,
                        bytes: 100,
                    },
                ),
            ]
            .into(),
            children: [(1, vec![2]), (2, vec![])].into(),
            change: None,
            calls: Cell::new(0),
        }
    }
    impl Sampler for Fake {
        fn sample(&self, pid: i32) -> Result<NativeSample, SampleError> {
            let call = self.calls.get();
            self.calls.set(call + 1);
            if call >= 2 && self.change.is_some_and(|sample| sample.identity.pid == pid) {
                return Ok(self.change.unwrap());
            }
            self.samples.get(&pid).copied().ok_or(SampleError::Gone)
        }
        fn children(&self, pid: i32) -> Result<Vec<i32>, SampleError> {
            Ok(self.children.get(&pid).cloned().unwrap_or_default())
        }
    }

    #[test]
    fn deduplicates_overlapping_roots_and_defers_at_eight_gib() {
        let observed = measure(
            &fake(),
            &[identity(1), identity(2), identity(1)],
            Pressure::Normal,
        )
        .unwrap();
        assert_eq!(observed.process_count, 2);
        assert_eq!(observed.bytes, SOFT_LIMIT_BYTES - 1);
        assert!(!observed.defers());
        let mut over = fake();
        over.samples.get_mut(&2).unwrap().bytes += 1;
        assert!(
            measure(&over, &[identity(1)], Pressure::Normal)
                .unwrap()
                .defers()
        );
        for pressure in [Pressure::Warning, Pressure::Critical] {
            assert!(measure(&fake(), &[identity(1)], pressure).unwrap().defers());
        }
    }

    #[test]
    fn reused_pid_or_reparented_child_is_unknown_not_zero_memory() {
        let wrong = Identity {
            seconds: 101,
            ..identity(1)
        };
        assert!(matches!(
            measure(&fake(), &[wrong], Pressure::Normal),
            Err(Error::Unknown(_))
        ));
        assert!(matches!(
            measure(&fake(), &[identity(1), wrong], Pressure::Normal),
            Err(Error::Unknown(_))
        ));
        for changed in [
            NativeSample {
                identity: Identity {
                    seconds: 101,
                    ..identity(2)
                },
                parent: 1,
                bytes: 1,
            },
            NativeSample {
                identity: identity(2),
                parent: 99,
                bytes: 1,
            },
        ] {
            let mut fixture = fake();
            fixture.change = Some(changed);
            assert!(matches!(
                measure(&fixture, &[identity(1)], Pressure::Normal),
                Err(Error::Unknown(_))
            ));
        }
    }

    #[cfg(target_os = "macos")]
    struct Temp(PathBuf);
    #[cfg(target_os = "macos")]
    impl Temp {
        fn new() -> Self {
            use std::os::unix::fs::DirBuilderExt;
            let path = std::env::temp_dir().join(format!(
                "mirket-resource-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
            Self(path)
        }
    }
    #[cfg(target_os = "macos")]
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn fixture_observations_are_root_scoped_and_removed_with_the_guard() {
        let first = Temp::new();
        let second = Temp::new();
        let _first_session = ResourceSession::register(&first.0).unwrap();
        let _second_session = ResourceSession::register(&second.0).unwrap();
        let first_observation = test_support::Observation::normal(&first.0);
        let second_observation = test_support::Observation::new(&second.0, 1, Pressure::Warning);
        let slot = admit(&first.0).unwrap();
        assert!(matches!(
            admit(&second.0),
            Err(Error::Deferred(Snapshot {
                pressure: Pressure::Warning,
                bytes: 1,
                process_count: 1,
            }))
        ));
        drop(second_observation);
        assert!(test_support::observation(&second.0).is_none());
        assert!(test_support::observation(&first.0).is_some());
        drop(slot);
        drop(first_observation);
        assert!(test_support::observation(&first.0).is_none());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn injected_observations_still_defer_pressure_and_eight_gib() {
        let temp = Temp::new();
        let _session = ResourceSession::register(&temp.0).unwrap();
        for (bytes, pressure) in [
            (1, Pressure::Warning),
            (1, Pressure::Critical),
            (SOFT_LIMIT_BYTES, Pressure::Normal),
        ] {
            let _observation = test_support::Observation::new(&temp.0, bytes, pressure);
            assert!(matches!(admit(&temp.0), Err(Error::Deferred(observed))
                if observed == Snapshot { bytes, pressure, process_count: 1 }));
        }
        let _observation =
            test_support::Observation::new(&temp.0, SOFT_LIMIT_BYTES - 1, Pressure::Normal);
        drop(admit(&temp.0).unwrap());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn fixture_observation_cannot_bypass_registration_slots_or_unresolved_markers() {
        let temp = Temp::new();
        let _observation = test_support::Observation::normal(&temp.0);
        assert!(matches!(admit(&temp.0), Err(Error::Unknown(_))));
        let _session = ResourceSession::register(&temp.0).unwrap();
        let mut first = admit(&temp.0).unwrap();
        let second = admit(&temp.0).unwrap();
        assert!(matches!(admit(&temp.0), Err(Error::Busy)));
        first
            .mark_unresolved("fixture-unconfirmed-cleanup")
            .unwrap();
        let marker = temp
            .0
            .join(format!("heavy-slot-{}.unresolved", first.slot_index()));
        drop(first);
        drop(second);
        assert!(matches!(admit(&temp.0), Err(Error::Unknown(_))));
        assert!(marker.exists());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn exited_descendant_reenumerates_but_unreadable_process_never_becomes_zero() {
        struct Exiting {
            root: Identity,
            gone: Cell<bool>,
            unreadable: bool,
        }
        impl Sampler for Exiting {
            fn sample(&self, pid: i32) -> Result<NativeSample, SampleError> {
                if pid != self.root.pid {
                    self.gone.set(true);
                    return if self.unreadable {
                        Err(unknown("permission denied").into())
                    } else {
                        Err(SampleError::Gone)
                    };
                }
                Ok(NativeSample {
                    identity: self.root,
                    parent: 1,
                    bytes: 12,
                })
            }
            fn children(&self, _: i32) -> Result<Vec<i32>, SampleError> {
                Ok(if self.gone.get() {
                    vec![]
                } else {
                    vec![i32::MAX]
                })
            }
        }
        let temp = Temp::new();
        let session = ResourceSession::register(&temp.0).unwrap();
        let gate = Gate::open(&temp.0).unwrap();
        let sampler = Exiting {
            root: session.registration.identity,
            gone: Cell::new(false),
            unreadable: false,
        };
        let (snapshot, _) = gate.observe(&sampler, Pressure::Normal).unwrap();
        assert_eq!((snapshot.bytes, snapshot.process_count), (12, 1));
        let sampler = Exiting {
            unreadable: true,
            gone: Cell::new(false),
            ..sampler
        };
        assert!(matches!(
            gate.observe(&sampler, Pressure::Normal),
            Err(Error::Unknown(_))
        ));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn persistent_transient_observation_is_busy_but_authority_errors_stay_unknown() {
        #[derive(Clone, Copy)]
        enum Failure {
            Gone,
            Changed,
            Unreadable,
            WrongBirth,
        }
        struct Unstable {
            identity: Identity,
            failure: Failure,
            calls: Cell<usize>,
        }
        impl Sampler for Unstable {
            fn sample(&self, _: i32) -> Result<NativeSample, SampleError> {
                self.calls.set(self.calls.get() + 1);
                match self.failure {
                    Failure::Gone => Err(SampleError::Gone),
                    Failure::Changed => Err(SampleError::Changed),
                    Failure::Unreadable => Err(unknown("permission denied").into()),
                    Failure::WrongBirth => Ok(NativeSample {
                        identity: Identity {
                            seconds: self.identity.seconds + 1,
                            ..self.identity
                        },
                        parent: 1,
                        bytes: 42,
                    }),
                }
            }
            fn children(&self, _: i32) -> Result<Vec<i32>, SampleError> {
                Ok(vec![])
            }
        }
        let temp = Temp::new();
        let session = ResourceSession::register(&temp.0).unwrap();
        let gate = Gate::open(&temp.0).unwrap();
        for failure in [
            Failure::Gone,
            Failure::Changed,
            Failure::Unreadable,
            Failure::WrongBirth,
        ] {
            let sampler = Unstable {
                identity: session.registration.identity,
                failure,
                calls: Cell::new(0),
            };
            let observed = gate.observe(&sampler, Pressure::Normal);
            if matches!(failure, Failure::Gone | Failure::Changed) {
                assert!(matches!(observed, Err(Error::Busy)));
                assert_eq!(sampler.calls.get(), 2, "one bounded rescan only");
            } else {
                assert!(matches!(observed, Err(Error::Unknown(_))));
                assert_eq!(
                    sampler.calls.get(),
                    1,
                    "authority errors must not retry to zero"
                );
            }
        }
        // These observation failures acquired no heavy lease and made no marker.
        let admission = Admission::open(&temp.0).unwrap();
        let _first = admission.try_acquire().unwrap();
        let _second = admission.try_acquire().unwrap();
        assert!(!temp.0.join("heavy-slot-0.unresolved").exists());
        assert!(!temp.0.join("heavy-slot-1.unresolved").exists());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn repeated_registration_departure_returns_busy_after_one_rescan() {
        use std::cell::RefCell;
        struct Departing {
            locks: RefCell<BTreeMap<i32, FileLock>>,
            calls: Cell<usize>,
        }
        impl Sampler for Departing {
            fn sample(&self, pid: i32) -> Result<NativeSample, SampleError> {
                let call = self.calls.get() + 1;
                self.calls.set(call);
                // First pass observes both identities, then loses registration2.
                // Second pass observes identity1, then loses that registration.
                if call == 1 {
                    self.locks.borrow_mut().remove(&2);
                }
                if call == 5 {
                    self.locks.borrow_mut().remove(&1);
                }
                Ok(NativeSample {
                    identity: identity(pid),
                    parent: 0,
                    bytes: 12,
                })
            }
            fn children(&self, _: i32) -> Result<Vec<i32>, SampleError> {
                Ok(vec![])
            }
        }
        let temp = Temp::new();
        let gate = Gate::open(&temp.0).unwrap();
        let mut locks = BTreeMap::new();
        for pid in [1, 2] {
            // Synthetic native identities stay within this injected sampler;
            // the registration files and their lifetime locks remain real.
            let row = Registration {
                identity: identity(pid),
                name: format!("root-{pid}-100-1-0.json"),
            };
            let mut file = create_private(&gate.resources.join(&row.name)).unwrap();
            file.write_all(&encode(&row).unwrap()).unwrap();
            file.try_lock().unwrap();
            locks.insert(pid, FileLock::acquired(file));
        }
        let sampler = Departing {
            locks: RefCell::new(locks),
            calls: Cell::new(0),
        };
        assert!(matches!(
            gate.observe(&sampler, Pressure::Normal),
            Err(Error::Busy)
        ));
        assert_eq!(sampler.calls.get(), 6);
        assert!(gate.registrations().unwrap().is_empty());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn slot_membership_requires_live_registration_and_never_reclaims_marker() {
        let temp = Temp::new();
        let session = ResourceSession::register(&temp.0).unwrap();
        let gate = Gate::open(&temp.0).unwrap();
        let rows = gate.registrations().unwrap();
        let admission = Admission::open(&temp.0).unwrap();
        let mut lease = admission.try_acquire().unwrap();
        assert!(matches!(gate.check_slots(&rows), Err(Error::Unknown(_))));
        gate.bind(lease.slot_index(), &session.registration)
            .unwrap();
        gate.check_slots(&rows).unwrap();
        lease.mark_unresolved("owned-test-job").unwrap();
        gate.check_slots(&rows).unwrap(); // Normal active marker is not unknown.
        drop(gate);
        // Dropping one same-process guard must not invalidate another live guard.
        let second = ResourceSession::register(&temp.0).unwrap();
        drop(session);
        let gate = Gate::open(&temp.0).unwrap();
        gate.check_slots(&gate.registrations().unwrap()).unwrap();
        drop(lease); // Simulate lost cleanup; no PID inference can clear marker.
        assert!(matches!(
            gate.check_slots(&gate.registrations().unwrap()),
            Err(Error::Unknown(_))
        ));
        assert!(temp.0.join("heavy-slot-0.unresolved").exists());
        drop(gate);
        drop(second);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn completed_owner_handoff_does_not_reuse_historical_marker_state() {
        let temp = Temp::new();
        let session = ResourceSession::register(&temp.0).unwrap();
        let gate = Gate::open(&temp.0).unwrap();
        let rows = gate.registrations().unwrap();
        let mut lease = Admission::open(&temp.0).unwrap().try_acquire().unwrap();
        let slot = lease.slot_index();
        gate.bind(slot, &session.registration).unwrap();
        lease.mark_unresolved("completed-handoff").unwrap();
        let marker = temp.0.join(format!("heavy-slot-{slot}.unresolved"));
        assert!(marker.exists());
        let mut owner = Some(lease);
        let available = gate
            .check_slots_before_lock(&rows, |observed_slot| {
                if observed_slot == slot {
                    let mut lease = owner.take().unwrap();
                    lease.clear_after_cleanup().unwrap();
                    drop(lease);
                }
            })
            .unwrap();
        assert!(available);
        assert!(owner.is_none());
        assert!(!marker.exists());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn dropped_registration_with_held_slot_and_contended_gate_fail_closed() {
        let temp = Temp::new();
        let session = ResourceSession::register(&temp.0).unwrap();
        let gate = Gate::open(&temp.0).unwrap();
        let lease = Admission::open(&temp.0).unwrap().try_acquire().unwrap();
        gate.bind(lease.slot_index(), &session.registration)
            .unwrap();
        assert!(matches!(Gate::open(&temp.0), Err(Error::Busy)));
        let path = gate.resources.join(&session.registration.name);
        drop(session); // Gate contention leaves a safe unlocked stale row.
        assert!(path.exists());
        let rows = gate.registrations().unwrap();
        assert!(rows.is_empty());
        assert!(!path.exists());
        assert!(matches!(gate.check_slots(&rows), Err(Error::Unknown(_))));
        drop(lease);
        gate.check_slots(&rows).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn resource_aliases_and_unbounded_rows_are_rejected() {
        use std::os::unix::fs::symlink;
        let temp = Temp::new();
        let outside = Temp::new();
        symlink(&outside.0, temp.0.join("resources")).unwrap();
        assert!(matches!(
            ResourceSession::register(&temp.0),
            Err(Error::Unknown(_))
        ));
        fs::remove_file(temp.0.join("resources")).unwrap();
        let gate = Gate::open(&temp.0).unwrap();
        let path = gate.resources.join("root-1.json");
        let mut row = create_private(&path).unwrap();
        row.write_all(&vec![b'x'; MAX_ROW_BYTES as usize + 1])
            .unwrap();
        assert!(matches!(gate.registrations(), Err(Error::Unknown(_))));
        fs::remove_file(&path).unwrap();
        let mut row = create_private(&path).unwrap();
        row.write_all(b"{\"unknown\":true}").unwrap();
        assert!(matches!(gate.registrations(), Err(Error::Unknown(_))));
        assert!(path.exists());
        fs::remove_file(&path).unwrap();
        fs::hard_link(gate.resources.join("gate.lock"), &path).unwrap();
        assert!(matches!(gate.registrations(), Err(Error::Unknown(_))));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn native_sampler_counts_an_owned_sleep_child_without_other_test_children() {
        struct Owned(std::process::Child);
        impl Drop for Owned {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let child = Owned(
            std::process::Command::new("/bin/sleep")
                .arg("10")
                .spawn()
                .unwrap(),
        );
        let sample = native::Native.sample(child.0.id() as i32).unwrap();
        let observed = measure(
            &native::Native,
            &[sample.identity],
            native::pressure().unwrap(),
        )
        .unwrap();
        assert_eq!(observed.process_count, 1);
        assert!(observed.bytes > 0);
    }
}
