//! Release-only collection of the locked dependency sources' actual notices.
use crate::util;
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};

const MAX_PACKAGES: usize = 2048;
const MAX_ENTRIES: usize = 250_000;
const MAX_NOTICE: u64 = 4 * 1024 * 1024;
const MAX_OUTPUT: usize = 32 * 1024 * 1024;
const SUPPLEMENTS: &str = "docs/licenses/rust-runtime";

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<Package>,
    resolve: Resolve,
}
#[derive(Deserialize)]
struct Package {
    id: String,
    name: String,
    version: String,
    source: Option<String>,
    manifest_path: PathBuf,
    license: Option<String>,
    license_file: Option<PathBuf>,
}
#[derive(Deserialize)]
struct Resolve {
    root: Option<String>,
    nodes: Vec<Node>,
}
#[derive(Deserialize)]
struct Node {
    id: String,
    dependencies: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Supplement {
    name: String,
    version: String,
    source: String,
    checksum: String,
    vcs_commit: String,
    files: Vec<SupplementFile>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SupplementFile {
    path: PathBuf,
    url: String,
    sha256: String,
}

fn text(path: &Path, limit: u64) -> Result<String> {
    let value = String::from_utf8(util::read_bounded(path, limit)?)?;
    ensure!(!value.trim().is_empty(), "empty notice: {}", path.display());
    Ok(value)
}

fn label(value: &str) -> Result<&str> {
    ensure!(
        !value.is_empty() && !value.chars().any(char::is_control),
        "invalid notice identity"
    );
    Ok(value)
}

fn relative(path: &Path) -> Result<String> {
    util::safe_relative(path)?;
    label(path.to_str().context("non-UTF-8 notice path")?)?;
    Ok(path.to_string_lossy().replace('\\', "/"))
}

fn notice_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    [
        "license",
        "licence",
        "copying",
        "copyright",
        "notice",
        "notices",
        "unlicense",
        "authors",
    ]
    .iter()
    .any(|prefix| {
        name.strip_prefix(prefix)
            .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with(['-', '_', '.', ' ']))
    })
}

fn license_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    ["license", "licence", "copying", "unlicense"]
        .iter()
        .any(|prefix| {
            name.strip_prefix(prefix)
                .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with(['-', '_', '.', ' ']))
        })
}

fn retain(files: &mut BTreeMap<String, String>, path: String, body: String) -> Result<()> {
    let retained: usize = files.values().map(String::len).sum();
    ensure!(
        retained + body.len() <= MAX_OUTPUT,
        "retained Rust notices exceed output limit"
    );
    files.insert(path, body);
    Ok(())
}

fn collect(
    root: &Path,
    directory: &Path,
    depth: usize,
    entries: &mut usize,
    files: &mut BTreeMap<String, String>,
    ring_headers: bool,
) -> Result<()> {
    ensure!(depth <= 32, "notice source exceeds directory depth limit");
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        *entries += 1;
        ensure!(*entries <= MAX_ENTRIES, "notice source exceeds entry limit");
        let path = entry.path();
        let kind = entry.file_type()?;
        ensure!(
            !kind.is_symlink(),
            "linked notice source: {}",
            path.display()
        );
        if kind.is_dir() {
            collect(root, &path, depth + 1, entries, files, ring_headers)?;
        } else {
            ensure!(
                kind.is_file(),
                "non-regular notice source: {}",
                path.display()
            );
            let rel = path.strip_prefix(root)?;
            if notice_name(&entry.file_name().to_string_lossy())
                || rel.parent().is_some_and(|parent| {
                    parent.components().any(|part| {
                        matches!(part.as_os_str().to_str(), Some("licenses" | "LICENSES"))
                    })
                })
            {
                retain(files, relative(rel)?, text(&path, MAX_NOTICE)?)?;
            } else if ring_headers
                && matches!(
                    path.extension().and_then(|ext| ext.to_str()),
                    Some("rs" | "c" | "h" | "inl" | "pl" | "S" | "s" | "asm")
                )
                && let Some(body) = ring_source_notice(&path)?
            {
                retain(files, format!("{} (source header)", relative(rel)?), body)?;
            }
        }
    }
    Ok(())
}

