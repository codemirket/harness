use anyhow::{Context, Result, bail, ensure};
use include_dir::{Dir, include_dir};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    io::{Cursor, Read},
    path::Path,
    sync::{Arc, OnceLock},
    time::Duration,
};

use crate::util::{
    atomic_write, ensure_directory, read_bounded, reject_symlink_components, sha256,
};

static SKILLS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/skills");
const CATALOG: &str = include_str!("../registry/catalog.json");
const CAPABILITIES: &str = include_str!("../registry/capabilities.json");
const HARNESS: &str = include_str!("../registry/harness.json");
const NOTICES: &[u8] = include_bytes!("../docs/third-party-notices.md");
const INSTRUCTIONS: &[u8] = include_bytes!("../instructions/AGENTS.md");
pub const MAX_FILE: usize = 8 * 1024 * 1024;
pub const MAX_PAYLOAD: usize = 32 * 1024 * 1024;
const MAX_ARCHIVE: usize = 100 * 1024 * 1024;
const MAX_EXPANDED: u64 = 512 * 1024 * 1024;
const MAX_FILES: usize = 4096;
const MAX_ARCHIVE_FILES: usize = 100_000;
const CACHE_MANIFEST: &str = ".mirket-payload.json";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub scope: String,
    pub delivery: String,
    pub path: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub agents: Vec<String>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub conflicts: Vec<String>,
    #[serde(default)]
    pub license_files: Vec<String>,
    #[serde(default)]
    pub extra_files: BTreeMap<String, String>,
    #[serde(default)]
    pub executable_files: BTreeSet<String>,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub installed_sha256: Option<String>,
    #[serde(default)]
    pub adaptation: Option<String>,
    #[serde(default)]
    pub replacements: Vec<Replacement>,
    #[serde(flatten)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Replacement {
    path: String,
    old: String,
    new: String,
    count: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Source {
    pub repository: String,
    pub commit: String,
    #[serde(default)]
    pub fetch_mode: Option<String>,
    #[serde(flatten)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Profile {
    pub description: String,
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Capability {
    pub id: String,
    pub title: String,
    pub lead: String,
    pub support: Vec<String>,
    pub aliases: Vec<String>,
    pub deliverable: String,
    pub acceptance: Vec<String>,
    pub failure_probe: String,
    pub prerequisites: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkillDocument {
    pub id: String,
    pub path: String,
    pub sha256: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct Payload {
    pub files: BTreeMap<String, Vec<u8>>,
    pub executables: BTreeSet<String>,
    pub sha256: String,
}

impl Payload {
    fn new(files: BTreeMap<String, Vec<u8>>, executables: BTreeSet<String>) -> Result<Self> {
        validate_files(&files)?;
        for executable in &executables {
            ensure!(
                files.contains_key(executable),
                "missing executable payload: {executable}"
            );
        }
        let sha256 = payload_hash(&files);
        Ok(Self {
            files,
            executables,
            sha256,
        })
    }
}

#[derive(Deserialize)]
struct CatalogData {
    schema_version: u32,
    skills: Vec<Skill>,
    sources: BTreeMap<String, Source>,
    profiles: BTreeMap<String, Profile>,
}

struct CatalogInner {
    data: CatalogData,
    by_id: HashMap<String, usize>,
    search: Vec<String>,
    globals: Vec<String>,
    capabilities: Vec<Capability>,
    capability_names: HashMap<String, usize>,
    payloads: HashMap<String, Payload>,
}

#[derive(Clone)]
pub struct Catalog(Arc<CatalogInner>);

impl Catalog {
    pub fn embedded() -> Result<Self> {
        static INSTANCE: OnceLock<std::result::Result<Arc<CatalogInner>, String>> = OnceLock::new();
        match INSTANCE.get_or_init(|| Self::load().map(Arc::new).map_err(|e| format!("{e:#}"))) {
            Ok(inner) => Ok(Self(inner.clone())),
            Err(message) => bail!("embedded catalog is invalid: {message}"),
        }
    }

    fn load() -> Result<CatalogInner> {
        let data: CatalogData = serde_json::from_str(CATALOG)?;
        ensure!(data.schema_version == 1, "unsupported catalog schema");
        let harness: Value = serde_json::from_str(HARNESS)?;
        let globals: Vec<String> = serde_json::from_value(harness["global_skills"].clone())?;
        let contracts: Value = serde_json::from_str(CAPABILITIES)?;
        ensure!(
            contracts["schema_version"] == 1,
            "unsupported capability schema"
        );
        let capabilities: Vec<Capability> =
            serde_json::from_value(contracts["capabilities"].clone())?;
        let mut by_id = HashMap::new();
        let mut payloads = HashMap::new();
        let mut search = Vec::new();
        for (i, skill) in data.skills.iter().enumerate() {
            validate_id(&skill.id)?;
            ensure!(
                by_id.insert(skill.id.clone(), i).is_none(),
                "duplicate skill {}",
                skill.id
            );
            search.push(
                format!(
                    "{} {} {} {} {}",
                    skill.id,
                    skill.name,
                    skill.description,
                    skill.category,
                    skill.tags.join(" ")
                )
                .to_lowercase(),
            );
            if skill.delivery == "embedded" {
                validate_id(&skill.name)?;
                let prefix = skill
                    .path
                    .strip_prefix("skills/")
                    .context("embedded skill must be in skills")?;
                safe_path(prefix)?;
                let dir = SKILLS
                    .get_dir(prefix)
                    .with_context(|| format!("missing embedded skill {}", skill.id))?;
                let mut files = BTreeMap::new();
                collect_embedded(dir, prefix, &mut files)?;
                // The notices accompany authored guidance and remain available offline.
                files.insert(".upstream-licenses/NOTICE.md".into(), NOTICES.to_vec());
                ensure!(
                    skill_name(files.get("SKILL.md").context("missing SKILL.md")?)? == skill.name,
                    "skill invocation differs from catalog: {}",
                    skill.id
                );
                payloads.insert(
                    skill.id.clone(),
                    Payload::new(files, skill.executable_files.clone())?,
                );
            } else if skill.delivery == "upstream" {
                validate_id(&skill.name)?;
                safe_path(&skill.path)?;
                ensure!(
                    skill.sha256.as_deref().is_some_and(valid_digest),
                    "missing source digest: {}",
                    skill.id
                );
                ensure!(
                    valid_digest(
                        skill
                            .installed_sha256
                            .as_deref()
                            .unwrap_or(skill.sha256.as_deref().unwrap())
                    ),
                    "invalid installed digest"
                );
                let source = data
                    .sources
                    .get(skill.source.as_ref().context("missing skill source")?)
                    .context("unknown skill source")?;
                validate_source(source)?;
                for path in skill
                    .license_files
                    .iter()
                    .chain(skill.extra_files.keys())
                    .chain(skill.extra_files.values())
                {
                    safe_path(path)?;
                }
            }
        }
        for id in &globals {
            let skill = by_id
                .get(id)
                .map(|&i| &data.skills[i])
                .context("missing global skill")?;
            ensure!(
                skill.scope == "global" && payloads.contains_key(id),
                "global skill must be embedded: {id}"
            );
        }
        let mut capability_names = HashMap::new();
        for (i, capability) in capabilities.iter().enumerate() {
            validate_id(&capability.id)?;
            for id in std::iter::once(&capability.lead).chain(&capability.support) {
                ensure!(
                    payloads.contains_key(id),
                    "capability requires an embedded skill: {id}"
                );
            }
            ensure!(
                !capability.acceptance.is_empty(),
                "capability acceptance is empty"
            );
            for name in std::iter::once(&capability.id)
                .chain(std::iter::once(&capability.title))
                .chain(&capability.aliases)
            {
                let key = normalize(name);
                if let Some(previous) = capability_names.insert(key, i) {
                    ensure!(previous == i, "ambiguous capability alias: {name}");
                }
            }
        }
        Ok(CatalogInner {
            data,
            by_id,
            search,
            globals,
            capabilities,
            capability_names,
            payloads,
        })
    }

    pub fn skills(&self) -> &[Skill] {
        &self.0.data.skills
    }
    pub fn global_skills(&self) -> &[String] {
        &self.0.globals
    }
    pub fn profiles(&self) -> &BTreeMap<String, Profile> {
        &self.0.data.profiles
    }
    pub fn sources(&self) -> &BTreeMap<String, Source> {
        &self.0.data.sources
    }
    pub fn capabilities(&self) -> &[Capability] {
        &self.0.capabilities
    }
    pub fn instructions(&self) -> &'static [u8] {
        INSTRUCTIONS
    }

    pub fn skill(&self, id: &str) -> Result<&Skill> {
        self.0
            .by_id
            .get(id)
            .map(|&i| &self.0.data.skills[i])
            .with_context(|| format!("unknown skill: {id}"))
    }

    pub fn search(&self, query: &str, offset: usize, limit: usize) -> Result<Value> {
        ensure!(query.len() <= 1024, "search exceeds 1024 bytes");
        ensure!((1..=100).contains(&limit), "limit must be 1..=100");
        let terms: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
        let matches: Vec<&Skill> = self
            .skills()
            .iter()
            .enumerate()
            .filter_map(|(i, skill)| {
                terms
                    .iter()
                    .all(|term| self.0.search[i].contains(term))
                    .then_some(skill)
            })
            .collect();
        ensure!(offset <= matches.len(), "offset exceeds result count");
        let page: Vec<Value> = matches
            .iter()
            .skip(offset)
            .take(limit)
            .map(|skill| {
                json!({
                    "id": skill.id, "name": skill.name, "description": skill.description,
                    "scope": skill.scope, "delivery": skill.delivery, "category": skill.category,
                    "installable": skill.delivery == "embedded" || skill.delivery == "upstream"
                })
            })
            .collect();
        Ok(json!({"items":page,"total":matches.len(),"offset":offset,
            "next_offset": (offset + limit < matches.len()).then_some(offset + limit)}))
    }

    pub fn payload(&self, id: &str) -> Result<Payload> {
        self.skill(id)?;
        self.0.payloads.get(id).cloned().with_context(|| format!(
            "{id} is not embedded; use mirket catalog fetch or mirket project sync for explicitly selected upstream skills"))
    }

    pub fn payload_digest(&self, id: &str) -> Result<&str> {
        self.0
            .payloads
            .get(id)
            .map(|payload| payload.sha256.as_str())
            .with_context(|| format!("skill is not embedded: {id}"))
    }

    pub fn read(&self, id: &str, path: &str, max_bytes: usize) -> Result<SkillDocument> {
        safe_path(path)?;
        ensure!(
            (1..=MAX_FILE).contains(&max_bytes),
            "document limit must be 1..={MAX_FILE} bytes"
        );
        let payload = self
            .0
            .payloads
            .get(id)
            .with_context(|| format!("skill is not embedded: {id}"))?;
        let bytes = payload
            .files
            .get(path)
            .with_context(|| format!("skill resource not found: {id}/{path}"))?;
        ensure!(
            bytes.len() <= max_bytes,
            "resource is {} bytes, exceeding the {max_bytes} byte limit",
            bytes.len()
        );
        let content = std::str::from_utf8(bytes)
            .context("skill resource is not UTF-8 text")?
            .to_owned();
        Ok(SkillDocument {
            id: id.into(),
            path: path.into(),
            sha256: sha256(bytes),
            content,
        })
    }

    pub fn capability(&self, name: &str) -> Result<&Capability> {
        self.0
            .capability_names
            .get(&normalize(name))
            .map(|&i| &self.0.capabilities[i])
            .with_context(|| format!("unknown capability: {name}; use mirket capabilities list"))
    }

    pub fn capability_plan(&self, name: &str, supporting: &[String]) -> Result<Value> {
        ensure!(
            supporting.len() <= 12,
            "at most 12 supporting capabilities can be composed"
        );
        let primary = self.capability(name)?;
        let mut contracts = vec![primary];
        for name in supporting {
            let row = self.capability(name)?;
            if !contracts.iter().any(|existing| existing.id == row.id) {
                contracts.push(row);
            }
        }
        let mut seen = BTreeSet::new();
        let mut reads = Vec::new();
        let mut project_skills = Vec::new();
        for row in &contracts {
            if !seen.insert(&row.lead) {
                continue;
            }
            let skill = self.skill(&row.lead)?;
            let payload = self
                .0
                .payloads
                .get(&row.lead)
                .context("capability lead is not embedded")?;
            let body = payload
                .files
                .get("SKILL.md")
                .context("missing skill body")?;
            let global = self.global_skills().contains(&row.lead);
            reads.push(json!({"id":row.lead,"name":skill.name,"path":"SKILL.md",
                "sha256":sha256(body),"payload_sha256":payload.sha256,"bytes":body.len(),
                "delivery":if global {"global"} else {"project"}}));
            if !global {
                project_skills.push(row.lead.clone());
            }
        }
        Ok(
            json!({"primary":primary.id,"contracts":contracts,"read_sequence":reads,
            "project_skills":project_skills,"runtime_status":"not_checked",
            "activation_status":"not_observed",
            "authority":"Expertise does not grant execution permissions or professional authority.",
            "next_steps":["Read selected lead guidance and relevant references.",
                "Register missing project leads with mirket project add, then sync and doctor.",
                "Inspect available tools and actual task inputs.",
                "Deliver the task and record observable acceptance and failure-probe evidence."]}),
        )
    }

    pub fn selection(
        &self,
        profiles: &[String],
        ids: &[String],
        target: &str,
    ) -> Result<Vec<&Skill>> {
        ensure!(
            matches!(target, "codex" | "claude"),
            "unknown target: {target}"
        );
        let mut selected = Vec::new();
        let mut visiting = BTreeSet::new();
        let mut seen = BTreeSet::new();
        for profile in profiles {
            let profile = self
                .profiles()
                .get(profile)
                .with_context(|| format!("unknown profile: {profile}"))?;
            for id in &profile.skills {
                self.visit(id, target, &mut visiting, &mut seen, &mut selected)?;
            }
        }
        for id in ids {
            self.visit(id, target, &mut visiting, &mut seen, &mut selected)?;
        }
        let mut names = BTreeMap::new();
        for skill in &selected {
            if let Some(other) = names.insert(&skill.name, &skill.id) {
                bail!(
                    "competing skill name {}: {other} / {}",
                    skill.name,
                    skill.id
                );
            }
            for conflict in &skill.conflicts {
                ensure!(
                    !seen.contains(conflict),
                    "conflicting skills: {} / {conflict}",
                    skill.id
                );
            }
        }
        Ok(selected)
    }

    fn visit<'a>(
        &'a self,
        id: &str,
        target: &str,
        visiting: &mut BTreeSet<String>,
        seen: &mut BTreeSet<String>,
        selected: &mut Vec<&'a Skill>,
    ) -> Result<()> {
        let skill = self.skill(id)?;
        if self.global_skills().iter().any(|global| global == id) {
            return Ok(());
        }
        ensure!(!visiting.contains(id), "cyclic skill requirement: {id}");
        if seen.contains(id) {
            return Ok(());
        }
        ensure!(
            skill.scope == "project" && matches!(skill.delivery.as_str(), "embedded" | "upstream"),
            "skill is not installable: {id}"
        );
        ensure!(
            skill.agents.is_empty() || skill.agents.iter().any(|agent| agent == target),
            "skill {id} does not support {target}"
        );
        visiting.insert(id.into());
        for dependency in &skill.requires {
            self.visit(dependency, target, visiting, seen, selected)?;
        }
        visiting.remove(id);
        seen.insert(id.into());
        selected.push(skill);
        Ok(())
    }

    pub fn check(&self) -> Result<Value> {
        for skill in self
            .skills()
            .iter()
            .filter(|skill| matches!(skill.delivery.as_str(), "embedded" | "upstream"))
        {
            for target in ["codex", "claude"] {
                if skill.agents.is_empty() || skill.agents.iter().any(|agent| agent == target) {
                    self.selection(&[], std::slice::from_ref(&skill.id), target)?;
                }
            }
        }
        for (name, profile) in self.profiles() {
            ensure!(!profile.skills.is_empty(), "empty profile: {name}");
            for id in &profile.skills {
                self.skill(id)?;
            }
        }
        Ok(
            json!({"ok":true,"status":"valid","skills":self.skills().len(),"embedded":self.0.payloads.len(),
            "capabilities":self.capabilities().len(),"profiles":self.profiles().len(),
            "global_skills":self.global_skills().len(),"runtime_status":"not_checked"}),
        )
    }

    pub fn prepare(&self, id: &str, cache: &Path) -> Result<Payload> {
        let skill = self.skill(id)?;
        if skill.delivery == "embedded" {
            return self.payload(id);
        }
        ensure!(
            skill.delivery == "upstream",
            "skill is not installable: {id}"
        );
        let expected = skill
            .installed_sha256
            .as_ref()
            .or(skill.sha256.as_ref())
            .context("missing digest")?;
        ensure!(valid_digest(expected), "invalid payload digest");
        let cache_path = cache.join("skills").join(expected);
        reject_symlink_components(&cache_path)?;
        if cache_path.exists() {
            let payload = read_cached(&cache_path)?;
            ensure!(
                &payload.sha256 == expected && payload.executables == skill.executable_files,
                "cached payload integrity failed: {id}"
            );
            return Ok(payload);
        }
        let source = self
            .sources()
            .get(skill.source.as_ref().context("missing source")?)
            .context("unknown source")?;
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120))
            .connect_timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(concat!("mirket/", env!("CARGO_PKG_VERSION")))
            .build()?;
        let files = if source.fetch_mode.as_deref() == Some("files") {
            fetch_selected_files(&client, source, skill, cache)?
        } else {
            let url = format!(
                "https://codeload.github.com/{}/zip/{}",
                source.repository, source.commit
            );
            let archive = fetch_cached(&client, &url, MAX_ARCHIVE, cache)?;
            select_archive(&archive, skill)?
        };
        let payload = prepare_upstream(files, skill)?;
        write_cache(&cache_path, &payload)?;
        Ok(payload)
    }
}

