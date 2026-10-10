use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use crate::{
    catalog::{Catalog, Payload, Skill, payload_hash, read_payload_directory, safe_path},
    paths::canonical_project,
    util::{atomic_write, ensure_directory, read_bounded, reject_symlink_components},
};

const CONFIG: &str = ".mirket/project.json";
const RECEIPT: &str = ".mirket-skill.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub schema_version: u32,
    pub targets: Vec<String>,
    pub profiles: Vec<String>,
    pub skills: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub target_skills: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema_version: u32,
    skill: String,
    sha256: String,
    executable_files: BTreeSet<String>,
}

struct ProjectLock {
    _lock: crate::util::FileLock,
}
impl ProjectLock {
    fn acquire(root: &Path) -> Result<Self> {
        let directory = root.join(".mirket");
        ensure_directory(&directory)?;
        let lock = crate::util::FileLock::acquire(&directory.join("project.lock"))
            .context("another mirket project operation is running")?;
        Ok(Self { _lock: lock })
    }
}

pub fn target_skills_dir(target: &str) -> Result<&'static str> {
    match target {
        "codex" => Ok(".agents/skills"),
        "claude" => Ok(".claude/skills"),
        _ => bail!("unknown target: {target}"),
    }
}

pub fn load(catalog: &Catalog, project: &Path) -> Result<(PathBuf, ProjectConfig)> {
    let root = canonical_project(project)?;
    let path = root.join(CONFIG);
    let config: ProjectConfig = serde_json::from_slice(
        &read_bounded(&path, 256 * 1024)
            .context("project is not configured; run mirket project init")?,
    )?;
    validate_config(catalog, &config)?;
    Ok((root, config))
}

fn validate_config(catalog: &Catalog, config: &ProjectConfig) -> Result<()> {
    ensure!(config.schema_version == 1, "unsupported project schema");
    ensure!(
        !config.targets.is_empty() && config.targets.len() <= 2,
        "select codex, claude or both"
    );
    let targets: BTreeSet<_> = config.targets.iter().collect();
    ensure!(
        targets.len() == config.targets.len(),
        "duplicate project targets"
    );
    for target in config.target_skills.keys() {
        ensure!(
            targets.contains(target),
            "target skill selection has no project target: {target}"
        );
    }
    ensure!(
        config.skills.len() <= 512 && config.profiles.len() <= 100,
        "project selection exceeds limits"
    );
    for target in &config.targets {
        target_skills_dir(target)?;
        let ids = selected_ids(config, target);
        ensure!(ids.len() <= 512, "target selection exceeds limits");
        catalog.selection(&config.profiles, &ids, target)?;
    }
    Ok(())
}

fn selected_ids(config: &ProjectConfig, target: &str) -> Vec<String> {
    let mut ids = config.skills.clone();
    if let Some(specific) = config.target_skills.get(target) {
        ids.extend(specific.iter().cloned());
    }
    ids
}

pub fn init(catalog: &Catalog, project: &Path, targets: &[String]) -> Result<Value> {
    let root = canonical_project(project)?;
    let _lock = ProjectLock::acquire(&root)?;
    let config_path = root.join(CONFIG);
    reject_symlink_components(&config_path)?;
    ensure!(
        !config_path.exists(),
        "project is already configured: {}",
        config_path.display()
    );
    let config = ProjectConfig {
        schema_version: 1,
        targets: if targets.is_empty() {
            vec!["codex".into(), "claude".into()]
        } else {
            targets.into()
        },
        profiles: vec!["project-foundation".into()],
        skills: vec![],
        target_skills: BTreeMap::new(),
    };
    validate_config(catalog, &config)?;
    atomic_write(&config_path, &serde_json::to_vec_pretty(&config)?)?;
    Ok(
        json!({"ok":true,"status":"configured","project":root,"config":config,
        "next":"mirket project sync; mirket project doctor"}),
    )
}

