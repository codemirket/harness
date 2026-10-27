//! Verify the admitted private SQLite input before the final Mirket link.
//! This is a drift guard, not proof against a writer controlling this checkout.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};

fn require(ok: bool, message: impl Into<String>) -> Result<(), String> {
    if ok { Ok(()) } else { Err(message.into()) }
}

fn regular(path: &Path) -> Result<(), String> {
    let m = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    require(
        m.file_type().is_file(),
        format!("Expected regular native file: {}", path.display()),
    )
}

fn hash(path: &Path) -> Result<String, String> {
    regular(path)?;
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn no_links(path: &Path) -> Result<(), String> {
    for p in path.ancestors() {
        let m = fs::symlink_metadata(p).map_err(|e| format!("{}: {e}", p.display()))?;
        require(
            !m.file_type().is_symlink(),
            format!("Symlinked native build path: {}", p.display()),
        )?;
    }
    Ok(())
}

fn entries(path: &Path, expected: &[&str]) -> Result<(), String> {
    no_links(path)?;
    let names = fs::read_dir(path)
        .map_err(|e| e.to_string())?
        .map(|entry| entry.map(|e| e.file_name().to_string_lossy().into_owned()))
        .collect::<Result<BTreeSet<_>, _>>()
        .map_err(|e| e.to_string())?;
    let expected = expected
        .iter()
        .map(|s| (*s).to_owned())
        .collect::<BTreeSet<_>>();
    require(
        names == expected,
        format!("Unexpected or competing native files in {}", path.display()),
    )
}

fn field_file(receipt: &Value, key: &str, path: &Path, relative: &str) -> Result<(), String> {
    regular(path)?;
    let bytes = fs::metadata(path).map_err(|e| e.to_string())?.len();
    require(
        receipt[key] == json!({"path": relative, "bytes": bytes, "sha256": hash(path)?}),
        format!("Native {key} hash/size/path differs from its receipt"),
    )
}

fn verify() -> Result<(), String> {
    let root =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").ok_or("Missing manifest directory")?);
    let target = env::var("TARGET").map_err(|e| e.to_string())?;
    for (key, expected) in [
        ("RUSTONIG_SYSTEM_LIBONIG", "0"),
        ("RUSTONIG_DYNAMIC_LIBONIG", "0"),
        ("RUSTONIG_STATIC_LIBONIG", "1"),
    ] {
        println!("cargo:rerun-if-env-changed={key}");
        require(
            env::var(key).as_deref() == Ok(expected),
            format!("Missing forced vendored Oniguruma setting: {key}"),
        )?;
    }
    let (library_name, machine) = match target.as_str() {
        "aarch64-apple-darwin" => ("libsqlite3.a", "arm64"),
        "aarch64-pc-windows-msvc" => ("sqlite3.lib", "arm64"),
        "x86_64-pc-windows-msvc" => ("sqlite3.lib", "x64"),
        _ => return Err(format!("Unsupported native SQLite target: {target}")),
    };
    if target.ends_with("windows-msvc") {
        require(
            env::var("CARGO_CFG_TARGET_FEATURE")
                .unwrap_or_default()
                .split(',')
                .any(|f| f == "crt-static"),
            "Windows SQLite /MT requires the admitted Rust crt-static target feature",
        )?;
    }
    let native = root.join("target/native/sqlite");
    let manifest_path = root.join("packaging/admission/m2-sqlite.native-build.json");
    let receipt_path = native.join("build-receipt.json");
    for path in [
        &native,
        &native.join("lib"),
        &native.join("include"),
        &manifest_path,
        &receipt_path,
    ] {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    entries(&native, &["lib", "include", "build-receipt.json"])?;
    entries(&native.join("lib"), &[library_name])?;
    entries(&native.join("include"), &["sqlite3.h"])?;
    for (key, value) in [
        ("SQLITE3_STATIC", "1"),
        ("SQLITE3_NO_PKG_CONFIG", "1"),
        ("LIBSQLITE3_SYS_USE_PKG_CONFIG", "0"),
    ] {
        println!("cargo:rerun-if-env-changed={key}");
        require(
            env::var(key).as_deref() == Ok(value),
            format!("Missing forced native build setting: {key}"),
        )?;
    }
    for (key, expected) in [
        ("SQLITE3_LIB_DIR", native.join("lib")),
        ("SQLITE3_INCLUDE_DIR", native.join("include")),
    ] {
        println!("cargo:rerun-if-env-changed={key}");
        let actual = PathBuf::from(
            env::var_os(key)
                .ok_or_else(|| format!("Missing {key}; invoke Cargo from the repository"))?,
        );
        require(actual.is_absolute(), format!("{key} must be absolute"))?;
        require(
            actual.canonicalize().map_err(|e| e.to_string())?
                == expected.canonicalize().map_err(|e| e.to_string())?,
            format!("{key} does not select this repository's native payload"),
        )?;
    }
    regular(&manifest_path)?;
    regular(&receipt_path)?;
    require(
        fs::metadata(&receipt_path)
            .map_err(|e| e.to_string())?
            .len()
            <= 128 * 1024,
        "Oversized native receipt",
    )?;
    let manifest: Value =
        serde_json::from_slice(&fs::read(&manifest_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let receipt: Value =
        serde_json::from_slice(&fs::read(&receipt_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    require(
        manifest["sqlite_version"] == "3.54.1" && manifest["source_modifications"] == "none",
        "Unexpected native source admission",
    )?;
    for (key, expected) in [
        ("schema", json!(1)),
        ("target", json!(target)),
        ("object_machine", json!(machine)),
        ("sqlite_version", manifest["sqlite_version"].clone()),
        ("sqlite_source_id", manifest["sqlite_source_id"].clone()),
        ("manifest_sha256", json!(hash(&manifest_path)?)),
        ("source_archive_sha256", manifest["sha256"].clone()),
        ("defines", manifest["required_defines"].clone()),
    ] {
        require(
            receipt[key] == expected,
            format!("Native receipt mismatch: {key}"),
        )?;
    }
    let selected = manifest["source_selection"]
        .as_array()
        .ok_or("Missing source selection")?;
    let files = manifest["files"].as_array().ok_or("Missing source files")?;
    let mut sources = serde_json::Map::new();
    for file in files {
        if selected.contains(&file["path"]) {
            let name = file["path"]
                .as_str()
                .ok_or("Invalid source path")?
                .rsplit('/')
                .next()
                .ok_or("Invalid source basename")?;
            require(
                matches!(name, "sqlite3.c" | "sqlite3.h"),
                "Unexpected native source member",
            )?;
            require(
                sources
                    .insert(name.to_owned(), file["sha256"].clone())
                    .is_none(),
                "Duplicate native source member",
            )?;
        }
    }
    require(
        sources.len() == 2 && selected.len() == 2,
        "Expected two selected native sources",
    )?;
    require(
        receipt["source_sha256"] == Value::Object(sources.clone()),
        "Native source hashes differ",
    )?;
    let library = native.join("lib").join(library_name);
    let header = native.join("include/sqlite3.h");
    for path in [&library, &header] {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    field_file(
        &receipt,
        "library",
        &library,
        &format!("lib/{library_name}"),
    )?;
    field_file(&receipt, "header", &header, "include/sqlite3.h")?;
    require(
        json!(hash(&header)?) == sources["sqlite3.h"],
        "Header differs from admitted source",
    )?;
    Ok(())
}

fn main() {
    if let Err(error) = verify() {
        panic!(
            "Private native build input refused: {error}. Prepare scripts/build-sqlite.py with the admitted archive and explicit target tools."
        );
    }
}
