//! Owned empty scratch for a finite worker or worker session. The lifecycle owner retains
//! this value until confirmed process cleanup; dropping it never deletes files.
use std::{
    fs,
    path::{Component, Path, PathBuf},
    time::SystemTime,
};

pub(crate) fn explicit_path(path: &Path, directory: bool) -> Result<fs::Metadata, String> {
    if !path.is_absolute()
        || path
            .components()
            .any(|p| matches!(p, Component::ParentDir | Component::CurDir))
    {
        return Err("worker paths must be explicit, absolute and normalized".into());
    }
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "worker configured path is unavailable")?;
    if metadata.file_type().is_symlink()
        || (if directory {
            !metadata.is_dir()
        } else {
            !metadata.is_file()
        })
        || fs::canonicalize(path).map_err(|_| "worker path cannot be resolved")? != path
    {
        return Err("worker paths must be canonical and have the expected file type".into());
    }
    Ok(metadata)
}

#[derive(Clone)]
pub(crate) struct DirectoryStamp {
    #[cfg(unix)]
    native: (u64, u64, u32),
    created: Option<SystemTime>,
}
impl DirectoryStamp {
    pub(crate) fn inspect(path: &Path) -> Result<Self, String> {
        let metadata = explicit_path(path, true)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            // SAFETY: geteuid has no arguments or memory ownership effects.
            if metadata.mode() & 0o077 != 0 || metadata.uid() != unsafe { libc::geteuid() } {
                return Err(
                    "worker state and scratch directories must be private and user-owned".into(),
                );
            }
        }
        Ok(Self {
            #[cfg(unix)]
            native: {
                use std::os::unix::fs::MetadataExt;
                (metadata.dev(), metadata.ino(), metadata.uid())
            },
            created: metadata.created().ok(),
        })
    }
    pub(crate) fn matches(&self, path: &Path) -> Result<(), String> {
        let current = Self::inspect(path)?;
        #[cfg(unix)]
        if current.native == self.native && current.created == self.created {
            return Ok(());
        }
        #[cfg(not(unix))]
        let _ = current;
        Err("worker directory ownership changed".into())
    }
}
pub(crate) struct OwnedScratch {
    pub(crate) path: PathBuf,
    state_root: PathBuf,
    state_stamp: DirectoryStamp,
    jobs_stamp: DirectoryStamp,
    stamp: DirectoryStamp,
}
impl OwnedScratch {
    pub(crate) fn create(
        state_root: &Path,
        state_stamp: &DirectoryStamp,
        job_id: &str,
    ) -> Result<Self, String> {
        if job_id.is_empty()
            || job_id.len() > 128
            || !job_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        {
            return Err("invalid worker scratch identity".into());
        }
        state_stamp.matches(state_root)?;
        let jobs = state_root.join("jobs");
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(&jobs) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
            Err(_) => return Err("worker jobs directory cannot be created".into()),
        }
        let jobs_stamp = DirectoryStamp::inspect(&jobs)?;
        let path = jobs.join(job_id);
        builder
            .create(&path)
            .map_err(|_| "worker job directory reservation failed; existing data retained")?;
        let stamp = DirectoryStamp::inspect(&path)?;
        Ok(Self {
            path,
            state_root: state_root.to_path_buf(),
            state_stamp: state_stamp.clone(),
            jobs_stamp,
            stamp,
        })
    }
    pub(crate) fn remove_empty(self) -> Result<(), String> {
        self.state_stamp.matches(&self.state_root)?;
        self.jobs_stamp.matches(&self.state_root.join("jobs"))?;
        self.stamp.matches(&self.path)?;
        let parent = fs::File::open(self.state_root.join("jobs"))
            .map_err(|_| "owned job parent cannot be opened for cleanup".to_owned())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = parent
                .metadata()
                .map_err(|_| "owned job parent identity unavailable".to_owned())?;
            if self.jobs_stamp.native != (metadata.dev(), metadata.ino(), metadata.uid())
                || self.jobs_stamp.created != metadata.created().ok()
                || metadata.mode() & 0o077 != 0
            {
                return Err("owned job parent identity changed".into());
            }
        }
        // remove_dir itself refuses nonempty directories. Do not enumerate,
        // remove children, adopt a preexisting path or infer ownership by name.
        fs::remove_dir(&self.path)
            .map_err(|_| "owned job directory is not empty or cannot be removed".to_owned())?;
        parent
            .sync_all()
            .map_err(|_| "owned job directory removal could not be flushed".to_owned())
    }
}
