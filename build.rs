use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn files(path: &Path, result: &mut Vec<PathBuf>) {
    // Finder metadata is neither compiled source nor an embedded resource.
    if path.file_name().is_some_and(|name| name == ".DS_Store") {
        return;
    }
    if path.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            files(&entry.unwrap().path(), result);
        }
    } else {
        result.push(path.to_path_buf());
    }
}
fn main() {
    let target = std::env::var("TARGET").unwrap();
    println!("cargo:rustc-env=MIRKET_TARGET={target}");
    let mut inputs = Vec::new();
    for path in [
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
    ] {
        println!("cargo:rerun-if-changed={path}");
        files(Path::new(path), &mut inputs);
    }
    inputs.sort();
    let mut hash = Sha256::new();
    for path in inputs {
        let name = path.to_string_lossy().replace('\\', "/");
        let bytes = fs::read(&path).unwrap();
        hash.update((name.len() as u64).to_le_bytes());
        hash.update(name.as_bytes());
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    println!("cargo:rustc-env=MIRKET_SOURCE_SHA256={:x}", hash.finalize());
}