fn collect_embedded(
    dir: &Dir<'_>,
    prefix: &str,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    for file in dir.files() {
        let path = file
            .path()
            .strip_prefix(prefix)?
            .to_str()
            .context("non-UTF-8 embedded path")?
            .replace('\\', "/");
        files.insert(path, file.contents().to_vec());
    }
    for child in dir.dirs() {
        collect_embedded(child, prefix, files)?;
    }
    Ok(())
}

pub fn validate_id(id: &str) -> Result<()> {
    ensure!(
        !id.is_empty()
            && id.len() <= 100
            && !id.starts_with('-')
            && !id.ends_with('-')
            && !id.contains("--")
            && id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
        "invalid identifier: {id}"
    );
    Ok(())
}

fn normalize(name: &str) -> String {
    name.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

pub fn safe_path(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty() && value.len() <= 1024 && !value.contains('\\') && !value.contains(':'),
        "unsafe payload path: {value}"
    );
    for part in value.split('/') {
        let upper = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        let reserved = matches!(
            upper.as_str(),
            "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
        ) || ["COM", "LPT"].iter().any(|prefix| {
            upper
                .strip_prefix(prefix)
                .is_some_and(|n| n.len() == 1 && matches!(n.as_bytes()[0], b'1'..=b'9'))
        });
        ensure!(
            !part.is_empty()
                && !matches!(part, "." | "..")
                && !part.ends_with(['.', ' '])
                && !part
                    .chars()
                    .any(|c| c.is_control() || "<>\"|?*".contains(c))
                && !reserved,
            "non-portable payload path: {value}"
        );
    }
    Ok(())
}

