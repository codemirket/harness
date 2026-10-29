//! Release-bound pi skill catalog.
//!
//! A catalog is descriptive data. Reading it never installs a skill, starts a
//! tool, or grants a private capability. Setup and project selection use this
//! one inventory rather than deriving ownership from a name prefix.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

const MAX_CATALOG_BYTES: u64 = 2 * 1024 * 1024;
const MAX_SKILL_BYTES: u64 = 128 * 1024;
const MAX_RESOURCE_COUNT: usize = 16;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillCatalog {
    pub schema_version: u8,
    pub authoring_date: String,
    pub status: String,
    pub authoring_basis: Vec<SourcePin>,
    pub capability_vocabulary: Vec<String>,
    pub global: Vec<SkillEntry>,
    pub project: Vec<SkillEntry>,
    pub profiles: BTreeMap<String, SkillProfile>,
    pub reference_sources: Vec<serde_json::Value>,
    pub evaluation_contract: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourcePin {
    pub path: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillEntry {
    pub id: String,
    pub description: String,
    pub path: String,
    pub required_capabilities: Vec<String>,
    pub conditional_capabilities: Vec<String>,
    pub source_sha256: String,
    pub source_bytes: u64,
    #[serde(default)]
    pub resources: Vec<SkillResource>,
    pub provenance: serde_json::Value,
    pub evaluation: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillResource {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillProfile {
    pub skills: Vec<String>,
    pub conditional: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct QualifiedCatalog {
    pub root: PathBuf,
    pub catalog: SkillCatalog,
}

impl QualifiedCatalog {
    pub fn open(root: &Path) -> Result<Self, String> {
        if !root.is_absolute() || !root.is_dir() {
            return Err("skill catalog requires an absolute release directory".into());
        }
        let root = root.canonicalize().map_err(str_error)?;
        let skills_dir = root.join("skills");
        let meta = fs::symlink_metadata(&skills_dir).map_err(str_error)?;
        if meta.file_type().is_symlink() || !meta.is_dir() {
            return Err("release skill root must be a real directory".into());
        }
        let bytes = read_regular(&skills_dir.join("catalog.json"), MAX_CATALOG_BYTES)?;
        let catalog: SkillCatalog = serde_json::from_slice(&bytes).map_err(str_error)?;
        if catalog.schema_version != 1 {
            return Err("unsupported skill catalog schema".into());
        }
        if catalog.authoring_date.trim().is_empty()
            || catalog.status.trim().is_empty()
            || catalog.status.len() > 256
            || catalog.status.chars().any(char::is_control)
            || catalog.authoring_basis.is_empty()
            || catalog.reference_sources.is_empty()
            || catalog.reference_sources.len() > 128
        {
            return Err("skill catalog provenance/evaluation status is incomplete".into());
        }
        let reference_ids = validate_reference_sources(&catalog.reference_sources)?;
        validate_evaluation_contract(&catalog.evaluation_contract)?;
        let capabilities = catalog
            .capability_vocabulary
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if capabilities.len() != catalog.capability_vocabulary.len()
            || capabilities
                .iter()
                .any(|capability| capability.is_empty() || capability.len() > 64)
        {
            return Err("skill catalog capability vocabulary is invalid".into());
        }
        let mut all = BTreeSet::new();
        let mut project = BTreeSet::new();
        for (scope, entries) in [("global", &catalog.global), ("project", &catalog.project)] {
            for entry in entries {
                validate_id(&entry.id)?;
                if !all.insert(entry.id.as_str()) {
                    return Err(format!("duplicate skill ID: {}", entry.id));
                }
                if scope == "project" {
                    project.insert(entry.id.as_str());
                }
                if entry.path != format!("skills/{}/SKILL.md", entry.id)
                    || entry.description.trim().is_empty()
                    || entry.description.len() > 600
                    || entry.source_bytes > MAX_SKILL_BYTES
                    || !valid_sha256(&entry.source_sha256)
                {
                    return Err(format!("invalid catalog entry: {}", entry.id));
                }
                if scope == "global" && !entry.required_capabilities.is_empty() {
                    return Err(format!(
                        "global skill requires a private capability: {}",
                        entry.id
                    ));
                }
                let declared_capabilities = entry
                    .required_capabilities
                    .iter()
                    .chain(&entry.conditional_capabilities)
                    .collect::<Vec<_>>();
                if declared_capabilities
                    .iter()
                    .any(|capability| !capabilities.contains(capability.as_str()))
                    || declared_capabilities.iter().collect::<BTreeSet<_>>().len()
                        != declared_capabilities.len()
                {
                    return Err(format!(
                        "skill has an unknown or repeated capability: {}",
                        entry.id
                    ));
                }
                if !entry.provenance.is_object() || !entry.evaluation.is_object() {
                    return Err(format!(
                        "skill has no provenance/evaluation record: {}",
                        entry.id
                    ));
                }
                validate_provenance(&entry.id, &entry.provenance, &reference_ids)?;
                let evaluation = object_with_keys(
                    &entry.evaluation,
                    &["cases", "status"],
                    &format!("skill evaluation: {}", entry.id),
                )?;
                let expected_record = format!("skills/PROVENANCE.md#{}", entry.id);
                if entry
                    .provenance
                    .get("record")
                    .and_then(serde_json::Value::as_str)
                    != Some(expected_record.as_str())
                    || evaluation.get("status").and_then(serde_json::Value::as_str)
                        != Some("planned-not-run")
                {
                    return Err(format!(
                        "skill provenance/evaluation does not match: {}",
                        entry.id
                    ));
                }
                let cases = evaluation
                    .get("cases")
                    .and_then(serde_json::Value::as_array)
                    .ok_or_else(|| format!("skill has no planned cases: {}", entry.id))?;
                let mut kinds = BTreeMap::<&str, usize>::new();
                let mut case_ids = BTreeSet::new();
                for case in cases {
                    let case = object_with_keys(
                        case,
                        &["expected", "id", "kind", "prompt"],
                        &format!("skill evaluation case: {}", entry.id),
                    )?;
                    let kind = case
                        .get("kind")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("");
                    let id = case
                        .get("id")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("");
                    let prompt = case
                        .get("prompt")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("");
                    let expected = case
                        .get("expected")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("");
                    if !id.starts_with(&format!("{}-", entry.id))
                        || !case_ids.insert(id)
                        || prompt.trim().is_empty()
                        || expected.trim().is_empty()
                    {
                        return Err(format!("skill has an invalid planned case: {}", entry.id));
                    }
                    *kinds.entry(kind).or_default() += 1;
                }
                if kinds != BTreeMap::from([("positive", 2), ("negative", 2), ("mixed", 1)]) {
                    return Err(format!("skill has an incomplete case mix: {}", entry.id));
                }
                let path = root.join(&entry.path);
                let parent = path.parent().ok_or("skill path has no parent")?;
                let parent_meta = fs::symlink_metadata(parent).map_err(str_error)?;
                if parent_meta.file_type().is_symlink() || !parent_meta.is_dir() {
                    return Err(format!("skill directory is not real: {}", entry.id));
                }
                validate_skill_resources(&root, entry)?;
                let body = read_regular(&path, MAX_SKILL_BYTES)?;
                validate_frontmatter(&body, &entry.id, &entry.description)?;
                if body.len() as u64 != entry.source_bytes || sha256(&body) != entry.source_sha256 {
                    return Err(format!(
                        "skill source digest differs from catalog: {}",
                        entry.id
                    ));
                }
            }
        }
        for (name, profile) in &catalog.profiles {
            validate_id(name)?;
            let mut selected = BTreeSet::new();
            for id in profile.skills.iter().chain(profile.conditional.iter()) {
                if !project.contains(id.as_str()) || !selected.insert(id) {
                    return Err(format!(
                        "profile {name} references an unknown or duplicate skill: {id}"
                    ));
                }
            }
            if profile.skills.is_empty() {
                return Err(format!("profile {name} has no baseline skills"));
            }
        }
        Ok(Self { root, catalog })
    }

    pub fn release_shape(&self) -> Result<(), String> {
        if self.catalog.global.is_empty()
            || self.catalog.global.len() + self.catalog.project.len() > 256
            || self.catalog.profiles.len() > 64
        {
            return Err(
                "release skill catalog needs core guidance and must remain within 256 skills and 64 profiles".into(),
            );
        }
        Ok(())
    }

    pub fn entry(&self, id: &str) -> Option<&SkillEntry> {
        self.catalog
            .global
            .iter()
            .chain(self.catalog.project.iter())
            .find(|entry| entry.id == id)
    }
}

fn object_with_keys<'a>(
    value: &'a serde_json::Value,
    expected: &[&str],
    label: &str,
) -> Result<&'a serde_json::Map<String, serde_json::Value>, String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("{label} must be an object"))?;
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(format!("{label} has an unexpected field set"));
    }
    Ok(object)
}

fn required_text<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
    label: &str,
) -> Result<&'a str, String> {
    let value = object
        .get(key)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("{label}.{key} must be text"))?;
    if value.trim().is_empty() || value.len() > 4096 {
        return Err(format!("{label}.{key} must be bounded and nonempty"));
    }
    Ok(value)
}