pub fn add(
    catalog: &Catalog,
    project: &Path,
    profiles: &[String],
    skills: &[String],
    capabilities: &[String],
    target: Option<&str>,
) -> Result<Value> {
    let (root, _) = load(catalog, project)?;
    let _lock = ProjectLock::acquire(&root)?;
    // Read after acquiring the writer lock so concurrent CLI operations cannot lose selections.
    let mut config = load(catalog, &root)?.1;
    let mut ids = skills.to_vec();
    for name in capabilities {
        ids.push(catalog.capability(name)?.lead.clone());
    }
    ids.retain(|id| !catalog.global_skills().contains(id));
    if let Some(target) = target {
        ensure!(
            config.targets.iter().any(|candidate| candidate == target),
            "target is not configured: {target}"
        );
        for profile in profiles {
            let profile = catalog
                .profiles()
                .get(profile)
                .with_context(|| format!("unknown profile: {profile}"))?;
            ids.extend(
                profile
                    .skills
                    .iter()
                    .filter(|id| !catalog.global_skills().contains(id))
                    .cloned(),
            );
        }
        let specific = config.target_skills.entry(target.into()).or_default();
        for id in ids {
            if !specific.contains(&id) && !config.skills.contains(&id) {
                specific.push(id);
            }
        }
    } else {
        for profile in profiles {
            if !config.profiles.contains(profile) {
                config.profiles.push(profile.clone());
            }
        }
        for id in ids {
            if !config.skills.contains(&id) {
                config.skills.push(id);
            }
        }
        for specific in config.target_skills.values_mut() {
            specific.retain(|id| !config.skills.contains(id));
        }
        config.target_skills.retain(|_, ids| !ids.is_empty());
    }
    validate_config(catalog, &config)?;
    atomic_write(&root.join(CONFIG), &serde_json::to_vec_pretty(&config)?)?;
    Ok(
        json!({"ok":true,"status":"configured","project":root,"config":config,
        "next":"mirket project sync; mirket project doctor"}),
    )
}

pub fn set_targets(catalog: &Catalog, project: &Path, targets: &[String]) -> Result<Value> {
    let (root, _) = load(catalog, project)?;
    let _lock = ProjectLock::acquire(&root)?;
    let mut config = load(catalog, &root)?.1;
    config.targets = targets.into();
    config
        .target_skills
        .retain(|target, _| targets.contains(target));
    validate_config(catalog, &config)?;
    atomic_write(&root.join(CONFIG), &serde_json::to_vec_pretty(&config)?)?;
    Ok(
        json!({"ok":true,"status":"configured","project":root,"config":config,
        "next":"mirket project sync --prune; mirket project doctor"}),
    )
}

pub fn remove(
    catalog: &Catalog,
    project: &Path,
    profiles: &[String],
    skills: &[String],
    capabilities: &[String],
    target: Option<&str>,
) -> Result<Value> {
    ensure!(
        !profiles.is_empty() || !skills.is_empty() || !capabilities.is_empty(),
        "select a profile, skill or capability to remove"
    );
    let (root, _) = load(catalog, project)?;
    let _lock = ProjectLock::acquire(&root)?;
    let mut config = load(catalog, &root)?.1;
    let mut ids: BTreeSet<String> = skills.iter().cloned().collect();
    for name in capabilities {
        ids.insert(catalog.capability(name)?.lead.clone());
    }
    for id in &ids {
        catalog.skill(id)?;
    }
    for profile in profiles {
        ensure!(
            catalog.profiles().contains_key(profile),
            "unknown profile: {profile}"
        );
    }
    if let Some(target) = target {
        ensure!(
            config.targets.iter().any(|candidate| candidate == target),
            "target is not configured: {target}"
        );
        for profile in profiles {
            ids.extend(catalog.profiles()[profile].skills.iter().cloned());
        }
        if let Some(specific) = config.target_skills.get_mut(target) {
            specific.retain(|id| !ids.contains(id));
        }
    } else {
        config
            .profiles
            .retain(|profile| !profiles.contains(profile));
        config.skills.retain(|id| !ids.contains(id));
        for specific in config.target_skills.values_mut() {
            specific.retain(|id| !ids.contains(id));
        }
    }
    config.target_skills.retain(|_, ids| !ids.is_empty());
    validate_config(catalog, &config)?;
    atomic_write(&root.join(CONFIG), &serde_json::to_vec_pretty(&config)?)?;
    let mut retained_by_selection = BTreeMap::new();
    for target in &config.targets {
        let retained: Vec<_> = catalog
            .selection(&config.profiles, &selected_ids(&config, target), target)?
            .into_iter()
            .filter(|skill| ids.contains(&skill.id))
            .map(|skill| skill.id.clone())
            .collect();
        if !retained.is_empty() {
            retained_by_selection.insert(target, retained);
        }
    }
    Ok(
        json!({"ok":true,"status":"configured","project":root,"config":config,
        "retained_by_selection":retained_by_selection,
        "meaning":"Exact declared selectors removed. Profile-owned and required companion skills remain selected by their owners.",
        "next":"mirket project sync --prune; mirket project doctor"}),
    )
}