pub fn payload_hash(files: &BTreeMap<String, Vec<u8>>) -> String {
    let mut digest = Sha256::new();
    for (name, bytes) in files {
        digest.update(name.as_bytes());
        digest.update([0]);
        digest.update(Sha256::digest(bytes));
    }
    format!("{:x}", digest.finalize())
}

fn validate_files(files: &BTreeMap<String, Vec<u8>>) -> Result<()> {
    ensure!(
        files.len() <= MAX_FILES,
        "payload exceeds {MAX_FILES} files"
    );
    let mut total = 0usize;
    let mut portable = BTreeMap::new();
    for (name, bytes) in files {
        safe_path(name)?;
        ensure!(
            name != CACHE_MANIFEST && name != ".mirket-skill.json",
            "payload uses a reserved name"
        );
        ensure!(
            bytes.len() <= MAX_FILE,
            "payload file exceeds {MAX_FILE} bytes: {name}"
        );
        total = total
            .checked_add(bytes.len())
            .context("payload size overflow")?;
        ensure!(total <= MAX_PAYLOAD, "payload exceeds {MAX_PAYLOAD} bytes");
        let parts: Vec<_> = name.split('/').collect();
        for i in 1..=parts.len() {
            let prefix = parts[..i].join("/");
            if i < parts.len() {
                ensure!(
                    !files.contains_key(&prefix),
                    "file/directory collision: {name}"
                );
            }
            if let Some(other) = portable.insert(prefix.to_lowercase(), prefix.clone()) {
                ensure!(other == prefix, "case-colliding payload path: {name}");
            }
        }
    }
    Ok(())
}

