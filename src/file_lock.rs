//! Lifetime ownership for an already acquired, independently opened native lock.
//! Close alone can retain a Unix lock in an unrelated forked child's descriptor.
//! This owner explicitly unlocks before closing; it adds no acquisition policy.
use std::fs::File;

#[derive(Debug)]
pub(crate) struct FileLock(File);

impl FileLock {
    /// Wrap immediately after successful acquisition, before fallible validation.
    /// Do not clone the descriptor or transfer this owner across a fork. Real
    /// child owners must open their own lock; pre-exec children must not drop it.
    pub(crate) fn acquired(file: File) -> Self {
        Self(file)
    }

    pub(crate) fn as_file(&self) -> &File {
        &self.0
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        // Drop cannot report failure. Closing still follows, and a failed unlock
        // conservatively retains exclusion until all inherited copies close.
        let _ = self.0.unlock();
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::fs::{self, OpenOptions, TryLockError};
    use std::os::unix::fs::OpenOptionsExt;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn duplicate_description_does_not_extend_owner_or_release_independent_owner() {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "mirket-file-lock-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let open = || {
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)
                .unwrap()
        };
        let first = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .unwrap();
        first.try_lock_shared().unwrap();
        let first = FileLock::acquired(first);
        // dup and fork share the same Unix open file description. Keeping a
        // duplicate here controls the lifetime without forking the test harness.
        let duplicate = first.as_file().try_clone().unwrap();
        let second = open();
        second.try_lock_shared().unwrap();
        let second = FileLock::acquired(second);
        let contender = open();
        drop(first);
        assert!(matches!(
            contender.try_lock(),
            Err(TryLockError::WouldBlock)
        ));
        drop(second);
        contender.try_lock().unwrap();
        drop(FileLock::acquired(contender));
        drop(duplicate);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn close_only_control_retains_lock_until_duplicate_closes() {
        let path =
            std::env::temp_dir().join(format!("mirket-close-only-lock-{}", std::process::id()));
        let owner = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .unwrap();
        owner.try_lock().unwrap();
        let duplicate = owner.try_clone().unwrap();
        let contender = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        drop(owner);
        assert!(matches!(
            contender.try_lock(),
            Err(TryLockError::WouldBlock)
        ));
        drop(duplicate);
        contender.try_lock().unwrap();
        drop(FileLock::acquired(contender));
        fs::remove_file(path).unwrap();
    }
}