// ring's LICENSE explicitly delegates per-file copyrights to these headers.
// Retain only the leading comments, plus generated assembly preprocessor
// prologues; never scan implementation code for license-like substrings.
fn ring_source_notice(path: &Path) -> Result<Option<String>> {
    const LIMIT: usize = 64 * 1024;
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take((LIMIT + 1) as u64)
        .read_to_end(&mut bytes)?;
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let assembly = matches!(extension, "S" | "s" | "asm");
    let mut end = 0;
    loop {
        let start = end
            + bytes[end..]
                .iter()
                .take_while(|b| b.is_ascii_whitespace())
                .count();
        let rest = &bytes[start..];
        if rest.starts_with(b"//!") || rest.starts_with(b"///") {
            break;
        }
        if rest.starts_with(b"//")
            || (extension == "pl" && rest.starts_with(b"#"))
            || (assembly && matches!(rest.first(), Some(b'@' | b';' | b'#')))
        {
            end = start
                + rest
                    .iter()
                    .position(|b| *b == b'\n')
                    .map_or(rest.len(), |i| i + 1);
        } else if rest.starts_with(b"/*") {
            let close = rest
                .windows(2)
                .position(|pair| pair == b"*/")
                .context("ring source notice has an unterminated or oversized comment")?;
            end = start + close + 2;
        } else {
            break;
        }
        ensure!(end <= LIMIT, "ring source notice exceeds prefix limit");
    }
    let body = std::str::from_utf8(&bytes[..end])?;
    let lower = body.to_ascii_lowercase();
    Ok((lower.contains("copyright") || lower.contains("permission")).then(|| body.to_string()))
}

fn append(output: &mut String, value: &str) -> Result<()> {
    ensure!(
        output.len() + value.len() <= MAX_OUTPUT,
        "Rust notices exceed output limit"
    );
    output.push_str(value);
    Ok(())
}

fn append_files(output: &mut String, files: &BTreeMap<String, String>) -> Result<()> {
    for (path, body) in files {
        append(
            output,
            &format!(
                "\n--- {path} (SHA-256 {}) ---\n",
                util::sha256(body.as_bytes())
            ),
        )?;
        append(output, body)?;
        append(output, "\n")?;
    }
    Ok(())
}

fn sqlite_notice(root: &Path) -> Result<String> {
    let path = root.join("sqlite3/sqlite3.c");
    util::reject_symlink_components(&path)?;
    ensure!(
        fs::symlink_metadata(&path)?.is_file(),
        "SQLite source must be regular"
    );
    let mut prefix = String::new();
    fs::File::open(path)?
        .take(16 * 1024)
        .read_to_string(&mut prefix)?;
    let end = prefix
        .find("#ifndef SQLITEINT_H")
        .context("SQLite notice header changed; review its source")?;
    let header = &prefix[..end];
    ensure!(
        header.contains("The author disclaims copyright to this source code.")
            && header.contains("May you share freely, never taking more than you give."),
        "SQLite public-domain notice is missing"
    );
    Ok(header.to_string())
}

