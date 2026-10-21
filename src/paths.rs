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
        let discovered = preferred_home(
            std::env::consts::OS,
            std::env::var_os("HOME").map(PathBuf::from),
            std::env::var_os("USERPROFILE").map(PathBuf::from),
        );
        Self::new(
            home.or(discovered)
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
    /// Private, disposable Mirket work. Callers own cleanup of each child.
    pub fn scratch(&self) -> Result<PathBuf> {
        self.ensure()?;
        let path = self.root.join("tmp");
        crate::util::ensure_directory(&path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))?;
        }
        Ok(path)
    }
    pub fn ensure(&self) -> Result<()> {
        crate::util::ensure_directory(&self.root)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.root, std::fs::Permissions::from_mode(0o700))?;
        }
        Ok(())
    }
}

fn preferred_home(os: &str, home: Option<PathBuf>, profile: Option<PathBuf>) -> Option<PathBuf> {
    if os == "windows" {
        profile.or(home)
    } else {
        home
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_prefers_the_user_profile_even_with_a_shell_home() {
        let shell = PathBuf::from("C:/shell/home");
        let profile = PathBuf::from("C:/Users/Ada");
        assert_eq!(
            preferred_home("windows", Some(shell.clone()), Some(profile.clone())),
            Some(profile.clone())
        );
        assert_eq!(
            preferred_home("linux", Some(shell.clone()), None),
            Some(shell)
        );
        assert_eq!(preferred_home("linux", None, Some(profile)), None);
    }

    #[test]
    fn disposable_work_stays_with_the_selected_home() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().to_owned()).unwrap();
        assert_eq!(paths.scratch().unwrap(), paths.root.join("tmp"));
        assert!(!home.path().join("tmp").exists());
    }
}
