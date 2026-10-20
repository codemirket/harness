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

mod design_references;
pub use design_references::DesignContext;
mod teams;
pub use teams::{FacetSelection, ProfessionalProfile, TeamFacet};
mod workflows;
pub use workflows::Workflow;

static SKILLS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/skills");
const CATALOG: &str = include_str!("../registry/catalog.json");
const CAPABILITIES: &str = include_str!("../registry/capabilities.json");
const HARNESS: &str = include_str!("../registry/harness.json");
const NOTICES: &str = include_str!("../docs/third-party-notices.md");
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
    /// Embedded descriptions come from SKILL.md; imported entries keep reviewed metadata.
    #[serde(default)]
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
    #[serde(default = "specialist_profile")]
    pub kind: String,
    pub skills: Vec<String>,
    /// Explicit portable delivery for a project whose hosts have no shared installation.
    #[serde(default)]
    pub include_globals: bool,
}

fn specialist_profile() -> String {
    "specialist".into()
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
    pub professional: ProfessionalProfile,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkillDocument {
    pub id: String,
    pub path: String,
    pub sha256: String,
    pub content: String,
}

fn guidance_access(skill: &Skill) -> Value {
    json!({
        "body_readable":skill.delivery == "embedded",
        "metadata":"catalog_show or mirket catalog show ID",
        "read_route":match skill.delivery.as_str() {
            "embedded" => "skill_read or mirket catalog read ID; record task-bound delivery when using Mirket tracking",
            "upstream" => "Inspect pinned source, adaptation and caveats first. Explicitly select with mirket project add --skill ID, then project sync and doctor; read the installed host skill. skill_read does not fetch upstream guidance.",
            _ => "Reference only: inspect its source manually under host permissions; not installable or readable via skill_read."
        }
    })
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
    search: Vec<SearchDocument>,
    globals: Vec<String>,
    capabilities: Vec<Capability>,
    team_facets: Vec<TeamFacet>,
    workflows: Vec<Workflow>,
    capability_names: HashMap<String, usize>,
    payloads: HashMap<String, Payload>,
    document_digests: HashMap<String, BTreeMap<String, OnceLock<String>>>,
    design_references: Option<design_references::Library>,
}

struct SearchDocument {
    text: String,
    positive: String,
    names: Vec<String>,
}

/// Select complete notice sections from explicit, validated provenance. The full
/// notice bundle still accompanies runtime distributions independently.
fn notice_sections(notices: &str) -> Result<BTreeMap<&str, &str>> {
    let mut sections = BTreeMap::new();
    for section in notices.split("\n## ").skip(1) {
        let heading = section.lines().next().context("empty notice heading")?;
        ensure!(
            sections.insert(heading, section.trim_end()).is_none(),
            "duplicate notice heading: {heading}"
        );
    }
    Ok(sections)
}

fn skill_notices(
    skill: &Skill,
    sources: &BTreeMap<String, Source>,
    sections: &BTreeMap<&str, &str>,
) -> Result<Vec<u8>> {
    let ids: Vec<String> = skill
        .metadata
        .get("informed_by")
        .map(|value| serde_json::from_value(value.clone()))
        .transpose()?
        .unwrap_or_default();
    let mut selected = BTreeSet::new();
    let mut output = String::from(
        "# Applicable source notices\n\nMirket authored guidance; upstream notices apply only to their respective materials.\nRuntime and optional-package licenses accompany those distributions separately.\n",
    );
    for id in ids {
        ensure!(
            selected.insert(id.clone()),
            "duplicate provenance source for {}: {id}",
            skill.id
        );
        let source = sources
            .get(&id)
            .with_context(|| format!("unknown provenance source for {}: {id}", skill.id))?;
        let section = source
            .metadata
            .get("notice_section")
            .and_then(Value::as_str);
        let exemption = source
            .metadata
            .get("notice_exemption")
            .and_then(Value::as_str);
        ensure!(
            section.is_some() != exemption.is_some(),
            "source {id} needs exactly one notice section or explicit exemption"
        );
        if let Some(heading) = section {
            let body = sections
                .get(heading)
                .with_context(|| format!("missing notice section for {id}: {heading}"))?;
            ensure!(
                body.contains(&source.commit),
                "notice section does not identify pinned source {id}"
            );
            output.push_str("\n## ");
            output.push_str(body);
            output.push('\n');
        } else if let Some(reason) = exemption {
            ensure!(!reason.trim().is_empty(), "empty notice exemption for {id}");
            output.push_str(&format!(
                "\n## {}\n\nSource: https://github.com/{}/tree/{}\n\n{}\n",
                source.repository, source.repository, source.commit, reason
            ));
        }
    }
    Ok(output.into_bytes())
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
        Self::load_sources(CATALOG, CAPABILITIES, HARNESS)
    }

    fn load_sources(catalog: &str, capabilities: &str, harness: &str) -> Result<CatalogInner> {
        let mut data: CatalogData = serde_json::from_str(catalog)?;
        ensure!(data.schema_version == 1, "unsupported catalog schema");
        let harness: Value = serde_json::from_str(harness)?;
        let globals: Vec<String> = serde_json::from_value(harness["global_skills"].clone())?;
        let contracts: Value = serde_json::from_str(capabilities)?;
        ensure!(
            contracts["schema_version"] == 1,
            "unsupported capability schema"
        );
        let capabilities: Vec<Capability> =
            serde_json::from_value(contracts["capabilities"].clone())?;
        let team_facets: Vec<TeamFacet> = serde_json::from_value(contracts["team_facets"].clone())?;
        let workflows: Vec<Workflow> = serde_json::from_value(contracts["workflows"].clone())?;
        let notices = notice_sections(NOTICES)?;
        let mut by_id = HashMap::new();
        let mut payloads = HashMap::new();
        let mut document_digests = HashMap::new();
        let mut search = Vec::new();
        for (i, skill) in data.skills.iter_mut().enumerate() {
            validate_id(&skill.id)?;
            if let Some(discovery) = skill.metadata.get("discovery") {
                ensure!(
                    discovery == "explicit-alternative",
                    "unknown discovery policy for {}",
                    skill.id
                );
            }
            ensure!(
                by_id.insert(skill.id.clone(), i).is_none(),
                "duplicate skill {}",
                skill.id
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
                // Preserve applicable offline notices without coupling unrelated skill identities.
                files.insert(
                    ".upstream-licenses/NOTICE.md".into(),
                    skill_notices(skill, &data.sources, &notices)?,
                );
                let body = files.get("SKILL.md").context("missing SKILL.md")?;
                ensure!(
                    skill_field(body, "name")? == skill.name,
                    "skill invocation differs from catalog: {}",
                    skill.id
                );
                skill.description = skill_field(body, "description")?;
                document_digests.insert(
                    skill.id.clone(),
                    files
                        .keys()
                        .map(|path| (path.clone(), OnceLock::new()))
                        .collect(),
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
                for path in skill
                    .license_files
                    .iter()
                    .chain(skill.extra_files.keys())
                    .chain(skill.extra_files.values())
                {
                    safe_path(path)?;
                }
            }
            if matches!(skill.delivery.as_str(), "upstream" | "reference") {
                let source = data
                    .sources
                    .get(skill.source.as_ref().context("missing skill source")?)
                    .context("unknown skill source")?;
                validate_source(source)?;
            }
            ensure!(
                !skill.description.trim().is_empty(),
                "missing description: {}",
                skill.id
            );
            let mut names = vec![skill.id.to_lowercase(), skill.name.to_lowercase()];
            for role in capabilities.iter().filter(|role| role.lead == skill.id) {
                names.extend(
                    std::iter::once(&role.id)
                        .chain(std::iter::once(&role.title))
                        .chain(&role.aliases)
                        .map(|name| name.to_lowercase()),
                );
            }
            let positive = format!(
                "{} {} {}",
                names.join(" "),
                skill.category,
                skill.tags.join(" ")
            )
            .to_lowercase();
            search.push(SearchDocument {
                text: format!("{} {}", positive, skill.description.to_lowercase()),
                positive,
                names,
            });
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
            capability.professional.validate(&capability.id)?;
            for id in std::iter::once(&capability.lead).chain(&capability.support) {
                ensure!(
                    payloads.contains_key(id),
                    "capability requires an embedded skill: {id}"
                );
            }
            ensure!(
                !capability.acceptance.is_empty()
                    && capability
                        .acceptance
                        .iter()
                        .all(|item| !item.trim().is_empty())
                    && [
                        &capability.title,
                        &capability.deliverable,
                        &capability.failure_probe,
                        &capability.prerequisites
                    ]
                    .iter()
                    .all(|item| !item.trim().is_empty()),
                "incomplete capability contract: {}",
                capability.id
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
        teams::validate_facets(&team_facets, &capabilities)?;
        workflows::validate(&workflows, &capabilities)?;
        let design_references = payloads
            .get(design_references::OWNER)
            .map(design_references::Library::load)
            .transpose()?;
        Ok(CatalogInner {
            data,
            by_id,
            search,
            globals,
            capabilities,
            team_facets,
            workflows,
            capability_names,
            payloads,
            document_digests,
            design_references,
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
    /// Includes all embedded guidance, provenance and execution contracts.
    pub fn fingerprint(&self) -> String {
        let mut hash = Sha256::new();
        for bytes in [
            CATALOG.as_bytes(),
            CAPABILITIES.as_bytes(),
            HARNESS.as_bytes(),
            INSTRUCTIONS,
            NOTICES.as_bytes(),
        ] {
            hash.update((bytes.len() as u64).to_le_bytes());
            hash.update(bytes);
        }
        let mut payloads: Vec<_> = self.0.payloads.iter().collect();
        payloads.sort_by_key(|(id, _)| *id);
        for (id, payload) in payloads {
            hash.update(id.as_bytes());
            hash.update(payload.sha256.as_bytes());
        }
        format!("{:x}", hash.finalize())
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

    /// Selected discovery metadata, without payloads or large patch tables.
    /// This is data to assess a choice; it never fetches or executes guidance.
    pub fn describe(&self, id: &str) -> Result<Value> {
        let skill = self.skill(id)?;
        let mut result = json!({
            "id":skill.id,"name":skill.name,"description":skill.description,
            "scope":skill.scope,"delivery":skill.delivery,"path":skill.path,
            "category":skill.category,"tags":skill.tags,"agents":skill.agents,
            "requires":skill.requires,"conflicts":skill.conflicts,
            "adaptation":skill.adaptation,"source_id":skill.source,"discovery":skill.metadata.get("discovery").and_then(Value::as_str).unwrap_or("default"),
            "license_files":skill.license_files,"sha256":skill.sha256,
            "installed_sha256":skill.installed_sha256,
            "dependencies":skill.metadata.get("dependencies"),
            "caveats":skill.metadata.get("caveats"),
            "asset_requirements":skill.metadata.get("asset_requirements"),
            "companion_requirements":skill.metadata.get("companion_requirements"),
            "optional_companions":skill.metadata.get("optional_companions"),
            "url":skill.metadata.get("url"),"license":skill.metadata.get("license")
        });
        self.attach_discovery(skill, &mut result);
        ensure!(
            serde_json::to_vec(&result)?.len() <= 32 * 1024,
            "selected metadata exceeds 32 KiB; inspect mirket catalog show {id} through the host CLI"
        );
        Ok(result)
    }

    /// Preserve the CLI's full entry metadata while exposing the same reference
    /// discovery routes as the bounded MCP describe response.
    pub fn show(&self, id: &str) -> Result<Value> {
        let skill = self.skill(id)?;
        let mut result = serde_json::to_value(skill)?;
        self.attach_discovery(skill, &mut result);
        Ok(result)
    }

    fn attach_discovery(&self, skill: &Skill, result: &mut Value) {
        result["guidance_access"] = guidance_access(skill);
        result["source_pin"] = json!(skill.source.as_ref().and_then(|id| self.sources().get(id)));
        if skill.id == design_references::OWNER
            && let Some(library) = &self.0.design_references
        {
            result["reference_library"] = library.discovery();
        }
    }

    pub fn search(&self, query: &str, offset: usize, limit: usize) -> Result<Value> {
        self.search_result(query, offset, limit)
            .map(|(result, _)| result)
    }

    fn search_result(&self, query: &str, offset: usize, limit: usize) -> Result<(Value, bool)> {
        ensure!(query.len() <= 1024, "search exceeds 1024 bytes");
        ensure!((1..=100).contains(&limit), "limit must be 1..=100");
        let terms: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
        let phrase = terms.join(" ");
        let reference_matches = self
            .0
            .design_references
            .as_ref()
            .map(|library| library.matching(&terms))
            .unwrap_or_default();
        let has_reference_matches = !reference_matches.is_empty();
        let mut matches: Vec<_> = self
            .skills()
            .iter()
            .enumerate()
            .filter_map(|(i, skill)| {
                // Alternative whole-workflow suites remain explicitly addressable.
                // Their source name (for example "superpowers") opts into discovery.
                let explicit_alternative = skill.metadata.get("discovery").and_then(Value::as_str)
                    == Some("explicit-alternative");
                if explicit_alternative
                    && phrase != skill.id
                    && !skill
                        .source
                        .as_ref()
                        .is_some_and(|source| terms.contains(&source.to_lowercase()))
                {
                    return None;
                }
                let document = &self.0.search[i];
                let references = if skill.id == design_references::OWNER {
                    reference_matches.clone()
                } else {
                    Vec::new()
                };
                if !terms.iter().all(|term| document.text.contains(term)) && references.is_empty() {
                    return None;
                }
                let positive = terms.iter().all(|term| document.positive.contains(term))
                    || !references.is_empty();
                let delivery = match skill.delivery.as_str() {
                    "embedded" => 2,
                    "upstream" => 1,
                    _ => 0,
                };
                let name_match = document
                    .names
                    .iter()
                    .map(|name| {
                        if name == &phrase {
                            3
                        } else if name.starts_with(&phrase) {
                            2
                        } else if name.contains(&phrase) {
                            1
                        } else {
                            0
                        }
                    })
                    .max()
                    .unwrap_or(0);
                let exact_tags = terms
                    .iter()
                    .filter(|term| skill.tags.iter().any(|tag| tag.eq_ignore_ascii_case(term)))
                    .count();
                Some((
                    (
                        skill.id == phrase
                            || references.iter().any(|instance| instance.exact(&phrase)),
                        positive,
                        name_match == 3,
                        exact_tags,
                        name_match,
                        delivery,
                    ),
                    skill,
                    references,
                ))
            })
            .collect();
        matches.sort_unstable_by(|(a_rank, a, _), (b_rank, b, _)| {
            b_rank.cmp(a_rank).then_with(|| a.id.cmp(&b.id))
        });
        ensure!(offset <= matches.len(), "offset exceeds result count");
        let page: Vec<Value> = matches
            .iter()
            .skip(offset)
            .take(limit)
            .map(|(_, skill, references)| {
                let mut item = json!({
                    "id": skill.id, "name": skill.name, "description": skill.description,
                    "scope": skill.scope, "delivery": skill.delivery, "category": skill.category,
                    "installable": skill.delivery == "embedded" || skill.delivery == "upstream",
                    "guidance_access": guidance_access(skill), "discovery":skill.metadata.get("discovery").and_then(Value::as_str).unwrap_or("default")
                });
                if skill.id == design_references::OWNER
                    && let Some(library) = &self.0.design_references
                {
                    item["reference_library"] = library.discovery();
                    item["reference_matches"] = design_references::Library::summaries(references);
                }
                item
            })
            .collect();
        Ok((
            json!({"items":page,"total":matches.len(),"offset":offset,
            "next_offset": (offset + limit < matches.len()).then_some(offset + limit)}),
            has_reference_matches,
        ))
    }

    /// Structured design requirements supplement keyword discovery. Coverage is
    /// an inventory lookup, not a claim that the host validated the current task.
    pub fn search_with_design_context(
        &self,
        query: &str,
        offset: usize,
        limit: usize,
        context: Option<&DesignContext>,
    ) -> Result<Value> {
        let coverage = context
            .map(|context| {
                self.0
                    .design_references
                    .as_ref()
                    .context("design reference library is unavailable")?
                    .coverage_for(context)
            })
            .transpose()?;
        let (mut result, has_reference_matches) = self.search_result(query, offset, limit)?;
        if let Some(library) = &self.0.design_references
            && !has_reference_matches
            && let Some(candidates) = library.partial_candidates(query)?
        {
            result["design_reference_candidates"] = candidates;
        }
        if let Some(coverage) = coverage {
            result["design_coverage"] = coverage;
        }
        Ok(result)
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
            .with_context(|| format!("skill is not embedded: {id}; inspect catalog_show or mirket catalog show {id} for its pinned source, caveats and read route"))?;
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
            sha256: self.0.document_digests[id][path]
                .get_or_init(|| sha256(bytes))
                .clone(),
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

    fn guidance_plan(&self, contracts: &[&Capability]) -> Result<(Vec<Value>, Vec<String>)> {
        let mut seen = BTreeSet::new();
        let mut reads = Vec::new();
        let mut project_skills = Vec::new();
        for row in contracts {
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
            let digest =
                self.0.document_digests[&row.lead]["SKILL.md"].get_or_init(|| sha256(body));
            reads.push(json!({"id":row.lead,"name":skill.name,"path":"SKILL.md",
                "sha256":digest,"payload_sha256":payload.sha256,"bytes":body.len(),
                "delivery":if global {"global"} else {"project"}}));
            if !global {
                project_skills.push(row.lead.clone());
            }
        }
        Ok((reads, project_skills))
    }

    pub fn capability_plan(&self, name: &str, supporting: &[String]) -> Result<Value> {
        ensure!(
            supporting.len() <= 12,
            "at most 12 supporting capabilities can be composed"
        );
        let primary = self.capability(name)?;
        let mut contracts = vec![primary];
        for name in supporting {
            let row = self.capability(name).with_context(|| {
                "supporting entries must be capability names or role aliases; for optional skill IDs use skill_read or mirket catalog read ID"
            })?;
            if !contracts.iter().any(|existing| existing.id == row.id) {
                contracts.push(row);
            }
        }
        let (reads, project_skills) = self.guidance_plan(&contracts)?;
        Ok(
            json!({"primary":primary.id,"contracts":contracts,"read_sequence":reads,
            "worker_workflows":self.workflow_choices(None),
            "role_workflows":self.workflow_choices(Some(&primary.id)),
            "worker_request_defaults":{"primary_role":primary.id,"assigned_tools":[]},
            "worker_routing":"Declare bounded task_needs with workflow IDs and concrete reasons. The owned router loads the selected role and task-specialist skills, selects scoped default tools when assigned_tools is empty, and rejects tools outside those needs. A role never grants permissions or expands a workflow's tools.",
            "project_skills":project_skills,"runtime_status":"not_checked",
            "activation_status":"not_observed",
            "team_route":"When independent professional assignments would help, team_facets and team_plan can compose briefs with task-specific reasons. The host chooses useful assignments, dispatches actual independent workers and integrates their evidence.",
            "authority":"Expertise does not grant execution permissions or professional authority.",
            "support_policy":"Contract support IDs are optional skills: read one with skill_read or mirket catalog read ID and register it with mirket project add --skill ID only when needed. MCP supporting and CLI --with compose up to 12 capability names or role aliases from mirket capabilities list, not optional skill IDs.",
            "next_steps":["Read useful selected guidance and reuse unchanged reads. Choose durable tracking when coordination, resumption or user/project requirements warrant it; keep sufficient existing workflows authoritative. Mirket task completion requires selected guidance delivery and current evidence.",
                "If choosing to install missing project expertise, use mirket project add, then sync and doctor.",
                "Inspect available tools and actual task inputs.",
                "Deliver the requested result and verify it with task-relevant checks. Record evidence in the existing workflow when useful or required; Mirket task tracking requires its acceptance and failure-probe artifacts."]}),
        )
    }

    pub fn selection(
        &self,
        profiles: &[String],
        ids: &[String],
        target: &str,
    ) -> Result<Vec<&Skill>> {
        crate::target::Target::parse(target)?;
        let mut selected = Vec::new();
        let mut visiting = BTreeSet::new();
        let mut seen = BTreeSet::new();
        let mut explicit = ids.to_vec();
        for profile in profiles {
            let profile = self
                .profiles()
                .get(profile)
                .with_context(|| format!("unknown profile: {profile}"))?;
            if profile.include_globals {
                explicit.extend(self.global_skills().iter().cloned());
            }
        }
        let local_globals: BTreeSet<_> = explicit
            .iter()
            .filter(|id| self.global_skills().contains(id))
            .map(String::as_str)
            .collect();
        for id in profiles
            .iter()
            .flat_map(|name| &self.profiles()[name].skills)
            .chain(&explicit)
        {
            self.visit(
                id,
                target,
                &local_globals,
                &mut visiting,
                &mut seen,
                &mut selected,
            )?;
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
        local_globals: &BTreeSet<&str>,
        visiting: &mut BTreeSet<String>,
        seen: &mut BTreeSet<String>,
        selected: &mut Vec<&'a Skill>,
    ) -> Result<()> {
        let skill = self.skill(id)?;
        if self.global_skills().iter().any(|global| global == id) && !local_globals.contains(id) {
            return Ok(());
        }
        ensure!(!visiting.contains(id), "cyclic skill requirement: {id}");
        if seen.contains(id) {
            return Ok(());
        }
        ensure!(
            (skill.scope == "project" || local_globals.contains(id))
                && matches!(skill.delivery.as_str(), "embedded" | "upstream"),
            "skill is not installable: {id}"
        );
        ensure!(
            skill.agents.is_empty() || skill.agents.iter().any(|agent| agent == target),
            "skill {id} does not support {target}"
        );
        visiting.insert(id.into());
        for dependency in &skill.requires {
            self.visit(dependency, target, local_globals, visiting, seen, selected)?;
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
            ensure!(
                ["starter", "specialist", "alternative"].contains(&profile.kind.as_str()),
                "invalid profile kind: {name}"
            );
            for id in &profile.skills {
                self.skill(id)?;
            }
            for target in ["codex", "claude"] {
                self.selection(std::slice::from_ref(name), &[], target)
                    .with_context(|| format!("invalid profile {name} for {target}"))?;
            }
        }
        Ok(
            json!({"ok":true,"status":"valid","skills":self.skills().len(),"embedded":self.0.payloads.len(),
            "capabilities":self.capabilities().len(),"profiles":self.profiles().len(),
            "global_skills":self.global_skills().len(),
            "design_reference_instances":self.0.design_references.as_ref().map_or(0, |library| library.len()),
            "runtime_status":"not_checked"}),
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

pub(crate) fn skill_field(bytes: &[u8], field: &str) -> Result<String> {
    let text = std::str::from_utf8(bytes).context("skill body is not UTF-8")?;
    let mut lines = text.lines();
    ensure!(
        lines.next() == Some("---"),
        "skill body requires frontmatter"
    );
    let prefix = format!("{field}:");
    let mut found = None;
    for line in lines {
        if line == "---" {
            return found.with_context(|| format!("skill frontmatter has no {field}"));
        }
        if let Some(value) = line.strip_prefix(&prefix) {
            ensure!(
                found.is_none(),
                "duplicate skill frontmatter field: {field}"
            );
            let value = value.trim();
            ensure!(
                !value.is_empty() && !value.starts_with(['|', '>']),
                "skill {field} requires an inline value"
            );
            found = Some(
                value
                    .strip_prefix('"')
                    .and_then(|v| v.strip_suffix('"'))
                    .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                    .unwrap_or(value)
                    .to_owned(),
            );
        }
    }
    bail!("skill frontmatter is not closed")
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
        skill_field(files.get("SKILL.md").context("missing SKILL.md")?, "name")? == skill.name,
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
        skill_field(
            payload
                .files
                .get("SKILL.md")
                .context("missing skill body")?,
            "name"
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
    fn skill_fields_require_closed_inline_frontmatter() {
        assert_eq!(
            skill_field(
                b"---\nname: test\ndescription: 'Use this skill.'\n---\ndescription: body text",
                "description"
            )
            .unwrap(),
            "Use this skill."
        );
        for invalid in [
            b"---\nname: test\n\n# Body\ndescription: body text".as_slice(),
            b"---\ndescription: header text\n".as_slice(),
            b"---\nname: test\n---\ndescription: body text".as_slice(),
            b"---\ndescription: >-\n  folded text\n---".as_slice(),
            b"---\ndescription: first\ndescription: second\n---".as_slice(),
        ] {
            assert!(skill_field(invalid, "description").is_err());
        }
    }

    #[test]
    fn reference_entries_require_valid_source_pins() {
        for invalid in ["missing", "unknown", "commit", "repository"] {
            let mut data: Value = serde_json::from_str(CATALOG).unwrap();
            let index = data["skills"]
                .as_array()
                .unwrap()
                .iter()
                .position(|skill| skill["id"] == "ecc-research-ops")
                .unwrap();
            let original = data["skills"][index]["source"].as_str().unwrap().to_owned();
            let source_id = "reference-only-test";
            data["sources"][source_id] = data["sources"][&original].clone();
            let reference = &mut data["skills"][index];
            reference["source"] = json!(source_id);
            match invalid {
                "missing" => {
                    reference.as_object_mut().unwrap().remove("source");
                }
                "unknown" => reference["source"] = json!("unknown-source"),
                "commit" => data["sources"][source_id]["commit"] = json!("0".repeat(39)),
                "repository" => {
                    data["sources"][source_id]["repository"] = json!("../invalid/repository")
                }
                _ => unreachable!(),
            }
            assert!(
                Catalog::load_sources(&data.to_string(), CAPABILITIES, HARNESS).is_err(),
                "{invalid}"
            );
        }
    }

    #[test]
    fn embedded_discovery_uses_the_skill_trigger_and_external_metadata_stays_required() {
        let mut data: Value = serde_json::from_str(CATALOG).unwrap();
        let skills = data["skills"].as_array_mut().unwrap();
        skills
            .iter_mut()
            .find(|s| s["id"] == "skill-catalog")
            .unwrap()["description"] =
            json!("Misleading legacy discovery text without the routine-work boundary");
        let catalog = Catalog(Arc::new(
            Catalog::load_sources(&data.to_string(), CAPABILITIES, HARNESS).unwrap(),
        ));
        let page = catalog.search("catalog", 0, 100).unwrap();
        let discovered = page["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["id"] == "skill-catalog")
            .unwrap();
        assert!(
            discovered["description"]
                .as_str()
                .unwrap()
                .ends_with("Skip trivial edits and reuse unchanged selections.")
        );
        assert!(
            !discovered["description"]
                .as_str()
                .unwrap()
                .contains("Misleading legacy")
        );

        let upstream = data["skills"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|s| s["id"] == "matt-research")
            .unwrap();
        upstream.as_object_mut().unwrap().remove("description");
        let error = match Catalog::load_sources(&data.to_string(), CAPABILITIES, HARNESS) {
            Ok(_) => panic!("external entries need their reviewed discovery description"),
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains("missing description: matt-research")
        );
    }

    #[test]
    fn notices_follow_applicable_provenance_without_unrelated_payload_churn() {
        let data: CatalogData = serde_json::from_str(CATALOG).unwrap();
        let backend = data
            .skills
            .iter()
            .find(|s| s.id == "backend-engineering")
            .unwrap();
        let motion = data
            .skills
            .iter()
            .find(|s| s.id == "advanced-motion")
            .unwrap();
        let changed = NOTICES.replace(
            "Copyright (c) 2025 LottieFiles",
            "Copyright (c) 2025–2026 LottieFiles",
        );
        assert_ne!(changed, NOTICES);
        let sections = notice_sections(NOTICES).unwrap();
        let changed_sections = notice_sections(&changed).unwrap();
        assert_eq!(
            skill_notices(backend, &data.sources, &sections).unwrap(),
            skill_notices(backend, &data.sources, &changed_sections).unwrap()
        );
        let before = skill_notices(motion, &data.sources, &sections).unwrap();
        let after = skill_notices(motion, &data.sources, &changed_sections).unwrap();
        assert_ne!(before, after);
        let catalog = Catalog::embedded().unwrap();
        let mut payload = catalog.payload("advanced-motion").unwrap();
        let original = payload.sha256.clone();
        payload
            .files
            .insert(".upstream-licenses/NOTICE.md".into(), after);
        assert_ne!(payload_hash(&payload.files), original);
        let body = payload.files.get_mut("SKILL.md").unwrap();
        body.extend_from_slice(b"\nChanged meaningful guidance.\n");
        assert_ne!(payload_hash(&payload.files), original);
        let notice = String::from_utf8(before).unwrap();
        assert!(notice.contains("Copyright (c) 2025 LottieFiles"));
        assert!(notice.contains("Permission is hereby granted, free of charge"));
        assert!(!notice.contains("Wondel.ai"));
    }

    #[test]
    fn embedded_provenance_cannot_silently_omit_required_notices() {
        let mut data: CatalogData = serde_json::from_str(CATALOG).unwrap();
        let motion = data
            .skills
            .iter()
            .find(|s| s.id == "advanced-motion")
            .unwrap()
            .clone();
        let sections = notice_sections(NOTICES).unwrap();
        let missing = NOTICES.replace("## LottieFiles/motion-design-skill", "## missing");
        assert!(
            skill_notices(&motion, &data.sources, &notice_sections(&missing).unwrap()).is_err()
        );
        data.sources
            .get_mut("lottiefiles-motion")
            .unwrap()
            .metadata
            .remove("notice_section");
        assert!(skill_notices(&motion, &data.sources, &sections).is_err());
        for skill in data
            .skills
            .iter()
            .filter(|s| s.delivery == "embedded" && s.id != "advanced-motion")
        {
            // Every current authored owner is checked by Catalog::load as well.
            assert!(skill_notices(skill, &data.sources, &sections).is_ok());
        }
    }

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
    fn professional_catalog_rejects_incomplete_profiles_and_invalid_team_assignments() {
        for change in 0..5 {
            let mut contracts: Value = serde_json::from_str(CAPABILITIES).unwrap();
            match change {
                0 => {
                    contracts["capabilities"][0]["professional"]["perspective"] = json!("");
                }
                1 => {
                    contracts["capabilities"][0]["professional"]["method"] = json!(["Inspect"]);
                }
                2 => {
                    contracts["team_facets"][0]["assignments"][0]["role"] =
                        json!("unknown-professional");
                }
                3 => {
                    contracts["team_facets"][0]["assignments"][0]["mode"] = json!("executed");
                }
                _ => {
                    contracts["team_facets"][1]["id"] = contracts["team_facets"][0]["id"].clone();
                }
            }
            assert!(
                Catalog::load_sources(CATALOG, &contracts.to_string(), HARNESS).is_err(),
                "mutation {change}"
            );
        }
    }

    #[test]
    fn search_prefers_relevance_over_delivery_and_keeps_availability_as_a_tie_break() {
        let catalog = Catalog::embedded().unwrap();
        for (query, expected) in [
            ("backend", "backend-engineering"),
            ("context", "context-management"),
            ("research", "matt-research"),
            ("CFO", "financial-analysis"),
            ("keyboard", "accessibility-engineering"),
            ("accessibility-audit", "accessibility-engineering"),
            ("accessible", "accessibility-engineering"),
            ("logo", "svg-creation"),
            ("generic copy", "marketing-writing"),
        ] {
            let page = catalog.search(query, 0, 5).unwrap();
            assert_eq!(page["items"][0]["id"], expected, "{query}");
        }
        let exact = catalog.search("matt-research", 0, 5).unwrap();
        assert_eq!(exact["items"][0]["id"], "matt-research");
        let all = catalog.search("research", 0, 100).unwrap();
        let first = catalog.search("research", 0, 2).unwrap();
        let second = catalog.search("research", 2, 2).unwrap();
        assert_eq!(
            first["items"],
            json!(&all["items"].as_array().unwrap()[..2])
        );
        assert_eq!(
            second["items"],
            json!(&all["items"].as_array().unwrap()[2..4])
        );

        let testing = catalog.search("testing", 0, 100).unwrap();
        let ids: Vec<_> = testing["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["id"].as_str().unwrap())
            .collect();
        let embedded = ids.iter().position(|id| *id == "test-design").unwrap();
        let upstream = ids.iter().position(|id| *id == "matt-tdd").unwrap();
        assert!(
            embedded < upstream,
            "equally relevant matches keep embedded delivery ahead: {ids:?}"
        );
    }

    #[test]
    fn discovery_reports_read_routes_and_bounded_selected_metadata() {
        let catalog = Catalog::embedded().unwrap();
        for query in ["anti slop", "anti-slop", "anti ai-slop"] {
            let page = catalog.search(query, 0, 10).unwrap();
            let items = page["items"].as_array().unwrap();
            assert_eq!(items[0]["id"], "interface-design");
            assert!(
                items
                    .iter()
                    .take(2)
                    .any(|row| row["id"] == "marketing-writing")
            );
            assert_eq!(items[0]["guidance_access"]["body_readable"], true);
        }
        let upstream = catalog.search("rust testing", 0, 10).unwrap();
        let row = upstream["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == "ecc-rust-testing")
            .unwrap();
        assert_eq!(row["installable"], true);
        assert_eq!(row["guidance_access"]["body_readable"], false);
        let selected = catalog.describe("ecc-rust-testing").unwrap();
        assert_eq!(selected["source_pin"]["repository"], "affaan-m/ECC");
        assert_eq!(
            selected["source_pin"]["commit"],
            "ef648e01899ba3e8dc6371642deaaf64b4477775"
        );
        assert!(
            selected["adaptation"]
                .as_str()
                .unwrap()
                .contains("requirements")
        );
        assert!(!selected["caveats"].as_array().unwrap().is_empty());
        assert!(selected.get("replacements").is_none());
        assert!(
            catalog
                .read("ecc-rust-testing", "SKILL.md", MAX_FILE)
                .is_err()
        );
        assert!(catalog.describe("missing-skill").is_err());
        for skill in catalog.skills() {
            let selected = catalog.describe(&skill.id).unwrap();
            let shown = catalog.show(&skill.id).unwrap();
            assert_eq!(shown["guidance_access"], selected["guidance_access"]);
            assert_eq!(shown["source_pin"], selected["source_pin"]);
            assert_eq!(shown["reference_library"], selected["reference_library"]);
            assert!(serde_json::to_vec(&selected).unwrap().len() <= 32 * 1024);
            if skill.delivery == "reference" {
                assert_eq!(selected["guidance_access"]["body_readable"], false);
                assert!(
                    selected["guidance_access"]["read_route"]
                        .as_str()
                        .unwrap()
                        .contains("Reference only")
                );
            }
        }
    }

    #[test]
    fn catalog_check_rejects_conflicting_profile_composition() {
        let mut data: Value = serde_json::from_str(CATALOG).unwrap();
        data["profiles"]["conflicting"] = json!({
            "description":"Invalid combined invocation ownership",
            "skills":["matt-prototype","emil-prototype"]
        });
        let catalog = Catalog(Arc::new(
            Catalog::load_sources(&data.to_string(), CAPABILITIES, HARNESS).unwrap(),
        ));
        let error = catalog.check().unwrap_err().to_string();
        assert!(error.contains("invalid profile conflicting"), "{error}");
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