fn package_files(
    project: &Path,
    package: &Package,
    checksum: &str,
    supplements: &[Supplement],
    entries: &mut usize,
) -> Result<BTreeMap<String, String>> {
    util::reject_symlink_components(&package.manifest_path)?;
    let root = package
        .manifest_path
        .parent()
        .context("crate manifest has no parent")?
        .canonicalize()?;
    let mut files = BTreeMap::new();
    collect(&root, &root, 0, entries, &mut files, package.name == "ring")?;
    let mut license_evidence = files.keys().any(|path| {
        Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(license_name)
    });
    // This pinned crate places its complete MIT grant in AUTHORS. A list of
    // contributors alone must not satisfy the missing-license check.
    if package.name == "r-efi" && package.version == "6.0.0" {
        license_evidence |= files.get("AUTHORS").is_some_and(|body| {
            body.contains("Permission is hereby granted, free of charge")
                && body.contains("THE SOFTWARE IS PROVIDED \"AS IS\"")
        });
    }
    if let Some(path) = &package.license_file {
        let rel = if path.is_absolute() {
            path.strip_prefix(&root)?
        } else {
            path
        };
        retain(
            &mut files,
            relative(rel)?,
            text(&root.join(rel), MAX_NOTICE)?,
        )?;
        license_evidence = true;
    }
    if package.name == "ring" {
        for required in [
            "LICENSE-other-bits",
            "LICENSE-BoringSSL",
            "third_party/fiat/LICENSE",
            "src/polyfill/once_cell/LICENSE-APACHE",
            "src/polyfill/once_cell/LICENSE-MIT",
            "src/limb.rs (source header)",
            "src/debug.rs (source header)",
            "src/digest/sha1.rs (source header)",
            "crypto/poly1305/poly1305.c (source header)",
            "crypto/fipsmodule/ec/ecp_nistz.c (source header)",
        ] {
            ensure!(
                files.contains_key(required),
                "ring native notice missing: {required}"
            );
        }
    }
    if package.name == "libsqlite3-sys" {
        retain(
            &mut files,
            "sqlite3/sqlite3.c (notice header)".into(),
            sqlite_notice(&root)?,
        )?;
    }
    for supplement in supplements
        .iter()
        .filter(|s| s.name == package.name && s.version == package.version)
    {
        ensure!(
            package.source.as_deref() == Some(&supplement.source)
                && checksum == supplement.checksum,
            "supplement source/checksum changed for {} {}; re-review its primary notice",
            package.name,
            package.version
        );
        let vcs: serde_json::Value =
            serde_json::from_str(&text(&root.join(".cargo_vcs_info.json"), 64 * 1024)?)?;
        ensure!(
            vcs["git"]["sha1"].as_str() == Some(&supplement.vcs_commit),
            "supplement revision changed for {}",
            package.name
        );
        for file in &supplement.files {
            let path = relative(&file.path)?;
            let body = text(&project.join(SUPPLEMENTS).join(&file.path), MAX_NOTICE)?;
            ensure!(
                util::sha256(body.as_bytes()) == file.sha256,
                "supplement notice hash changed: {path}"
            );
            ensure!(
                file.url.starts_with("https://raw.githubusercontent.com/")
                    && file.url.contains(&format!("/{}/", supplement.vcs_commit)),
                "supplement must name an immutable primary source"
            );
            label(&file.url)?;
            retain(
                &mut files,
                format!("supplement {path}; source {}", file.url),
                body,
            )?;
            license_evidence = true;
        }
    }
    ensure!(
        license_evidence,
        "no license/notice evidence for {} {}; retain a pinned primary-source supplement",
        package.name,
        package.version
    );
    Ok(files)
}