fn destination(root: &Path, target: &str, skill: &Skill) -> Result<PathBuf> {
    safe_path(&skill.name)?;
    ensure!(
        !skill.name.contains('/'),
        "skill name must be one directory component"
    );
    let path = root.join(target_skills_dir(target)?).join(&skill.name);
    reject_symlink_components(&path)?;
    Ok(path)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum InstalledState {
    Missing,
    Managed(String, BTreeSet<String>),
    Unmanaged,
}

fn state(path: &Path, skill: &Skill) -> Result<InstalledState> {
    reject_symlink_components(path)?;
    if !path.exists() {
        return Ok(InstalledState::Missing);
    }
    ensure!(
        path.is_dir(),
        "skill destination is not a directory: {}",
        path.display()
    );
    let receipt_path = path.join(RECEIPT);
    if !receipt_path.exists() {
        return Ok(InstalledState::Unmanaged);
    }
    let receipt = inspect_receipt(path)?;
    ensure!(
        receipt.skill == skill.id,
        "skill receipt identity differs: {}",
        path.display()
    );
    Ok(InstalledState::Managed(
        receipt.sha256,
        receipt.executable_files,
    ))
}

fn inspect_receipt(path: &Path) -> Result<Receipt> {
    reject_symlink_components(path)?;
    let receipt: Receipt = serde_json::from_slice(&read_bounded(&path.join(RECEIPT), 64 * 1024)?)?;
    ensure!(receipt.schema_version == 1, "unsupported skill receipt");
    crate::catalog::validate_id(&receipt.skill)?;
    let files = read_payload_directory(path, &[RECEIPT])?;
    let digest = payload_hash(&files);
    ensure!(
        digest == receipt.sha256,
        "managed skill was modified: {}",
        path.display()
    );
    check_executables(path, &files, &receipt.executable_files)?;
    Ok(receipt)
}

#[derive(Serialize)]
struct Unselected {
    path: PathBuf,
    target: String,
    skill: Option<String>,
    digest: Option<String>,
    error: Option<String>,
}

fn unselected(catalog: &Catalog, root: &Path, config: &ProjectConfig) -> Result<Vec<Unselected>> {
    let mut selected = BTreeSet::new();
    for target in &config.targets {
        for skill in catalog.selection(&config.profiles, &selected_ids(config, target), target)? {
            selected.insert(destination(root, target, skill)?);
        }
    }
    let mut copies = Vec::new();
    for target in ["codex", "claude"] {
        let directory = root.join(target_skills_dir(target)?);
        reject_symlink_components(&directory)?;
        if !directory.exists() {
            continue;
        }
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            let path = entry.path();
            if selected.contains(&path) || !entry.file_type()?.is_dir() {
                continue;
            }
            if fs::symlink_metadata(path.join(RECEIPT)).is_err() {
                continue;
            }
            let (skill, digest, error) = match inspect_receipt(&path) {
                Ok(receipt) => (Some(receipt.skill), Some(receipt.sha256), None),
                Err(error) => (None, None, Some(format!("{error:#}"))),
            };
            copies.push(Unselected {
                path,
                target: target.into(),
                skill,
                digest,
                error,
            });
        }
    }
    copies.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(copies)
}

