use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Paths {
    pub home: PathBuf,
    pub root: PathBuf,
}

impl Paths {
    pub fn new(home: PathBuf) -> Result<Self> {
        if !home.is_absolute() {
            bail!("home must be an absolute path");
        }
        let home = home.canonicalize().context("home must exist")?;
        Ok(Self {
            root: home.join(".mirket"),
            home,
        })
    }
    pub fn discover(home: Option<PathBuf>) -> Result<Self> {
        Self::new(
            home.or_else(|| std::env::var_os("HOME").map(PathBuf::from))
                .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
                .context("home is unavailable; supply --home")?,
        )
    }
    pub fn binary(&self) -> PathBuf {
        self.root.join("bin").join(if cfg!(windows) {
            "mirket.exe"
        } else {
            "mirket"
        })
    }
    pub fn database(&self) -> PathBuf {
        self.root.join("state.db")
    }
    pub fn settings(&self) -> PathBuf {
        self.root.join("setup.json")
    }
    pub fn cache(&self) -> PathBuf {
        self.root.join("cache")
    }
    pub fn ensure(&self) -> Result<()> {
        crate::util::ensure_directory(&self.root)?;
        Ok(())
    }
}

pub fn canonical_project(path: &Path) -> Result<PathBuf> {
    let path = path
        .canonicalize()
        .context("project directory must exist")?;
    if !path.is_dir() {
        bail!("project must be a directory");
    }
    Ok(path)
}