fn skill_name(bytes: &[u8]) -> Result<String> {
    let text = std::str::from_utf8(bytes).context("skill body is not UTF-8")?;
    let mut lines = text.lines();
    ensure!(
        lines.next() == Some("---"),
        "skill body requires frontmatter"
    );
    for line in lines.take_while(|line| *line != "---") {
        if let Some(value) = line.strip_prefix("name:") {
            return Ok(value.trim().trim_matches(['\'', '"']).to_owned());
        }
    }
    bail!("skill frontmatter has no name")
}

fn validate_source(source: &Source) -> Result<()> {
    let parts: Vec<_> = source.repository.split('/').collect();
    ensure!(
        parts.len() == 2
            && parts.iter().all(|part| !part.is_empty()
                && *part != "."
                && *part != ".."
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_ .".contains(&b) && b != b' ')),
        "invalid GitHub repository"
    );
    ensure!(
        source.commit.len() == 40 && source.commit.bytes().all(|b| b.is_ascii_hexdigit()),
        "source requires a pinned commit"
    );
    ensure!(
        matches!(
            source.fetch_mode.as_deref(),
            None | Some("files") | Some("archive")
        ),
        "unsupported source fetch mode"
    );
    Ok(())
}

fn member_targets(relative: &str, skill: &Skill) -> Vec<String> {
    let mut targets = Vec::new();
    if let Some(path) = relative.strip_prefix(&format!("{}/", skill.path)) {
        targets.push(path.into());
    }
    if let Some(path) = skill.extra_files.get(relative) {
        targets.push(path.clone());
    }
    if skill.license_files.iter().any(|path| path == relative) {
        targets.push(format!(".upstream-licenses/{relative}"));
    }
    targets
}

fn select_archive(bytes: &[u8], skill: &Skill) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).context("invalid ZIP archive")?;
    ensure!(
        archive.len() <= MAX_ARCHIVE_FILES,
        "archive has too many entries"
    );
    let mut expanded = 0u64;
    let mut selected_bytes = 0usize;
    let mut selected = BTreeMap::new();
    let mut prefix: Option<String> = None;
    let mut seen = BTreeSet::new();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index)?;
        expanded = expanded
            .checked_add(file.size())
            .context("archive size overflow")?;
        ensure!(
            expanded <= MAX_EXPANDED,
            "archive expanded size exceeds limit"
        );
        let raw = file.name().trim_end_matches('/');
        safe_path(raw)?;
        ensure!(
            seen.insert(raw.to_owned()),
            "duplicate archive entry: {raw}"
        );
        let (root, relative) = raw.split_once('/').unwrap_or((raw, ""));
        if let Some(prefix) = &prefix {
            ensure!(root == prefix, "archive contains multiple roots");
        } else {
            prefix = Some(root.into());
        }
        if relative.is_empty() || file.is_dir() {
            continue;
        }
        let targets = member_targets(relative, skill);
        if targets.is_empty() {
            continue;
        }
        if let Some(mode) = file.unix_mode() {
            ensure!(
                mode & 0o170000 == 0 || mode & 0o170000 == 0o100000,
                "selected archive member is not a regular file"
            );
        }
        ensure!(
            file.size() <= MAX_FILE as u64,
            "selected file exceeds limit"
        );
        let mut bytes = Vec::new();
        (&mut file)
            .take(MAX_FILE as u64 + 1)
            .read_to_end(&mut bytes)?;
        ensure!(bytes.len() <= MAX_FILE, "selected file exceeds limit");
        for target in targets {
            selected_bytes = selected_bytes
                .checked_add(bytes.len())
                .context("payload size overflow")?;
            ensure!(
                selected_bytes <= MAX_PAYLOAD && selected.len() < MAX_FILES,
                "payload exceeds size or file limit"
            );
            ensure!(
                selected.insert(target.clone(), bytes.clone()).is_none(),
                "duplicate payload path: {target}"
            );
        }
    }
    validate_files(&selected)?;
    Ok(selected)
}