fn desired_digest(catalog: &Catalog, skill: &Skill) -> Result<String> {
    if skill.delivery == "embedded" {
        Ok(catalog.payload_digest(&skill.id)?.to_owned())
    } else {
        skill
            .installed_sha256
            .clone()
            .or_else(|| skill.sha256.clone())
            .context("missing skill digest")
    }
}

fn check_name_collision(root: &Path, target: &str, skill: &Skill) -> Result<()> {
    let directory = root.join(target_skills_dir(target)?);
    reject_symlink_components(&directory)?;
    if !directory.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_name() == skill.name.as_str() {
            continue;
        }
        let body = entry.path().join("SKILL.md");
        // Unrelated symlinked directories are not followed or altered.
        if entry.file_type()?.is_symlink() || !body.exists() {
            continue;
        }
        let bytes = read_bounded(&body, 256 * 1024)?;
        let text = std::str::from_utf8(&bytes).context("existing skill is not UTF-8")?;
        let name = text
            .lines()
            .skip(1)
            .take_while(|line| *line != "---")
            .find_map(|line| line.strip_prefix("name:"))
            .map(|s| s.trim().trim_matches(['\'', '"']));
        ensure!(
            name != Some(&skill.name),
            "skill invocation already exists in {}",
            entry.path().display()
        );
    }
    Ok(())
}

pub fn plan(catalog: &Catalog, project: &Path) -> Result<Value> {
    let (root, config) = load(catalog, project)?;
    let mut jobs = Vec::new();
    let mut conflicts = 0;
    for target in &config.targets {
        for skill in catalog.selection(&config.profiles, &selected_ids(&config, target), target)? {
            let expected = desired_digest(catalog, skill)?;
            let inspection = (|| {
                let path = destination(&root, target, skill)?;
                check_name_collision(&root, target, skill)?;
                let current = state(&path, skill)?;
                let status = match current {
                    InstalledState::Missing => "install",
                    InstalledState::Managed(ref digest, ref executable_files)
                        if digest == &expected && executable_files == &skill.executable_files =>
                    {
                        "current"
                    }
                    InstalledState::Managed(_, _) => "update",
                    InstalledState::Unmanaged => {
                        bail!("unmanaged skill exists: {}", path.display())
                    }
                };
                Ok::<_, anyhow::Error>(
                    json!({"skill":skill.id,"target":target,"path":path,"status":status,"sha256":expected}),
                )
            })();
            match inspection {
                Ok(row) => jobs.push(row),
                Err(error) => {
                    conflicts += 1;
                    jobs.push(json!({"skill":skill.id,"target":target,"status":"conflict","error":format!("{error:#}")}));
                }
            }
        }
    }
    let unselected = unselected(catalog, &root, &config)?;
    Ok(
        json!({"ok":conflicts == 0,"status":if conflicts == 0 {"ready"} else {"conflicts"}, "project":root,
        "targets":config.targets,"jobs":jobs,"conflicts":conflicts,"unselected":unselected}),
    )
}

struct PreparedJob {
    path: PathBuf,
    skill: String,
    target: String,
    expected_before: InstalledState,
    payload: Payload,
}

struct Published {
    path: PathBuf,
    skill: String,
    digest: Option<String>,
    executable_files: BTreeSet<String>,
    backup: Option<PathBuf>,
    staging: tempfile::TempDir,
}

pub fn sync(catalog: &Catalog, project: &Path, cache: &Path) -> Result<Value> {
    sync_with_options(catalog, project, cache, false)
}

