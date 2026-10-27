//! Explicit project selection and bounded source reads.
//!
//! There is no current-directory or global-project fallback. These canonical
//! path checks reject detected symlinks and traversal, but path-based std calls
//! are not containment against hostile concurrent ancestor replacement. Native
//! directory-handle qualification remains necessary for that boundary. A scope
//! is a selection, not host permission or a durable project identity.

use std::fmt;
use std::fs::{self, File, Metadata};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

pub const MAX_TEXT_BYTES: usize = 1024 * 1024;
pub const MAX_BINARY_BYTES: usize = 20 * 1024 * 1024;

#[derive(Debug)]
pub enum ScopeError {
    RootMustBeAbsolute,
    RootMustBeDirectory,
    OutsideScope,
    Symlink,
    NotRegularFile,
    InvalidLimit,
    TooLarge {
        limit: usize,
    },
    InvalidUtf8,
    ChangedDuringRead,
    Io {
        operation: &'static str,
        source: io::Error,
    },
}

impl fmt::Display for ScopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootMustBeAbsolute => write!(f, "select an explicit absolute project root"),
            Self::RootMustBeDirectory => {
                write!(f, "selected project root must be an existing directory")
            }
            Self::OutsideScope => write!(
                f,
                "source path is outside the selected root or contains traversal"
            ),
            Self::Symlink => write!(
                f,
                "symlink source paths are not supported by the M0 prototype"
            ),
            Self::NotRegularFile => write!(f, "selected source must be a regular file"),
            Self::InvalidLimit => write!(
                f,
                "read limit is outside the selected operation's byte bound"
            ),
            Self::TooLarge { limit } => {
                write!(f, "selected source exceeds the {limit}-byte read limit")
            }
            Self::InvalidUtf8 => write!(f, "selected source is not valid UTF-8 text"),
            Self::ChangedDuringRead => write!(
                f,
                "selected source changed during the read; inspect before retrying"
            ),
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
        }
    }
}

impl std::error::Error for ScopeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn io_error(operation: &'static str, source: io::Error) -> ScopeError {
    ScopeError::Io { operation, source }
}

#[derive(Clone, Debug)]
pub struct ProjectScope {
    root: PathBuf,
    selected_root: PathBuf,
}

/// The returned path and byte length describe this bounded read, not a durable
/// revision token. Source content is intentionally omitted from Debug output.
pub struct ScopedBytes {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
}

pub struct ScopedText {
    pub path: PathBuf,
    pub byte_len: usize,
    pub text: String,
}