fn validate_reference_sources(sources: &[serde_json::Value]) -> Result<BTreeSet<String>, String> {
    let mut ids = BTreeSet::new();
    for source in sources {
        let object = object_with_keys(
            source,
            &[
                "classification",
                "id",
                "inspection_status",
                "rights_status",
                "selection_locator",
                "urls",
            ],
            "reference source",
        )?;
        let id = required_text(object, "id", "reference source")?;
        if id.len() > 64
            || !id.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'.'
            })
            || !ids.insert(id.to_owned())
        {
            return Err(format!(
                "reference source has an invalid or duplicate ID: {id}"
            ));
        }
        let classification = required_text(object, "classification", "reference source")?;
        if !matches!(
            classification,
            "adaptable" | "comparison-only" | "link-only" | "rejected" | "method-adapted"
        ) {
            return Err(format!(
                "reference source has an invalid classification: {id}"
            ));
        }
        for key in ["inspection_status", "rights_status", "selection_locator"] {
            required_text(object, key, "reference source")?;
        }
        let urls = object
            .get("urls")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| format!("reference source.urls must be a list: {id}"))?;
        if urls.is_empty()
            || urls.iter().any(|url| {
                url.as_str()
                    .is_none_or(|url| url.len() > 2048 || !url.starts_with("https://"))
            })
        {
            return Err(format!("reference source.urls is invalid: {id}"));
        }
    }
    Ok(ids)
}