pub fn sync_with_options(
    catalog: &Catalog,
    project: &Path,
    cache: &Path,
    prune: bool,
) -> Result<Value> {
    let (root, _) = load(catalog, project)?;
    let _lock = ProjectLock::acquire(&root)?;
    let (_, config) = load(catalog, &root)?;
    let unselected = unselected(catalog, &root, &config)?;
    if prune {
        for copy in &unselected {
            ensure!(
                copy.error.is_none(),
                "cannot prune {}: {}",
                copy.path.display(),
                copy.error.as_deref().unwrap_or("invalid managed copy")
            );
        }
    }
    let mut prepared = BTreeMap::new();
    let mut jobs = Vec::new();
    let mut results = Vec::new();
    // Detect all conflicts before downloading or changing any project skill.
    for target in &config.targets {
        for skill in catalog.selection(&config.profiles, &selected_ids(&config, target), target)? {
            let path = destination(&root, target, skill)?;
            check_name_collision(&root, target, skill)?;
            let before = state(&path, skill)?;
            ensure!(
                before != InstalledState::Unmanaged,
                "refusing unmanaged skill: {}",
                path.display()
            );
            let expected = desired_digest(catalog, skill)?;
            if before == InstalledState::Managed(expected, skill.executable_files.clone()) {
                results.push(json!({"skill":skill.id,"target":target,"status":"current"}));
                continue;
            }
            jobs.push((skill, target, path, before));
        }
    }
    let mut ready = Vec::new();
    for (skill, target, path, before) in jobs {
        if !prepared.contains_key(&skill.id) {
            prepared.insert(skill.id.clone(), catalog.prepare(&skill.id, cache)?);
        }
        ready.push(PreparedJob {
            path,
            skill: skill.id.clone(),
            target: target.clone(),
            expected_before: before,
            payload: prepared
                .get(&skill.id)
                .context("prepared payload missing")?
                .clone(),
        });
    }
    let mut published: Vec<Published> = Vec::new();
    let publication = (|| -> Result<()> {
        if prune {
            for copy in &unselected {
                let receipt = inspect_receipt(&copy.path)?;
                ensure!(
                    Some(&receipt.sha256) == copy.digest.as_ref(),
                    "unselected skill changed during synchronization"
                );
                let staging =
                    tempfile::tempdir_in(copy.path.parent().context("skill has no parent")?)?;
                let backup = staging.path().join("before");
                fs::rename(&copy.path, &backup)?;
                results
                    .push(json!({"skill":receipt.skill,"target":copy.target,"status":"removed"}));
                published.push(Published {
                    path: copy.path.clone(),
                    skill: receipt.skill,
                    digest: None,
                    executable_files: receipt.executable_files,
                    backup: Some(backup),
                    staging,
                });
            }
        }
        for job in ready {
            let skill = catalog.skill(&job.skill)?;
            ensure!(
                state(&job.path, skill)? == job.expected_before,
                "skill changed during synchronization"
            );
            let parent = job.path.parent().context("skill has no parent")?;
            ensure_directory(parent)?;
            let staging = tempfile::tempdir_in(parent)?;
            let next = staging.path().join("next");
            ensure_directory(&next)?;
            for (relative, bytes) in &job.payload.files {
                let path = next.join(relative);
                atomic_write(&path, bytes)?;
                set_executable(&path, job.payload.executables.contains(relative))?;
            }
            let receipt = Receipt {
                schema_version: 1,
                skill: job.skill.clone(),
                sha256: job.payload.sha256.clone(),
                executable_files: job.payload.executables.clone(),
            };
            atomic_write(&next.join(RECEIPT), &serde_json::to_vec_pretty(&receipt)?)?;
            ensure!(
                state(&next, skill)?
                    == InstalledState::Managed(job.payload.sha256, job.payload.executables),
                "staged skill validation failed"
            );
            // Recheck immediately before publication, after all potentially slow staging work.
            ensure!(
                state(&job.path, skill)? == job.expected_before,
                "skill changed during synchronization"
            );
            let backup = if job.path.exists() {
                let backup = staging.path().join("before");
                fs::rename(&job.path, &backup)?;
                Some(backup)
            } else {
                None
            };
            if let Err(error) = fs::rename(&next, &job.path) {
                if let Some(backup) = &backup
                    && let Err(restore) = fs::rename(backup, &job.path)
                {
                    let kept = staging.keep();
                    bail!(
                        "publish failed: {error}; restore failed: {restore}; original retained in {}",
                        kept.display()
                    );
                }
                return Err(error.into());
            }
            results.push(json!({"skill":job.skill,"target":job.target,"status":"installed"}));
            published.push(Published {
                path: job.path,
                skill: job.skill,
                digest: Some(receipt.sha256),
                executable_files: receipt.executable_files,
                backup,
                staging,
            });
        }
        Ok(())
    })();
    if let Err(error) = publication {
        let mut rollback_errors = Vec::new();
        for publication in published.into_iter().rev() {
            let matches = match &publication.digest {
                Some(digest) => inspect_receipt(&publication.path).is_ok_and(|receipt| {
                    receipt.sha256 == *digest
                        && receipt.skill == publication.skill
                        && receipt.executable_files == publication.executable_files
                }),
                None => fs::symlink_metadata(&publication.path)
                    .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound),
            };
            if !matches {
                let kept = publication.staging.keep();
                rollback_errors.push(format!(
                    "published skill changed; preserved {} and recovery files in {}",
                    publication.path.display(),
                    kept.display()
                ));
                continue;
            }
            if publication.digest.is_some()
                && let Err(error) = fs::remove_dir_all(&publication.path)
            {
                let kept = publication.staging.keep();
                rollback_errors.push(format!(
                    "remove published skill: {error}; recovery files retained in {}",
                    kept.display()
                ));
                continue;
            }
            if let Some(backup) = &publication.backup
                && let Err(error) = fs::rename(backup, &publication.path)
            {
                let kept = publication.staging.keep();
                rollback_errors.push(format!(
                    "restore {}: {error}; retained {}",
                    publication.path.display(),
                    kept.display()
                ));
            }
        }
        if !rollback_errors.is_empty() {
            bail!(
                "sync failed: {error:#}; rollback issues: {}",
                rollback_errors.join("; ")
            );
        }
        return Err(error);
    }
    Ok(
        json!({"ok":true,"status":"synced","project":root,"skills":results,"unselected":if prune {Vec::<Unselected>::new()} else {unselected},
        "runtime_status":"not_checked","activation_status":"not_observed"}),
    )
}