fn fetch_bytes(client: &reqwest::blocking::Client, url: &str, limit: usize) -> Result<Vec<u8>> {
    let response = client
        .get(url)
        .send()
        .with_context(|| format!("download {url}"))?
        .error_for_status()?;
    ensure!(
        response
            .content_length()
            .is_none_or(|length| length <= limit as u64),
        "download exceeds byte limit"
    );
    let mut bytes = Vec::new();
    response.take(limit as u64 + 1).read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= limit, "download exceeds byte limit");
    Ok(bytes)
}

fn fetch_cached(
    client: &reqwest::blocking::Client,
    url: &str,
    limit: usize,
    cache: &Path,
) -> Result<Vec<u8>> {
    let path = cache.join("sources").join(sha256(url.as_bytes()));
    reject_symlink_components(&path)?;
    if path.exists() {
        return read_bounded(&path, limit as u64);
    }
    let bytes = fetch_bytes(client, url, limit)?;
    // Source caches are untrusted input: selection, bounds, licenses and pinned
    // payload hashes are checked again on every use, before installation.
    atomic_write(&path, &bytes)?;
    Ok(bytes)
}

fn fetch_selected_files(
    client: &reqwest::blocking::Client,
    source: &Source,
    skill: &Skill,
    cache: &Path,
) -> Result<BTreeMap<String, Vec<u8>>> {
    let url = format!(
        "https://api.github.com/repos/{}/git/trees/{}?recursive=1",
        source.repository, source.commit
    );
    let tree: Value =
        serde_json::from_slice(&fetch_cached(client, &url, 16 * 1024 * 1024, cache)?)?;
    ensure!(
        tree["truncated"] == false,
        "GitHub tree is missing completeness evidence"
    );
    let members = tree["tree"].as_array().context("invalid GitHub tree")?;
    ensure!(
        members.len() <= MAX_ARCHIVE_FILES,
        "tree has too many entries"
    );
    let mut selected = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut plan = Vec::new();
    let mut total = 0u64;
    for member in members {
        let path = member["path"].as_str().context("missing tree path")?;
        ensure!(seen.insert(path.to_owned()), "duplicate GitHub tree path");
        let targets = member_targets(path, skill);
        if targets.is_empty() {
            continue;
        }
        safe_path(path)?;
        if member["type"] == "tree" {
            continue;
        }
        ensure!(
            member["type"] == "blob"
                && matches!(member["mode"].as_str(), Some("100644" | "100755")),
            "selected GitHub path is not a regular file: {path}"
        );
        let size = member["size"].as_u64().context("invalid blob size")?;
        ensure!(size <= MAX_FILE as u64, "selected file exceeds limit");
        total = total
            .checked_add(size * targets.len() as u64)
            .context("payload size overflow")?;
        ensure!(total <= MAX_PAYLOAD as u64, "payload exceeds byte limit");
        plan.push((path.to_owned(), size, targets));
        ensure!(plan.len() <= MAX_FILES, "payload exceeds file limit");
    }
    for (path, size, targets) in plan {
        let mut url = reqwest::Url::parse("https://raw.githubusercontent.com/")?;
        url.path_segments_mut()
            .map_err(|_| anyhow::anyhow!("invalid source URL"))?
            .extend(source.repository.split('/'))
            .push(&source.commit)
            .extend(path.split('/'));
        let bytes = fetch_cached(client, url.as_str(), size as usize, cache)?;
        ensure!(
            bytes.len() == size as usize,
            "file size differs from pinned tree"
        );
        for target in targets {
            ensure!(
                selected.insert(target.clone(), bytes.clone()).is_none(),
                "duplicate payload path: {target}"
            );
        }
    }
    validate_files(&selected)?;
    Ok(selected)
}

