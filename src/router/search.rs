//! Bounded source discovery with explicit matching and project-local Git ignore rules.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;
use std::sync::Arc;

#[derive(Clone, Copy, Default, Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum Mode {
    #[default]
    Literal,
    Regex,
}

struct Rules {
    matcher: ignore::gitignore::Gitignore,
    parent: Option<Arc<Rules>>,
}

fn relative_path(project: &Path, path: &Path) -> Result<String> {
    Ok(path
        .strip_prefix(project)?
        .iter()
        .map(|part| part.to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

fn ignored(rules: &Option<Arc<Rules>>, path: &Path, directory: bool) -> bool {
    let mut current = rules.as_ref();
    while let Some(rules) = current {
        let matched = rules.matcher.matched(path, directory);
        if !matched.is_none() {
            return matched.is_ignore();
        }
        current = rules.parent.as_ref();
    }
    false
}

fn load_rules(
    project: &Path,
    directory: &Path,
    parent: Option<Arc<Rules>>,
    bytes: &mut u64,
    receipts: &mut Vec<Value>,
) -> Result<Option<Arc<Rules>>> {
    let path = directory.join(".gitignore");
    match std::fs::symlink_metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(parent),
        other => {
            other?;
        }
    }
    crate::util::reject_symlink_components(&path)?;
    let content = crate::util::read_bounded(&path, (1024 * 1024 - *bytes).min(65536))?;
    *bytes += content.len() as u64;
    let mut builder = ignore::gitignore::GitignoreBuilder::new(directory);
    for line in std::str::from_utf8(&content)?.lines() {
        builder.add_line(Some(path.clone()), line)?;
    }
    receipts.push(
        json!({"path":relative_path(project, &path)?,"sha256":crate::util::sha256(&content)}),
    );
    Ok(Some(Arc::new(Rules {
        matcher: builder.build()?,
        parent,
    })))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    query: String,
    #[serde(default)]
    mode: Mode,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
}

pub fn search(project: &Path, args: Value) -> Result<Value> {
    let args: Args = serde_json::from_value(args)?;
    ensure!(
        !args.query.is_empty() && args.query.len() <= 4096 && !args.query.contains(['\n', '\r']),
        "query requires 1..4096 bytes on one line"
    );
    let limit = args.limit.unwrap_or(100);
    ensure!((1..=200).contains(&limit), "limit requires 1..200 matches");
    let regex = match args.mode {
        Mode::Literal => None,
        Mode::Regex => Some(
            regex::RegexBuilder::new(&args.query)
                .size_limit(2 * 1024 * 1024)
                .dfa_size_limit(2 * 1024 * 1024)
                .nest_limit(64)
                .build()
                .context("invalid or oversized search regex")?,
        ),
    };
    let root = match args.path.as_deref() {
        None | Some(".") => project.to_owned(),
        Some(path) => super::tools::file(project, path)?,
    };
    std::fs::symlink_metadata(&root).context("search path does not exist")?;
    let mut ignore_bytes = 0;
    let mut ignore_files = Vec::new();
    let mut rules = None;
    let mut ancestors = Vec::new();
    let mut ancestor = root.as_path();
    while ancestor != project {
        ancestor = ancestor.parent().context("search ancestor")?;
        ancestors.push(ancestor);
    }
    let mut excluded_ancestor = false;
    for directory in ancestors.into_iter().rev() {
        if ignored(&rules, directory, true) {
            excluded_ancestor = true;
            break;
        }
        rules = load_rules(
            project,
            directory,
            rules,
            &mut ignore_bytes,
            &mut ignore_files,
        )?;
    }
    let mut pending = vec![(root, rules)];
    let mut matches = Vec::new();
    let mut skipped = Vec::new();
    let mut skipped_count = 0;
    let mut entries = 0;
    let mut bytes = 0;
    let mut truncated = false;
    while let Some((path, rules)) = pending.pop() {
        entries += 1;
        if entries > 10_000 || bytes >= 32 * 1024 * 1024 || matches.len() >= limit {
            truncated = true;
            break;
        }
        let relative = relative_path(project, &path)?;
        crate::util::reject_symlink_components(path.parent().context("search parent")?)?;
        let metadata = std::fs::symlink_metadata(&path)?;
        if excluded_ancestor || ignored(&rules, &path, metadata.is_dir()) {
            skipped_count += 1;
            if skipped.len() < 32 {
                skipped.push(json!({"path":relative,"reason":"gitignore"}));
            }
            continue;
        }
        if metadata.file_type().is_symlink() {
            skipped_count += 1;
            if skipped.len() < 32 {
                skipped.push(json!({"path":relative,"reason":"symlink"}));
            }
            continue;
        }
        if metadata.is_dir() {
            let rules = load_rules(project, &path, rules, &mut ignore_bytes, &mut ignore_files)?;
            let mut children = Vec::new();
            for entry in std::fs::read_dir(&path)? {
                let entry = entry?;
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name.eq_ignore_ascii_case(".git")
                    || ["node_modules", "target", ".venv", "__pycache__"].contains(&name.as_ref())
                {
                    continue;
                }
                ensure!(
                    entries + pending.len() + children.len() < 10_000,
                    "search frontier exceeds traversal limit; choose a narrower path"
                );
                children.push(entry.path());
            }
            children.sort();
            pending.extend(children.into_iter().rev().map(|path| (path, rules.clone())));
            continue;
        }
        if !metadata.is_file() || metadata.len() > 1024 * 1024 {
            skipped_count += 1;
            if skipped.len() < 32 {
                skipped.push(json!({"path":relative,"reason":"nonregular_or_over_1_MiB"}));
            }
            continue;
        }
        let remaining = 32 * 1024 * 1024 - bytes;
        if metadata.len() > remaining {
            truncated = true;
            break;
        }
        let content = crate::util::read_bounded(&path, remaining.min(1024 * 1024))?;
        bytes += content.len() as u64;
        let sha = crate::util::sha256(&content);
        let Ok(content) = std::str::from_utf8(&content) else {
            skipped_count += 1;
            if skipped.len() < 32 {
                skipped.push(json!({"path":relative,"reason":"non_utf8"}));
            }
            continue;
        };
        if content.contains('\0') {
            skipped_count += 1;
            if skipped.len() < 32 {
                skipped.push(json!({"path":relative,"reason":"binary"}));
            }
            continue;
        }
        for (line, text) in content.lines().enumerate() {
            let found = match &regex {
                Some(regex) => regex
                    .find(text)
                    .map(|matched| (matched.start(), matched.end())),
                None => text
                    .find(&args.query)
                    .map(|column| (column, column + args.query.len())),
            };
            if let Some((column, match_end)) = found {
                if matches.len() == limit {
                    truncated = true;
                    break;
                }
                // Bound every result, even for generated files with enormous lines.
                let mut start = column.saturating_sub(120);
                while !text.is_char_boundary(start) {
                    start += 1;
                }
                let mut end = (match_end.min(column + 4096) + 120).min(text.len());
                while !text.is_char_boundary(end) {
                    end -= 1;
                }
                matches.push(json!({"path":relative,"line":line+1,"byte_column":column+1,"match_bytes":match_end-column,"sha256":sha,"excerpt":&text[start..end],"excerpt_byte_offset":start}));
            }
        }
        if truncated {
            break;
        }
    }
    Ok(
        json!({"matches":matches,"truncated":truncated,"skipped":skipped,"skipped_count":skipped_count,"scanned_entries":entries,"scanned_bytes":bytes,
        "mode":args.mode,"ignore_files":ignore_files,"ignore_bytes":ignore_bytes,
        "coverage":"One match per line; UTF-8 files <=1 MiB, scan <=32 MiB, <=10000 entries; no symlinks; excludes .git, node_modules, target, .venv, __pycache__. Project-local .gitignore rules apply, including nested overrides; no global excludes, .git/info/exclude or files outside the project. Ignore files <=64 KiB each and <=1 MiB combined. Narrow path for further results. Search is discovery, not whole-artifact readback."}),
    )
}

#[cfg(test)]
include!("tests/search.rs");