pub fn doctor(catalog: &Catalog, project: &Path) -> Result<Value> {
    let mut result = plan(catalog, project)?;
    let jobs = result["jobs"].as_array().context("invalid project plan")?;
    let mut issues: Vec<Value> = jobs
        .iter()
        .filter(|job| job["status"] != "current")
        .cloned()
        .collect();
    for copy in result["unselected"]
        .as_array()
        .context("invalid unselected report")?
    {
        let mut issue = copy.clone();
        issue["status"] = json!("unselected");
        issue["next"] = json!("Inspect the managed copy, then run mirket project sync --prune.");
        issues.push(issue);
    }
    result["ok"] = json!(issues.is_empty());
    result["status"] = json!(if issues.is_empty() {
        "healthy"
    } else {
        "issues"
    });
    result["issues"] = json!(issues);
    result["runtime_status"] = json!("not_checked");
    result["activation_status"] = json!("not_observed");
    Ok(result)
}

fn set_executable(path: &Path, executable: bool) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(
            path,
            fs::Permissions::from_mode(if executable { 0o755 } else { 0o644 }),
        )?;
    }
    #[cfg(not(unix))]
    {
        let _ = (path, executable);
    }
    Ok(())
}

fn check_executables(
    path: &Path,
    files: &BTreeMap<String, Vec<u8>>,
    expected: &BTreeSet<String>,
) -> Result<()> {
    for executable in expected {
        ensure!(
            files.contains_key(executable),
            "missing recorded executable: {executable}"
        );
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut found = BTreeSet::new();
        for name in files.keys() {
            if fs::metadata(path.join(name))?.permissions().mode() & 0o111 != 0 {
                found.insert(name.clone());
            }
        }
        ensure!(
            found == *expected,
            "managed executable modes differ: {}",
            path.display()
        );
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foundation_and_capabilities_install_exact_target_membership() {
        let catalog = Catalog::embedded().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let cache = temp.path().join("cache");
        init(&catalog, temp.path(), &["codex".into(), "claude".into()]).unwrap();
        add(
            &catalog,
            temp.path(),
            &[],
            &[],
            &["CFO".into()],
            Some("codex"),
        )
        .unwrap();
        sync(&catalog, temp.path(), &cache).unwrap();
        let lead = &catalog.capability("CFO").unwrap().lead;
        let name = &catalog.skill(lead).unwrap().name;
        assert!(
            temp.path()
                .join(".agents/skills")
                .join(name)
                .join("SKILL.md")
                .is_file()
        );
        assert!(!temp.path().join(".claude/skills").join(name).exists());
        assert!(!temp.path().join(".agents/skills/skill-catalog").exists());
        assert_eq!(doctor(&catalog, temp.path()).unwrap()["status"], "healthy");
    }

    #[test]
    fn drift_stops_all_writes_and_unrelated_skills_survive() {
        let catalog = Catalog::embedded().unwrap();
        let temp = tempfile::tempdir().unwrap();
        init(&catalog, temp.path(), &["codex".into()]).unwrap();
        sync(&catalog, temp.path(), &temp.path().join("cache")).unwrap();
        let body = temp.path().join(".agents/skills/debugging/SKILL.md");
        fs::write(&body, "my change").unwrap();
        let unrelated = temp.path().join(".agents/skills/custom/notes.txt");
        fs::create_dir_all(unrelated.parent().unwrap()).unwrap();
        fs::write(&unrelated, "keep").unwrap();
        add(
            &catalog,
            temp.path(),
            &[],
            &["performance-engineering".into()],
            &[],
            None,
        )
        .unwrap();
        assert!(sync(&catalog, temp.path(), &temp.path().join("cache")).is_err());
        assert_eq!(fs::read_to_string(body).unwrap(), "my change");
        assert_eq!(fs::read_to_string(unrelated).unwrap(), "keep");
        assert!(
            !temp
                .path()
                .join(".agents/skills/performance-engineering")
                .exists()
        );
        assert_eq!(doctor(&catalog, temp.path()).unwrap()["status"], "issues");
    }

    #[test]
    fn invalid_selection_does_not_change_manifest() {
        let catalog = Catalog::embedded().unwrap();
        let temp = tempfile::tempdir().unwrap();
        init(&catalog, temp.path(), &["claude".into()]).unwrap();
        let before = fs::read(temp.path().join(CONFIG)).unwrap();
        assert!(add(&catalog, temp.path(), &[], &["missing".into()], &[], None).is_err());
        assert_eq!(fs::read(temp.path().join(CONFIG)).unwrap(), before);
        assert!(
            add(
                &catalog,
                temp.path(),
                &[],
                &["debugging".into()],
                &[],
                Some("codex")
            )
            .is_err()
        );
        assert_eq!(fs::read(temp.path().join(CONFIG)).unwrap(), before);
    }

    #[test]
    fn selection_removal_requires_explicit_prune_and_preserves_unmanaged_copies() {
        let catalog = Catalog::embedded().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let cache = temp.path().join("cache");
        init(&catalog, temp.path(), &["codex".into(), "claude".into()]).unwrap();
        add(
            &catalog,
            temp.path(),
            &[],
            &["performance-engineering".into()],
            &[],
            None,
        )
        .unwrap();
        sync(&catalog, temp.path(), &cache).unwrap();
        remove(
            &catalog,
            temp.path(),
            &[],
            &["performance-engineering".into()],
            &[],
            None,
        )
        .unwrap();
        let unused = temp.path().join(".agents/skills/performance-engineering");
        sync(&catalog, temp.path(), &cache).unwrap();
        assert!(unused.exists());
        assert_eq!(doctor(&catalog, temp.path()).unwrap()["ok"], false);
        let custom = temp.path().join(".agents/skills/custom");
        fs::create_dir(&custom).unwrap();
        fs::write(custom.join("notes.txt"), "owned by user").unwrap();
        sync_with_options(&catalog, temp.path(), &cache, true).unwrap();
        assert!(!unused.exists());
        assert!(custom.join("notes.txt").exists());
        assert_eq!(doctor(&catalog, temp.path()).unwrap()["ok"], true);
        set_targets(&catalog, temp.path(), &["codex".into()]).unwrap();
        assert_eq!(doctor(&catalog, temp.path()).unwrap()["ok"], false);
        sync_with_options(&catalog, temp.path(), &cache, true).unwrap();
        assert!(!temp.path().join(".claude/skills/debugging").exists());
        assert!(temp.path().join(".agents/skills/debugging").exists());
        assert_eq!(doctor(&catalog, temp.path()).unwrap()["ok"], true);
    }

    #[test]
    fn modified_unselected_copy_blocks_prune_before_new_writes() {
        let catalog = Catalog::embedded().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let cache = temp.path().join("cache");
        init(&catalog, temp.path(), &["codex".into()]).unwrap();
        add(
            &catalog,
            temp.path(),
            &[],
            &["performance-engineering".into()],
            &[],
            None,
        )
        .unwrap();
        sync(&catalog, temp.path(), &cache).unwrap();
        remove(
            &catalog,
            temp.path(),
            &[],
            &["performance-engineering".into()],
            &[],
            None,
        )
        .unwrap();
        let body = temp
            .path()
            .join(".agents/skills/performance-engineering/SKILL.md");
        fs::write(&body, "user modification").unwrap();
        add(
            &catalog,
            temp.path(),
            &[],
            &["database-systems".into()],
            &[],
            None,
        )
        .unwrap();
        assert!(sync_with_options(&catalog, temp.path(), &cache, true).is_err());
        assert_eq!(fs::read_to_string(body).unwrap(), "user modification");
        assert!(!temp.path().join(".agents/skills/database-systems").exists());
        assert_eq!(doctor(&catalog, temp.path()).unwrap()["ok"], false);
    }

    #[test]
    fn profile_owned_selection_is_reported_after_removing_a_skill_selector() {
        let catalog = Catalog::embedded().unwrap();
        let temp = tempfile::tempdir().unwrap();
        init(&catalog, temp.path(), &["codex".into()]).unwrap();
        let report = remove(&catalog, temp.path(), &[], &["debugging".into()], &[], None).unwrap();
        assert_eq!(
            report["retained_by_selection"]["codex"],
            json!(["debugging"])
        );
        remove(
            &catalog,
            temp.path(),
            &["project-foundation".into()],
            &[],
            &[],
            None,
        )
        .unwrap();
        assert_eq!(plan(&catalog, temp.path()).unwrap()["jobs"], json!([]));
    }

    #[cfg(unix)]
    #[test]
    fn current_catalog_modes_are_checked_even_if_local_receipt_was_edited() {
        use std::os::unix::fs::PermissionsExt;
        let catalog = Catalog::embedded().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let cache = temp.path().join("cache");
        init(&catalog, temp.path(), &["codex".into()]).unwrap();
        sync(&catalog, temp.path(), &cache).unwrap();
        let root = temp.path().join(".agents/skills/debugging");
        fs::set_permissions(root.join("SKILL.md"), fs::Permissions::from_mode(0o755)).unwrap();
        let mut receipt: Receipt =
            serde_json::from_slice(&fs::read(root.join(RECEIPT)).unwrap()).unwrap();
        receipt.executable_files.insert("SKILL.md".into());
        fs::write(root.join(RECEIPT), serde_json::to_vec(&receipt).unwrap()).unwrap();
        assert_eq!(doctor(&catalog, temp.path()).unwrap()["ok"], false);
        sync(&catalog, temp.path(), &cache).unwrap();
        assert_eq!(
            fs::metadata(root.join("SKILL.md"))
                .unwrap()
                .permissions()
                .mode()
                & 0o111,
            0
        );
        assert_eq!(doctor(&catalog, temp.path()).unwrap()["ok"], true);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_target_and_executable_drift_fail_closed() {
        use std::os::unix::{fs::PermissionsExt, fs::symlink};
        let catalog = Catalog::embedded().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        init(&catalog, temp.path(), &["codex".into()]).unwrap();
        symlink(external.path(), temp.path().join(".agents")).unwrap();
        assert!(sync(&catalog, temp.path(), &temp.path().join("cache")).is_err());
        assert_eq!(fs::read_dir(external.path()).unwrap().count(), 0);
        fs::remove_file(temp.path().join(".agents")).unwrap();
        sync(&catalog, temp.path(), &temp.path().join("cache")).unwrap();
        fs::set_permissions(
            temp.path().join(".agents/skills/debugging/SKILL.md"),
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        assert!(sync(&catalog, temp.path(), &temp.path().join("cache")).is_err());
    }
}