fn prepare_upstream(mut files: BTreeMap<String, Vec<u8>>, skill: &Skill) -> Result<Payload> {
    validate_files(&files)?;
    for license in &skill.license_files {
        ensure!(
            files.contains_key(&format!(".upstream-licenses/{license}")),
            "missing selected license: {license}"
        );
    }
    for (source, target) in &skill.extra_files {
        ensure!(
            files.contains_key(target),
            "missing selected companion: {source}"
        );
    }
    ensure!(
        payload_hash(&files) == *skill.sha256.as_ref().context("missing source digest")?,
        "source payload differs from pinned reviewed content: {}",
        skill.id
    );
    ensure!(
        skill_name(files.get("SKILL.md").context("missing SKILL.md")?)? == skill.name,
        "skill invocation mismatch"
    );
    for replacement in &skill.replacements {
        safe_path(&replacement.path)?;
        ensure!(
            !replacement.old.is_empty() && replacement.count > 0,
            "invalid replacement"
        );
        let bytes = files
            .get_mut(&replacement.path)
            .context("missing replacement file")?;
        let text = std::str::from_utf8(bytes)?;
        ensure!(
            text.matches(&replacement.old).count() == replacement.count,
            "replacement count differs: {}",
            replacement.path
        );
        *bytes = text
            .replace(&replacement.old, &replacement.new)
            .into_bytes();
    }
    if let Some(note) = &skill.adaptation {
        let bytes = files.get_mut("SKILL.md").context("missing skill body")?;
        let text = std::str::from_utf8(bytes)?;
        let boundary = text
            .get(3..)
            .and_then(|text| text.find("\n---"))
            .context("invalid frontmatter boundary")?
            + 7;
        *bytes = format!(
            "{}\n\n## Personal catalog integration\n\n{}\n{}",
            &text[..boundary],
            note,
            &text[boundary..]
        )
        .into_bytes();
    }
    let payload = Payload::new(files, skill.executable_files.clone())?;
    let expected = skill
        .installed_sha256
        .as_ref()
        .or(skill.sha256.as_ref())
        .context("missing installed digest")?;
    ensure!(
        &payload.sha256 == expected,
        "adapted payload differs from reviewed content: {}",
        skill.id
    );
    ensure!(
        skill_name(
            payload
                .files
                .get("SKILL.md")
                .context("missing skill body")?
        )? == skill.name,
        "adaptation changed skill invocation"
    );
    Ok(payload)
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CacheManifest {
    sha256: String,
    executables: BTreeSet<String>,
}

fn write_cache(path: &Path, payload: &Payload) -> Result<()> {
    let parent = path.parent().context("cache requires parent")?;
    ensure_directory(parent)?;
    let staging = tempfile::tempdir_in(parent)?;
    for (name, bytes) in &payload.files {
        atomic_write(&staging.path().join(name), bytes)?;
    }
    atomic_write(
        &staging.path().join(CACHE_MANIFEST),
        &serde_json::to_vec(&CacheManifest {
            sha256: payload.sha256.clone(),
            executables: payload.executables.clone(),
        })?,
    )?;
    match std::fs::rename(staging.path(), path) {
        Ok(()) => Ok(()),
        Err(error) if path.is_dir() => {
            let current = read_cached(path)?;
            ensure!(
                current.sha256 == payload.sha256 && current.executables == payload.executables,
                "concurrent cache write differs from requested payload"
            );
            let _ = error;
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

fn read_cached(path: &Path) -> Result<Payload> {
    let manifest: CacheManifest =
        serde_json::from_slice(&read_bounded(&path.join(CACHE_MANIFEST), 64 * 1024)?)?;
    let files = read_payload_directory(path, &[CACHE_MANIFEST])?;
    let payload = Payload::new(files, manifest.executables)?;
    ensure!(
        payload.sha256 == manifest.sha256,
        "cached content differs from its manifest"
    );
    Ok(payload)
}

pub fn read_payload_directory(path: &Path, excluded: &[&str]) -> Result<BTreeMap<String, Vec<u8>>> {
    reject_symlink_components(path)?;
    let mut files = BTreeMap::new();
    fn visit(
        root: &Path,
        path: &Path,
        files: &mut BTreeMap<String, Vec<u8>>,
        excluded: &[&str],
        depth: usize,
        budget: &mut (usize, usize),
    ) -> Result<()> {
        ensure!(depth <= 32, "payload directory nesting exceeds limit");
        for entry in std::fs::read_dir(path)? {
            budget.0 += 1;
            ensure!(
                budget.0 <= MAX_FILES * 4,
                "payload directory entry count exceeds limit"
            );
            let entry = entry?;
            let child = entry.path();
            let metadata = std::fs::symlink_metadata(&child)?;
            ensure!(
                !metadata.file_type().is_symlink(),
                "payload contains a symlink: {}",
                child.display()
            );
            let relative = child
                .strip_prefix(root)?
                .to_str()
                .context("non-UTF-8 payload path")?
                .replace('\\', "/");
            safe_path(&relative)?;
            if metadata.is_dir() {
                visit(root, &child, files, excluded, depth + 1, budget)?;
            } else {
                ensure!(metadata.is_file(), "payload contains a non-regular file");
                if excluded.contains(&relative.as_str()) {
                    continue;
                }
                let bytes = read_bounded(&child, MAX_FILE as u64)?;
                budget.1 = budget
                    .1
                    .checked_add(bytes.len())
                    .context("payload size overflow")?;
                ensure!(
                    budget.1 <= MAX_PAYLOAD && files.len() < MAX_FILES,
                    "payload exceeds byte or file limit"
                );
                files.insert(relative, bytes);
            }
        }
        Ok(())
    }
    visit(path, path, &mut files, excluded, 0, &mut (0, 0))?;
    validate_files(&files)?;
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn capability_composes_only_explicit_leads() {
        let catalog = Catalog::embedded().unwrap();
        let plan = catalog
            .capability_plan("CFO", &["Excel Expert".into()])
            .unwrap();
        assert_eq!(plan["contracts"].as_array().unwrap().len(), 2);
        assert_eq!(plan["read_sequence"].as_array().unwrap().len(), 2);
        assert!(catalog.capability("some made-up CFO title").is_err());
        assert!(catalog.check().is_ok());
    }

    #[test]
    fn bounded_read_rejects_traversal_and_huge_response() {
        let catalog = Catalog::embedded().unwrap();
        assert!(
            catalog
                .read("skill-catalog", "../SKILL.md", MAX_FILE)
                .is_err()
        );
        assert!(catalog.read("skill-catalog", "SKILL.md", 1).is_err());
        let document = catalog.read("skill-catalog", "SKILL.md", MAX_FILE).unwrap();
        assert_eq!(document.sha256, sha256(document.content.as_bytes()));
        assert!(catalog.search("", 0, 101).is_err());
        assert!(catalog.search("nonesuch123", 1, 20).is_err());
    }

    #[test]
    fn payload_paths_are_portable_and_noncolliding() {
        for path in ["/abs", "a/../b", "a//b", "a\\b", "a/NUL.md", "a/b.", "a:c"] {
            assert!(safe_path(path).is_err());
        }
        assert!(
            validate_files(&BTreeMap::from([
                ("A/a".into(), vec![]),
                ("a/b".into(), vec![])
            ]))
            .is_err()
        );
        assert!(
            validate_files(&BTreeMap::from([
                ("a".into(), vec![]),
                ("a/b".into(), vec![])
            ]))
            .is_err()
        );
    }

    #[test]
    fn archive_rejects_selected_symlink_and_retains_licenses() {
        let catalog = Catalog::embedded().unwrap();
        let skill = catalog.skill("matt-domain-modeling").unwrap();
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file("root/LICENSE", zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"terms").unwrap();
        writer
            .start_file(
                format!("root/{}/SKILL.md", skill.path),
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
        writer
            .write_all(b"---\nname: domain-modeling\n---\nHello")
            .unwrap();
        let bytes = writer.finish().unwrap().into_inner();
        let selected = select_archive(&bytes, skill).unwrap();
        assert_eq!(selected[".upstream-licenses/LICENSE"], b"terms");
        assert!(prepare_upstream(selected, skill).is_err());
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .add_symlink(
                format!("root/{}/SKILL.md", skill.path),
                "/etc/passwd",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
        let bytes = writer.finish().unwrap().into_inner();
        assert!(select_archive(&bytes, skill).is_err());
    }

    #[test]
    fn cached_payload_modification_is_detected() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().canonicalize().unwrap().join("payload");
        let payload = Catalog::embedded().unwrap().payload("debugging").unwrap();
        write_cache(&path, &payload).unwrap();
        assert_eq!(read_cached(&path).unwrap().sha256, payload.sha256);
        std::fs::write(path.join("SKILL.md"), b"changed").unwrap();
        assert!(read_cached(&path).is_err());
    }
}