fn validate_provenance(
    id: &str,
    value: &serde_json::Value,
    reference_ids: &BTreeSet<String>,
) -> Result<(), String> {
    let object = object_with_keys(
        value,
        &[
            "classification",
            "local_basis",
            "record",
            "reference_families",
            "reuse_terms",
        ],
        &format!("skill provenance: {id}"),
    )?;
    if required_text(object, "classification", &format!("skill provenance: {id}"))?
        != "independently-authored"
        && required_text(object, "classification", &format!("skill provenance: {id}"))?
            != "method-adapted"
    {
        return Err(format!(
            "skill provenance has an invalid classification: {id}"
        ));
    }
    let local_basis = object
        .get("local_basis")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("skill provenance.local_basis must be a list: {id}"))?;
    if local_basis.is_empty()
        || local_basis.iter().any(|item| {
            item.as_str()
                .is_none_or(|item| item.trim().is_empty() || item.len() > 1024)
        })
    {
        return Err(format!("skill provenance.local_basis is invalid: {id}"));
    }
    let families = object
        .get("reference_families")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("skill provenance.reference_families must be a list: {id}"))?;
    let mut seen = BTreeSet::new();
    if families.is_empty()
        || families.iter().any(|family| {
            let Some(family) = family.as_str() else {
                return true;
            };
            family.trim().is_empty() || !seen.insert(family) || !reference_ids.contains(family)
        })
    {
        return Err(format!(
            "skill provenance.reference_families is unmapped or duplicated: {id}"
        ));
    }
    required_text(object, "reuse_terms", &format!("skill provenance: {id}"))?;
    Ok(())
}

fn validate_evaluation_contract(value: &serde_json::Value) -> Result<(), String> {
    let object = object_with_keys(
        value,
        &[
            "baseline",
            "cases_per_skill",
            "metrics",
            "output_check",
            "platforms",
            "qualification",
            "status",
        ],
        "catalog evaluation contract",
    )?;
    if required_text(object, "status", "catalog evaluation contract")? != "planned-not-run" {
        return Err("catalog evaluation contract has an invalid status".into());
    }
    for key in ["baseline", "output_check", "qualification"] {
        required_text(object, key, "catalog evaluation contract")?;
    }
    for key in ["metrics", "platforms"] {
        let values = object
            .get(key)
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| format!("catalog evaluation contract.{key} must be a list"))?;
        if values.is_empty()
            || values.iter().any(|value| {
                value
                    .as_str()
                    .is_none_or(|value| value.trim().is_empty() || value.len() > 512)
            })
        {
            return Err(format!("catalog evaluation contract.{key} is invalid"));
        }
    }
    let cases = object_with_keys(
        object
            .get("cases_per_skill")
            .ok_or("catalog evaluation contract.cases_per_skill is missing")?,
        &["mixed", "negative", "positive"],
        "catalog evaluation contract.cases_per_skill",
    )?;
    for (key, expected) in [("positive", 2), ("negative", 2), ("mixed", 1)] {
        if cases.get(key).and_then(serde_json::Value::as_u64) != Some(expected) {
            return Err(format!(
                "catalog evaluation contract.cases_per_skill.{key} is invalid"
            ));
        }
    }
    Ok(())
}

fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.len() > 64
        || id.starts_with("mirket-")
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        || id.starts_with('-')
        || id.ends_with('-')
        || id.contains("--")
    {
        return Err(format!("invalid or branded skill ID: {id}"));
    }
    Ok(())
}

fn validate_frontmatter(bytes: &[u8], id: &str, description: &str) -> Result<(), String> {
    let text = std::str::from_utf8(bytes).map_err(str_error)?;
    let mut lines = text.lines();
    if lines.next() != Some("---") {
        return Err(format!("skill {id} lacks frontmatter"));
    }
    let mut name = None;
    let mut actual_description = None;
    let mut closed = false;
    for line in lines {
        if line == "---" {
            closed = true;
            break;
        }
        if let Some(value) = line.strip_prefix("name:") {
            name = Some(value.trim().trim_matches(['\'', '"']));
        }
        if let Some(value) = line.strip_prefix("description:") {
            actual_description = Some(value.trim().trim_matches(['\'', '"']));
        }
    }
    if !closed || name != Some(id) || actual_description != Some(description) {
        return Err(format!("skill frontmatter differs from catalog: {id}"));
    }
    Ok(())
}

fn read_regular(path: &Path, max: u64) -> Result<Vec<u8>, String> {
    let meta = fs::symlink_metadata(path).map_err(str_error)?;
    if meta.file_type().is_symlink() || !meta.is_file() || meta.len() > max {
        return Err(format!(
            "catalog file is not a bounded regular file: {}",
            path.display()
        ));
    }
    let bytes = fs::read(path).map_err(str_error)?;
    if bytes.len() as u64 > max {
        return Err("catalog file changed or exceeded size limit".into());
    }
    Ok(bytes)
}

fn validate_skill_resources(root: &Path, entry: &SkillEntry) -> Result<(), String> {
    if entry.resources.len() > MAX_RESOURCE_COUNT {
        return Err(format!("skill has too many resources: {}", entry.id));
    }
    let skill_dir = root.join("skills").join(&entry.id);
    let skill_meta = fs::symlink_metadata(&skill_dir).map_err(str_error)?;
    if skill_meta.file_type().is_symlink() || !skill_meta.is_dir() {
        return Err(format!("skill directory is not real: {}", entry.id));
    }
    let mut expected_files = BTreeSet::from(["SKILL.md".to_owned()]);
    let mut expected_dirs = BTreeSet::new();
    for resource in &entry.resources {
        validate_resource_path(&resource.path)?;
        if !expected_files.insert(resource.path.clone()) {
            return Err(format!("duplicate skill resource: {}", entry.id));
        }
        let mut prefix = String::new();
        let components = resource.path.split('/').collect::<Vec<_>>();
        for component in &components[..components.len() - 1] {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(component);
            expected_dirs.insert(prefix.clone());
        }
    }
    let mut actual_files = BTreeSet::new();
    let mut actual_dirs = BTreeSet::new();
    collect_skill_paths(&skill_dir, &skill_dir, &mut actual_files, &mut actual_dirs)?;
    if actual_files != expected_files || actual_dirs != expected_dirs {
        return Err(format!("skill resources differ from catalog: {}", entry.id));
    }
    for resource in &entry.resources {
        let path = skill_dir.join(&resource.path);
        let bytes = read_regular(&path, MAX_SKILL_BYTES)?;
        if bytes.len() as u64 != resource.bytes || sha256(&bytes) != resource.sha256 {
            return Err(format!(
                "skill resource digest differs from catalog: {}",
                entry.id
            ));
        }
    }
    Ok(())
}

