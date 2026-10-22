//! Shared build/runtime fingerprint contract. No Git state or user files.
use sha2::{Digest, Sha256};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
pub const INPUTS: &[&str] = &[
    "src",
    "setup",
    "skills",
    "registry",
    "instructions",
    "docs/third-party-notices.md",
    "docs/licenses",
    "runtime/design",
    "runtime/agent",
    "Cargo.toml",
    "Cargo.lock",
    "build.rs",
    "rust-toolchain.toml",
    "tests",
    "evals",
];
fn files(root: &Path, path: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    if path.file_name().is_some_and(|n| n == ".DS_Store") {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(io::Error::other(
            "source identity does not follow symbolic links",
        ));
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            files(root, &entry?.path(), out)?;
        }
    } else {
        out.push(
            path.strip_prefix(root)
                .map_err(io::Error::other)?
                .to_owned(),
        );
    }
    Ok(())
}
// Execution compatibility excludes catalog/guidance and standalone tests. Selected
// workflows, guidance and tool schemas are bound separately by the run envelope.
const EXECUTION_INPUTS: &[&str] = &[
    "src",
    "runtime/design",
    "runtime/agent",
    "Cargo.toml",
    "Cargo.lock",
    "build.rs",
    "rust-toolchain.toml",
];
pub fn execution_fingerprint(root: &Path) -> io::Result<String> {
    fingerprint_inputs(root, EXECUTION_INPUTS, true)
}
pub fn fingerprint(root: &Path) -> io::Result<String> {
    fingerprint_inputs(root, INPUTS, false)
}
fn fingerprint_inputs(root: &Path, names: &[&str], execution: bool) -> io::Result<String> {
    let mut inputs = Vec::new();
    for name in names {
        files(root, &root.join(name), &mut inputs)?;
    }
    inputs.sort();
    let mut hash = Sha256::new();
    for path in inputs {
        if execution
            && path.components().any(|part| {
                matches!(
                    part.as_os_str().to_str(),
                    Some("test" | "tests" | "__pycache__")
                )
            })
        {
            continue;
        }
        if execution {
            let filename = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if filename.starts_with("test_")
                || filename.ends_with("_tests.rs")
                || [
                    "src/cli.rs",
                    "src/update.rs",
                    "src/role_eval.rs",
                    "src/dev.rs",
                ]
                .iter()
                .any(|name| path == Path::new(name))
                || path.starts_with("src/update")
                || path.starts_with("src/dev")
            {
                continue;
            }
        }
        let name = path.to_string_lossy().replace('\\', "/");
        let bytes = fs::read(root.join(path))?;
        hash.update((name.len() as u64).to_le_bytes());
        hash.update(name.as_bytes());
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    Ok(format!("{:x}", hash.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn audit_and_execution_identities_have_distinct_inputs() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        for name in INPUTS {
            let path = root.join(name);
            if path.extension().is_some() {
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(path, "fixture").unwrap();
            } else {
                fs::create_dir_all(path).unwrap();
            }
        }
        fs::create_dir_all(root.join("runtime/design/tools/test")).unwrap();
        fs::write(root.join("src/worker.rs"), "runtime v1").unwrap();
        let audit = fingerprint(root).unwrap();
        let execution = execution_fingerprint(root).unwrap();
        for name in [
            "src/cli.rs",
            "src/codex_tests.rs",
            "runtime/agent/test_runtime.py",
            "skills/unrelated.md",
            "registry/profiles.json",
            "tests/probe.rs",
            "runtime/design/tools/test/probe.mjs",
            "docs/third-party-notices.md",
        ] {
            fs::write(root.join(name), "unrelated edit").unwrap();
            assert_ne!(fingerprint(root).unwrap(), audit);
            assert_eq!(execution_fingerprint(root).unwrap(), execution, "{name}");
        }
        fs::write(root.join("src/worker.rs"), "runtime v2").unwrap();
        assert_ne!(execution_fingerprint(root).unwrap(), execution);
        fs::write(root.join("src/worker.rs"), "runtime v1").unwrap();
        fs::write(root.join("Cargo.lock"), "dependency change").unwrap();
        assert_ne!(execution_fingerprint(root).unwrap(), execution);
    }
}