impl ProjectScope {
    pub fn open(root: &Path) -> Result<Self, ScopeError> {
        if !root.is_absolute() {
            return Err(ScopeError::RootMustBeAbsolute);
        }
        let metadata =
            fs::symlink_metadata(root).map_err(|e| io_error("inspect project root", e))?;
        if metadata.file_type().is_symlink() {
            return Err(ScopeError::Symlink);
        }
        if !metadata.is_dir() {
            return Err(ScopeError::RootMustBeDirectory);
        }
        let selected_root = root.to_owned();
        let root = fs::canonicalize(root).map_err(|e| io_error("resolve project root", e))?;
        if !fs::metadata(&root)
            .map_err(|e| io_error("inspect resolved project root", e))?
            .is_dir()
        {
            return Err(ScopeError::RootMustBeDirectory);
        }
        Ok(Self {
            root,
            selected_root,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn resolve_existing_file(&self, path: &Path) -> Result<PathBuf, ScopeError> {
        let path = if path.is_absolute() {
            // Translate only the root spelling explicitly selected by the
            // caller (for example macOS /var versus /private/var). Do not
            // canonicalize the file first: that would hide internal symlinks.
            if let Ok(relative) = path.strip_prefix(&self.root) {
                self.root.join(relative)
            } else {
                let relative = path
                    .strip_prefix(&self.selected_root)
                    .map_err(|_| ScopeError::OutsideScope)?;
                self.root.join(relative)
            }
        } else {
            self.root.join(path)
        };
        let relative = path
            .strip_prefix(&self.root)
            .map_err(|_| ScopeError::OutsideScope)?;
        if relative.as_os_str().is_empty()
            || relative
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err(ScopeError::OutsideScope);
        }
        let mut current = self.root.clone();
        let root_metadata =
            fs::symlink_metadata(&current).map_err(|e| io_error("recheck project root", e))?;
        if root_metadata.file_type().is_symlink() {
            return Err(ScopeError::Symlink);
        }
        if !root_metadata.is_dir() {
            return Err(ScopeError::RootMustBeDirectory);
        }
        for component in relative.components() {
            current.push(component);
            let metadata =
                fs::symlink_metadata(&current).map_err(|e| io_error("inspect source path", e))?;
            if metadata.file_type().is_symlink() {
                return Err(ScopeError::Symlink);
            }
        }
        let canonical = fs::canonicalize(&path).map_err(|e| io_error("resolve source path", e))?;
        if canonical != path || !canonical.starts_with(&self.root) {
            return Err(ScopeError::OutsideScope);
        }
        if !fs::symlink_metadata(&canonical)
            .map_err(|e| io_error("inspect source file", e))?
            .is_file()
        {
            return Err(ScopeError::NotRegularFile);
        }
        Ok(canonical)
    }

    pub fn read_text(&self, path: &Path, max_bytes: usize) -> Result<ScopedText, ScopeError> {
        if max_bytes == 0 || max_bytes > MAX_TEXT_BYTES {
            return Err(ScopeError::InvalidLimit);
        }
        let source = self.read_bytes(path, max_bytes)?;
        let byte_len = source.bytes.len();
        let text = String::from_utf8(source.bytes).map_err(|_| ScopeError::InvalidUtf8)?;
        Ok(ScopedText {
            path: source.path,
            byte_len,
            text,
        })
    }

    pub fn read_bytes(&self, path: &Path, max_bytes: usize) -> Result<ScopedBytes, ScopeError> {
        if max_bytes == 0 || max_bytes > MAX_BINARY_BYTES {
            return Err(ScopeError::InvalidLimit);
        }
        let path = self.resolve_existing_file(path)?;
        let before =
            fs::symlink_metadata(&path).map_err(|e| io_error("inspect source before read", e))?;
        if before.len() > max_bytes as u64 {
            return Err(ScopeError::TooLarge { limit: max_bytes });
        }
        let mut file = File::open(&path).map_err(|e| io_error("open source", e))?;
        let opened = file
            .metadata()
            .map_err(|e| io_error("inspect open source", e))?;
        if !opened.is_file() || !same_metadata(&before, &opened) {
            return Err(ScopeError::ChangedDuringRead);
        }
        let mut bytes = Vec::new();
        (&mut file)
            .take(max_bytes as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| io_error("read source", e))?;
        if bytes.len() > max_bytes {
            return Err(ScopeError::TooLarge { limit: max_bytes });
        }
        self.resolve_existing_file(&path)?;
        let after =
            fs::symlink_metadata(&path).map_err(|e| io_error("inspect source after read", e))?;
        let handle_after = file
            .metadata()
            .map_err(|e| io_error("recheck open source", e))?;
        if !same_metadata(&before, &after)
            || !same_metadata(&before, &handle_after)
            || bytes.len() as u64 != before.len()
        {
            return Err(ScopeError::ChangedDuringRead);
        }
        Ok(ScopedBytes { path, bytes })
    }
}

// A consistency check during one read only. No snapshot storage, hashing or
// persistent identity is introduced here. Metadata is weaker on non-Unix hosts.
fn same_metadata(a: &Metadata, b: &Metadata) -> bool {
    if a.len() != b.len() || a.modified().ok() != b.modified().ok() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if (a.dev(), a.ino(), a.ctime(), a.ctime_nsec())
            != (b.dev(), b.ino(), b.ctime(), b.ctime_nsec())
        {
            return false;
        }
    }
    true
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
                "mirket-scope-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            Self(fs::canonicalize(root).unwrap())
        }
        fn scope(&self) -> ProjectScope {
            ProjectScope::open(&self.0).unwrap()
        }
        fn path(&self, relative: &str) -> PathBuf {
            self.0.join(relative)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn roots_are_explicit_absolute_existing_directories() {
        assert!(matches!(
            ProjectScope::open(Path::new(".")),
            Err(ScopeError::RootMustBeAbsolute)
        ));
        let fixture = Fixture::new();
        fs::write(fixture.path("file"), "content").unwrap();
        assert!(matches!(
            ProjectScope::open(&fixture.path("file")),
            Err(ScopeError::RootMustBeDirectory)
        ));
        assert!(ProjectScope::open(&fixture.path("missing")).is_err());
        assert_eq!(fixture.scope().root(), fixture.0);
    }

    #[test]
    fn two_roots_do_not_share_selection_or_allow_absolute_cross_reads() {
        let a = Fixture::new();
        let b = Fixture::new();
        fs::write(a.path("same.txt"), "İstanbul").unwrap();
        fs::write(b.path("same.txt"), "Berlin").unwrap();
        let first = a.scope().read_text(Path::new("same.txt"), 100).unwrap();
        let second = b.scope().read_text(Path::new("same.txt"), 100).unwrap();
        assert_eq!(first.text, "İstanbul");
        assert_eq!(first.byte_len, "İstanbul".len());
        assert_eq!(first.path, a.path("same.txt"));
        assert_eq!(second.text, "Berlin");
        assert!(matches!(
            a.scope().read_text(&b.path("same.txt"), 100),
            Err(ScopeError::OutsideScope)
        ));
        assert_eq!(
            a.scope()
                .resolve_existing_file(&a.path("same.txt"))
                .unwrap(),
            a.path("same.txt")
        );
    }

    #[test]
    fn traversal_and_directories_are_refused() {
        let f = Fixture::new();
        fs::create_dir(f.path("nested")).unwrap();
        fs::write(f.path("text"), "content").unwrap();
        for path in ["../outside", "nested/../text", "", "."] {
            assert!(matches!(
                f.scope().resolve_existing_file(Path::new(path)),
                Err(ScopeError::OutsideScope)
            ));
        }
        assert!(matches!(
            f.scope().resolve_existing_file(Path::new("nested")),
            Err(ScopeError::NotRegularFile)
        ));
    }

    #[test]
    fn full_reads_enforce_cap_utf8_and_exact_byte_boundary() {
        let f = Fixture::new();
        fs::write(f.path("text"), "é").unwrap();
        assert_eq!(f.scope().read_text(Path::new("text"), 2).unwrap().text, "é");
        assert!(matches!(
            f.scope().read_text(Path::new("text"), 1),
            Err(ScopeError::TooLarge { limit: 1 })
        ));
        for limit in [0, MAX_TEXT_BYTES + 1, usize::MAX] {
            assert!(matches!(
                f.scope().read_text(Path::new("text"), limit),
                Err(ScopeError::InvalidLimit)
            ));
        }
        fs::write(f.path("binary"), [0xff]).unwrap();
        assert!(matches!(
            f.scope().read_text(Path::new("binary"), 100),
            Err(ScopeError::InvalidUtf8)
        ));
        fs::write(f.path("large"), vec![b'x'; MAX_TEXT_BYTES + 1]).unwrap();
        assert!(matches!(
            f.scope().read_text(Path::new("large"), MAX_TEXT_BYTES),
            Err(ScopeError::TooLarge { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_sources_parents_roots_and_escapes_are_refused() {
        use std::os::unix::fs::symlink;
        let f = Fixture::new();
        let other = Fixture::new();
        fs::write(f.path("real"), "inside").unwrap();
        fs::write(other.path("real"), "outside").unwrap();
        symlink(f.path("real"), f.path("alias")).unwrap();
        symlink(&other.0, f.path("escape")).unwrap();
        symlink(other.path("real"), f.path("external")).unwrap();
        for path in ["alias", "escape/real", "external"] {
            assert!(matches!(
                f.scope().read_text(Path::new(path), 100),
                Err(ScopeError::Symlink)
            ));
        }
        assert!(matches!(
            ProjectScope::open(&f.path("escape")),
            Err(ScopeError::Symlink)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn selected_root_ancestor_alias_is_normalized_without_hiding_source_symlinks() {
        use std::os::unix::fs::symlink;
        let f = Fixture::new();
        fs::create_dir_all(f.path("real/project")).unwrap();
        fs::write(f.path("real/project/source.txt"), "selected source").unwrap();
        symlink(f.path("real"), f.path("alias")).unwrap();
        symlink(
            f.path("real/project/source.txt"),
            f.path("real/project/link.txt"),
        )
        .unwrap();
        let scope = ProjectScope::open(&f.path("alias/project")).unwrap();
        let text = scope
            .read_text(&f.path("alias/project/source.txt"), 100)
            .unwrap();
        assert_eq!(text.text, "selected source");
        assert_eq!(text.path, f.path("real/project/source.txt"));
        assert!(
            scope
                .read_text(&f.path("real/project/source.txt"), 100)
                .is_ok()
        );
        assert!(matches!(
            scope.read_text(&f.path("alias/project/link.txt"), 100),
            Err(ScopeError::Symlink)
        ));
        assert!(matches!(
            scope.read_text(&f.path("alias/project/../project/source.txt"), 100),
            Err(ScopeError::OutsideScope)
        ));
    }
}