fn collect_skill_paths(
    root: &Path,
    path: &Path,
    files: &mut BTreeSet<String>,
    directories: &mut BTreeSet<String>,
) -> Result<(), String> {
    for item in fs::read_dir(path).map_err(str_error)? {
        let item = item.map_err(str_error)?;
        let candidate = item.path();
        let meta = fs::symlink_metadata(&candidate).map_err(str_error)?;
        if meta.file_type().is_symlink() {
            return Err(format!(
                "skill resource is a symlink: {}",
                candidate.display()
            ));
        }
        let relative = candidate
            .strip_prefix(root)
            .map_err(str_error)?
            .to_str()
            .ok_or("skill resource path is not UTF-8")?
            .replace(std::path::MAIN_SEPARATOR, "/");
        if meta.is_dir() {
            validate_resource_directory(&relative)?;
            directories.insert(relative);
            collect_skill_paths(root, &candidate, files, directories)?;
        } else if meta.is_file() {
            if relative != "SKILL.md" {
                validate_resource_path(&relative)?;
            }
            files.insert(relative);
        } else {
            return Err(format!(
                "skill resource is not regular: {}",
                candidate.display()
            ));
        }
    }
    Ok(())
}

fn validate_resource_directory(path: &str) -> Result<(), String> {
    if path.is_empty()
        || path.len() > 512
        || (path != "references" && !path.starts_with("references/"))
        || path.contains('\\')
        || path.split('/').any(|part| {
            part.is_empty()
                || part == "."
                || part == ".."
                || part.contains(':')
                || part.chars().any(char::is_control)
                || !valid_resource_component(part)
        })
    {
        return Err(format!("invalid skill resource directory: {path}"));
    }
    Ok(())
}

fn validate_resource_path(path: &str) -> Result<(), String> {
    if path.is_empty()
        || path.len() > 512
        || !path.starts_with("references/")
        || !path.ends_with(".md")
        || path.contains('\\')
        || path.split('/').any(|part| {
            part.is_empty()
                || part == "."
                || part == ".."
                || part.contains(':')
                || part.chars().any(char::is_control)
                || (part != "references" && !valid_resource_component(part))
        })
    {
        return Err(format!("invalid skill resource path: {path}"));
    }
    Ok(())
}

fn valid_resource_component(component: &str) -> bool {
    component.split('.').all(|part| {
        let bytes = part.as_bytes();
        !bytes.is_empty()
            && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
            && bytes.iter().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-' || *byte == b'_'
            })
            && bytes
                .last()
                .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    })
}