/// The full resolved closure is deliberately retained, including build and
/// target-specific dependencies. It is not an assertion that every crate links.
pub(crate) fn generate(
    project: &Path,
    root_id: &str,
    metadata: &[u8],
    sysroot: &Path,
    compiler: &str,
) -> Result<Vec<u8>> {
    let metadata: Metadata =
        serde_json::from_slice(metadata).context("invalid notice dependency metadata")?;
    ensure!(
        metadata.packages.len() <= MAX_PACKAGES && metadata.resolve.nodes.len() <= MAX_PACKAGES,
        "too many notice dependencies"
    );
    ensure!(
        metadata.resolve.root.as_deref() == Some(root_id),
        "notice dependency root changed"
    );
    let packages: BTreeMap<_, _> = metadata
        .packages
        .iter()
        .map(|p| (p.id.as_str(), p))
        .collect();
    let nodes: BTreeMap<_, _> = metadata
        .resolve
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), n))
        .collect();
    ensure!(
        packages.len() == metadata.packages.len() && nodes.len() == metadata.resolve.nodes.len(),
        "duplicate notice metadata identity"
    );
    let mut pending = vec![root_id];
    let mut reached = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if reached.insert(id) {
            let node = nodes
                .get(id)
                .context("dependency missing from resolved notice graph")?;
            pending.extend(node.dependencies.iter().map(String::as_str));
        }
    }
    let lock =
        text(&project.join("Cargo.lock"), 4 * 1024 * 1024)?.parse::<toml_edit::DocumentMut>()?;
    let locked = lock
        .get("package")
        .and_then(toml_edit::Item::as_array_of_tables)
        .context("Cargo.lock packages missing")?;
    let supplement_path = project.join(SUPPLEMENTS).join("provenance.json");
    let supplements: Vec<Supplement> = if supplement_path.exists() {
        serde_json::from_str(&text(&supplement_path, 256 * 1024)?)?
    } else {
        Vec::new()
    };
    let mut ordered = BTreeMap::new();
    for id in reached.into_iter().filter(|id| *id != root_id) {
        let package = packages
            .get(id)
            .context("resolved package missing from notice metadata")?;
        let source = package
            .source
            .as_deref()
            .context("unlocked local dependency needs explicit notice review")?;
        let (_, url) = source
            .split_once('+')
            .context("unrecognized Cargo source")?;
        let url = reqwest::Url::parse(url)?;
        ensure!(
            url.scheme() == "https" && url.username().is_empty() && url.password().is_none(),
            "dependency source is not a public HTTPS identity"
        );
        let matches = locked
            .iter()
            .filter(|row| {
                row["name"].as_str() == Some(&package.name)
                    && row["version"].as_str() == Some(&package.version)
                    && row.get("source").and_then(toml_edit::Item::as_str) == Some(source)
            })
            .collect::<Vec<_>>();
        ensure!(
            matches.len() == 1,
            "dependency is not uniquely pinned in Cargo.lock: {} {}",
            package.name,
            package.version
        );
        let checksum = matches[0]
            .get("checksum")
            .and_then(toml_edit::Item::as_str)
            .context("dependency lacks a locked archive checksum")?;
        ensure!(
            checksum.len() == 64 && checksum.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid locked crate checksum"
        );
        let key = (
            label(&package.name)?,
            label(&package.version)?,
            label(source)?,
        );
        ensure!(
            ordered.insert(key, (*package, checksum)).is_none(),
            "duplicate dependency notice identity"
        );
    }
    let mut output = String::from(
        "Mirket Rust dependency notices\n\nScope: the complete Cargo --locked resolved source closure, including build,\ndevelopment and target-specific entries; not every listed crate is statically\nlinked into each executable. License declarations are package metadata; the\noriginal notice texts below retain their specific terms and attributions.\n\n",
    );
    append(
        &mut output,
        &format!(
            "Cargo.lock SHA-256: {}\nDependency packages: {}\n",
            util::sha256(&util::read_bounded(
                &project.join("Cargo.lock"),
                4 * 1024 * 1024
            )?),
            ordered.len()
        ),
    )?;
    let mut entries = 0;
    for ((name, version, source), (package, checksum)) in ordered {
        let files = package_files(project, package, checksum, &supplements, &mut entries)
            .with_context(|| format!("collect notices for {name} {version}"))?;
        let license = package
            .license
            .as_deref()
            .unwrap_or("custom license-file; see source notice");
        append(
            &mut output,
            &format!(
                "\n======== {name} {version} ========\nSource: {source}\nLocked archive SHA-256: {checksum}\nDeclared license: {}\n",
                label(license)?
            ),
        )?;
        append_files(&mut output, &files)?;
    }
    let toolchain =
        text(&project.join("rust-toolchain.toml"), 64 * 1024)?.parse::<toml_edit::DocumentMut>()?;
    let channel = toolchain["toolchain"]["channel"]
        .as_str()
        .context("pinned Rust toolchain missing")?;
    ensure!(
        compiler.starts_with(&format!("rustc {channel} ")),
        "notice compiler differs from pinned Rust toolchain {channel}"
    );
    let docs = sysroot.join("share/doc/rust");
    let mut files = BTreeMap::new();
    files.insert(
        "share/doc/rust/COPYRIGHT-library.html".into(),
        text(&docs.join("COPYRIGHT-library.html"), MAX_NOTICE)
            .context("installed toolchain lacks standard-library notices")?,
    );
    collect(
        sysroot,
        &docs.join("licenses"),
        0,
        &mut entries,
        &mut files,
        false,
    )?;
    for required in [
        "share/doc/rust/licenses/MIT.txt",
        "share/doc/rust/licenses/Apache-2.0.txt",
    ] {
        ensure!(
            files.contains_key(required),
            "installed Rust license missing: {required}"
        );
    }
    append(
        &mut output,
        &format!(
            "\n======== Rust standard library ========\nCompiler: {}\nSource: https://github.com/rust-lang/rust/tree/{channel}\nThe library-specific copyright inventory is retained verbatim in HTML source\nform, followed by the installed toolchain license texts.\n",
            label(compiler)?
        ),
    )?;
    append_files(&mut output, &files)?;
    Ok(output.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> (tempfile::TempDir, PathBuf, serde_json::Value) {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        fs::create_dir_all(root.join("crate/native/fiat")).unwrap();
        fs::write(root.join("crate/Cargo.toml"), "[package]\nname='example'\n").unwrap();
        fs::write(root.join("Cargo.lock"), format!("[[package]]\nname='example'\nversion='1.2.3'\nsource='registry+https://example.test/index'\nchecksum='{}'\n", "a".repeat(64))).unwrap();
        fs::write(
            root.join("rust-toolchain.toml"),
            "[toolchain]\nchannel='1.99.0'\n",
        )
        .unwrap();
        let docs = root.join("sysroot/share/doc/rust");
        fs::create_dir_all(docs.join("licenses")).unwrap();
        for (path, body) in [
            (
                "COPYRIGHT-library.html",
                "<p>Standard library attribution</p>",
            ),
            ("licenses/MIT.txt", "Rust MIT terms"),
            ("licenses/Apache-2.0.txt", "Rust Apache terms"),
        ] {
            fs::write(docs.join(path), body).unwrap();
        }
        let metadata = json!({"packages":[{"id":"dep", "name":"example", "version":"1.2.3", "source":"registry+https://example.test/index", "manifest_path":root.join("crate/Cargo.toml"), "license":"MIT"}], "resolve":{"root":"root", "nodes":[{"id":"root", "dependencies":["dep"]},{"id":"dep", "dependencies":[]}]}});
        (temp, root, metadata)
    }

    fn render(root: &Path, metadata: &serde_json::Value) -> Result<String> {
        Ok(String::from_utf8(generate(
            root,
            "root",
            &serde_json::to_vec(metadata)?,
            &root.join("sysroot"),
            "rustc 1.99.0 (test 2026-10-01)",
        )?)?)
    }

    #[test]
    fn rust_notices_missing_body_and_standard_library_evidence_fail() {
        let (_temp, root, metadata) = fixture();
        assert!(
            format!("{:#}", render(&root, &metadata).unwrap_err())
                .contains("no license/notice evidence")
        );
        fs::write(root.join("crate/AUTHORS"), "A list of contributor names").unwrap();
        assert!(
            format!("{:#}", render(&root, &metadata).unwrap_err())
                .contains("no license/notice evidence")
        );
        fs::write(root.join("crate/LICENSE"), "").unwrap();
        assert!(render(&root, &metadata).is_err());
        fs::write(
            root.join("crate/LICENSE"),
            "Actual package permission and copyright",
        )
        .unwrap();
        fs::remove_file(root.join("sysroot/share/doc/rust/COPYRIGHT-library.html")).unwrap();
        assert!(
            format!("{:#}", render(&root, &metadata).unwrap_err())
                .contains("standard-library notices")
        );
    }

    #[test]
    fn rust_notices_retain_nested_native_notices_deterministically_without_local_paths() {
        let (_temp, root, mut metadata) = fixture();
        fs::write(root.join("crate/LICENSE"), "Top-level terms").unwrap();
        fs::write(
            root.join("crate/native/fiat/LICENSE"),
            "Native copyright and grant",
        )
        .unwrap();
        let first = render(&root, &metadata).unwrap();
        metadata["resolve"]["nodes"]
            .as_array_mut()
            .unwrap()
            .reverse();
        assert_eq!(first, render(&root, &metadata).unwrap());
        assert!(first.contains("native/fiat/LICENSE"));
        assert!(first.contains("Native copyright and grant"));
        assert!(first.contains("Standard library attribution"));
        assert!(!first.contains(root.to_str().unwrap()));
        fs::write(
            root.join("crate/native/fiat/LICENSE"),
            "Changed native terms",
        )
        .unwrap();
        assert_ne!(first, render(&root, &metadata).unwrap());
    }

    #[test]
    fn rust_notices_declared_file_cannot_escape_and_lock_must_match() {
        let (_temp, root, mut metadata) = fixture();
        fs::write(root.join("crate/LICENSE"), "Actual terms").unwrap();
        metadata["packages"][0]["license_file"] = json!("../outside");
        assert!(render(&root, &metadata).is_err());
        metadata["packages"][0]["license_file"] = serde_json::Value::Null;
        metadata["packages"][0]["version"] = json!("9.9.9");
        assert!(render(&root, &metadata).is_err());
    }

    #[test]
    fn rust_notices_sqlite_header_is_required_separately_from_wrapper_license() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        fs::create_dir(root.join("sqlite3")).unwrap();
        fs::write(
            root.join("sqlite3/sqlite3.c"),
            "/* MIT wrapper only */\n#ifndef SQLITEINT_H",
        )
        .unwrap();
        assert!(sqlite_notice(&root).is_err());
        let header = "/* SQLite version 3.50.2\nThe author disclaims copyright to this source code.\nMay you share freely, never taking more than you give.\n*/\n";
        fs::write(
            root.join("sqlite3/sqlite3.c"),
            format!("{header}#ifndef SQLITEINT_H\nimplementation"),
        )
        .unwrap();
        assert_eq!(sqlite_notice(&root).unwrap(), header);
    }

    #[test]
    fn rust_notices_ring_preserves_distinct_source_grants_and_requires_reviewed_headers() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        for path in [
            "LICENSE",
            "LICENSE-other-bits",
            "LICENSE-BoringSSL",
            "third_party/fiat/LICENSE",
            "src/polyfill/once_cell/LICENSE-APACHE",
            "src/polyfill/once_cell/LICENSE-MIT",
        ] {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "Required crate license body").unwrap();
        }
        fs::write(root.join("Cargo.toml"), "[package]\nname='ring'\n").unwrap();
        let package = Package {
            id: "ring".into(),
            name: "ring".into(),
            version: "0.17.14".into(),
            source: None,
            manifest_path: root.join("Cargo.toml"),
            license: Some("ISC".into()),
            license_file: None,
        };
        for (path, copyright) in [
            ("src/limb.rs", "Copyright 2016 David Judd."),
            ("src/debug.rs", "Copyright 2018 Trent Clarke."),
            ("src/digest/sha1.rs", "Copyright 2016 Simon Sapin."),
            (
                "crypto/poly1305/poly1305.c",
                "Copyright (c) 2014, Google Inc.",
            ),
            (
                "crypto/fipsmodule/ec/ecp_nistz.c",
                "Copyright (c) 2014, Intel Corporation.",
            ),
        ] {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            let header = if path.extension().unwrap() == "rs" {
                format!("// {copyright}\n// Permission text and complete disclaimer.\n")
            } else {
                format!("/* {copyright}\n * Permission text and complete disclaimer. */")
            };
            fs::write(&path, format!("{header}\n\nimplementation_code();")).unwrap();
            assert_eq!(ring_source_notice(&path).unwrap().unwrap(), header);
        }
        let files = package_files(&root, &package, "unused", &[], &mut 0).unwrap();
        let mut rendered = String::new();
        append_files(&mut rendered, &files).unwrap();
        for expected in [
            "David Judd",
            "Trent Clarke",
            "Simon Sapin",
            "Google Inc.",
            "Intel Corporation.",
        ] {
            assert!(rendered.contains(expected));
        }
        assert!(!rendered.contains("implementation_code"));
        fs::write(
            root.join("src/limb.rs"),
            "implementation_code();\n// Copyright hidden after code",
        )
        .unwrap();
        assert!(
            format!(
                "{:#}",
                package_files(&root, &package, "unused", &[], &mut 0).unwrap_err()
            )
            .contains("src/limb.rs (source header)")
        );
        let perl = root.join("generator.pl");
        let header = "#!/usr/bin/env perl\n# Copyright CloudFlare\n# Complete grant.\n";
        fs::write(&perl, format!("{header}\nmy $code = 1;\n")).unwrap();
        assert_eq!(ring_source_notice(&perl).unwrap().unwrap(), header);
    }
}