fn str_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
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
                "mirket-skill-catalog-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(root.join("skills/task-planning")).unwrap();
            Self(root.canonicalize().unwrap())
        }
        fn write(&self, id: &str, frontmatter_name: &str) {
            let body =
                format!("---\nname: {frontmatter_name}\ndescription: Plan work\n---\n\n# Plan\n");
            let cases = ["positive", "positive", "negative", "negative", "mixed"]
                .iter()
                .enumerate()
                .map(|(index, kind)| {
                    serde_json::json!({
                        "id": format!("{id}-{kind}-{index}"),
                        "kind": kind,
                        "prompt": "Do the task",
                        "expected": "Provide evidence"
                    })
                })
                .collect::<Vec<_>>();
            let references = (0..58)
                .map(|index| {
                    serde_json::json!({
                        "id": format!("fixture-reference-{index}"),
                        "urls": [format!("https://example.com/{index}")],
                        "selection_locator": "fixture source",
                        "classification": "comparison-only",
                        "inspection_status": "fixture source is not executed",
                        "rights_status": "fixture source is not redistributed"
                    })
                })
                .collect::<Vec<_>>();
            fs::write(self.0.join("skills/task-planning/SKILL.md"), &body).unwrap();
            fs::write(self.0.join("skills/catalog.json"), serde_json::json!({
                "schema_version": 1,
                "authoring_date": "2026-10-25",
                "status": "authored; sampled evaluation in progress",
                "authoring_basis": [{"path":"docs/skills.md","sha256":"0".repeat(64)}],
                "capability_vocabulary": [],
                "global": [{"id":id,"description":"Plan work","path":"skills/task-planning/SKILL.md","required_capabilities":[],"conditional_capabilities":[],"source_sha256":sha256(body.as_bytes()),"source_bytes":body.len(),"provenance":{"classification":"independently-authored","record":format!("skills/PROVENANCE.md#{id}"),"local_basis":["fixture basis"],"reference_families":["fixture-reference-0"],"reuse_terms":"Independently authored fixture prose; source reference retained"},"evaluation":{"status":"planned-not-run","cases":cases}}],
                "project": [],
                "profiles": {},
                "reference_sources": references,
                "evaluation_contract": {"status":"planned-not-run","cases_per_skill":{"positive":2,"negative":2,"mixed":1},"baseline":"fixture baseline","output_check":"fixture output check","metrics":["task outcome"],"platforms":["fixture platform"],"qualification":"fixture qualification"}
            }).to_string()).unwrap();
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn validates_exact_id_and_frontmatter() {
        let fixture = Fixture::new();
        fixture.write("task-planning", "task-planning");
        let catalog = QualifiedCatalog::open(&fixture.0).unwrap();
        assert_eq!(
            catalog.entry("task-planning").unwrap().description,
            "Plan work"
        );
        assert!(catalog.release_shape().is_ok());
        fixture.write("task-planning", "other-name");
        assert!(QualifiedCatalog::open(&fixture.0).is_err());
    }

    #[test]
    fn rejects_unmapped_reference_families_and_empty_reuse_terms() {
        let fixture = Fixture::new();
        fixture.write("task-planning", "task-planning");
        let path = fixture.0.join("skills/catalog.json");
        let mut catalog: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        catalog["global"][0]["provenance"]["reference_families"][0] =
            serde_json::Value::String("unknown-reference".into());
        fs::write(&path, serde_json::to_vec(&catalog).unwrap()).unwrap();
        assert!(QualifiedCatalog::open(&fixture.0).is_err());

        fixture.write("task-planning", "task-planning");
        let mut catalog: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        catalog["global"][0]["provenance"]["reuse_terms"] = serde_json::Value::String("   ".into());
        fs::write(&path, serde_json::to_vec(&catalog).unwrap()).unwrap();
        assert!(QualifiedCatalog::open(&fixture.0).is_err());
    }

    #[test]
    fn rejects_extra_evaluation_fields() {
        let fixture = Fixture::new();
        fixture.write("task-planning", "task-planning");
        let path = fixture.0.join("skills/catalog.json");
        let mut catalog: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        catalog["global"][0]["evaluation"]["unreviewed_claim"] = serde_json::Value::Bool(true);
        fs::write(&path, serde_json::to_vec(&catalog).unwrap()).unwrap();
        assert!(QualifiedCatalog::open(&fixture.0).is_err());
    }

    #[test]
    fn status_can_report_new_evidence_without_changing_catalog_format() {
        let fixture = Fixture::new();
        fixture.write("task-planning", "task-planning");
        let path = fixture.0.join("skills/catalog.json");
        let mut catalog: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        catalog["status"] = serde_json::Value::String("authored; new trial recorded".into());
        fs::write(&path, serde_json::to_vec(&catalog).unwrap()).unwrap();
        assert!(QualifiedCatalog::open(&fixture.0).is_ok());
        catalog["status"] = serde_json::Value::String("\n".into());
        fs::write(&path, serde_json::to_vec(&catalog).unwrap()).unwrap();
        assert!(QualifiedCatalog::open(&fixture.0).is_err());
    }

    #[test]
    fn authored_release_catalog_is_bounded_and_consistent() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        QualifiedCatalog::open(root)
            .unwrap()
            .release_shape()
            .unwrap();
    }

    #[test]
    fn refuses_branded_and_escaping_ids() {
        let fixture = Fixture::new();
        fixture.write("mirket-task-planning", "mirket-task-planning");
        assert!(QualifiedCatalog::open(&fixture.0).is_err());
        fixture.write("../outside", "../outside");
        assert!(QualifiedCatalog::open(&fixture.0).is_err());
    }

    #[test]
    fn resource_paths_allow_versioned_markdown_and_reject_escapes() {
        assert!(validate_resource_path("references/hyperframes-0.8.197-authoring.md").is_ok());
        for path in [
            "references/../outside.md",
            "references\\outside.md",
            "references/UPPER.md",
            "references/foo.md/escape",
        ] {
            assert!(
                validate_resource_path(path).is_err(),
                "accepted unsafe path: {path}"
            );
        }
        assert!(validate_resource_directory("references/0.8").is_ok());
        assert!(validate_resource_directory("references\\alias").is_err());
        assert!(validate_resource_directory("references-other").is_err());
    }
}
