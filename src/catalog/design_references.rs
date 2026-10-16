//! Authored design lessons travel in the existing embedded skill payload.
//! Their external sources are records to inspect, never install or execute.
use super::{Payload, safe_path, valid_digest, validate_id};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

mod coverage;
mod selection;
pub use coverage::DesignContext;

pub(super) const OWNER: &str = "design-inspiration";
const MANIFEST: &str = "references/design-languages/library.json";
const READ_LIMIT: usize = 48 * 1024;
const MATCH_LIMIT: usize = 5;
const RECOVERY_GUIDE: &str = "references/design-languages/recovery-feedback.md";
const RECOVERY_EXAMPLE: &str = "assets/design-languages/recovery-feedback/index.html";
const COMPARISON_GUIDE: &str = "references/design-languages/comparison-at-use-size.md";
const COMPARISON_EXAMPLE: &str = "assets/design-languages/comparison-at-use-size/index.html";

#[derive(Debug)]
pub(super) struct Library {
    index: String,
    instances: Vec<Instance>,
    coverage: coverage::Coverage,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    owner: String,
    index: String,
    license: License,
    coverage: String,
    instances: Vec<Record>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Record {
    id: String,
    record: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct License {
    spdx: String,
    path: String,
    scope: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Instance {
    id: String,
    title: String,
    language: String,
    domain: String,
    aliases: Vec<String>,
    tags: Vec<String>,
    summary: String,
    use_when: String,
    avoid_when: String,
    card: String,
    example: String,
    sources: Vec<Source>,
    original_example: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Source {
    title: String,
    url: String,
    edition: String,
    checked_on: String,
    rights: String,
    license_url: Option<String>,
    sha256: Option<String>,
    capture_note: String,
}

fn nonempty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn https_url(value: &str) -> bool {
    // URL parsing is already provided by the catalog's HTTP dependency.
    reqwest::Url::parse(value).is_ok_and(|url| {
        url.scheme() == "https"
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
    })
}

fn resource(files: &BTreeMap<String, Vec<u8>>, path: &str) -> Result<()> {
    safe_path(path)?;
    let bytes = files
        .get(path)
        .with_context(|| format!("missing design reference resource: {path}"))?;
    ensure!(
        !bytes.is_empty() && bytes.len() <= READ_LIMIT,
        "design reference exceeds bounded text read or is empty: {path}"
    );
    std::str::from_utf8(bytes).context("design reference must be readable UTF-8")?;
    Ok(())
}

fn route(path: &str) -> Value {
    json!({"skill_id":OWNER,"path":path,"uri":format!("mirket://skills/{OWNER}/{path}")})
}

fn comparison_practice() -> Value {
    json!({"guide":route(COMPARISON_GUIDE),"example":route(COMPARISON_EXAMPLE),
        "use_when":"Optional study of independently wrapping headings that displace intended partner graphics, or overflowing required records with no visible continuation cue."})
}

impl Library {
    pub(super) fn load(payload: &Payload) -> Result<Self> {
        resource(&payload.files, MANIFEST)?;
        let bytes = payload
            .files
            .get(MANIFEST)
            .context("missing design language manifest")?;
        let manifest: Manifest =
            serde_json::from_slice(bytes).context("invalid design language manifest")?;
        ensure!(
            manifest.schema_version == 2 && manifest.owner == OWNER,
            "unsupported design language library"
        );
        ensure!(
            (1..=64).contains(&manifest.instances.len()),
            "design reference count must be 1..=64"
        );
        ensure!(
            manifest.index.starts_with("references/design-languages/")
                && manifest.index.ends_with(".md"),
            "invalid design reference index"
        );
        let files = &payload.files;
        resource(files, &manifest.index)?;
        resource(files, RECOVERY_GUIDE)?;
        resource(files, RECOVERY_EXAMPLE)?;
        resource(files, COMPARISON_GUIDE)?;
        resource(files, COMPARISON_EXAMPLE)?;
        ensure!(
            manifest.license.spdx == "MIT" && nonempty(&manifest.license.scope),
            "missing original-example license scope"
        );
        resource(files, &manifest.license.path)?;
        let mut ids = BTreeSet::new();
        let mut instances = Vec::new();
        for record in &manifest.instances {
            validate_id(&record.id)?;
            ensure!(
                ids.insert(&record.id),
                "duplicate design reference: {}",
                record.id
            );
            ensure!(
                record.record == record_path(&record.id),
                "design record must belong to its ID: {}",
                record.id
            );
            resource(files, &record.record)?;
            let instance: Instance = serde_json::from_slice(&files[&record.record])
                .with_context(|| format!("invalid design reference record: {}", record.id))?;
            ensure!(
                instance.id == record.id,
                "design record identity mismatch: {}",
                record.id
            );
            ensure!(
                ["interface", "graphic", "motion", "data"].contains(&instance.domain.as_str()),
                "invalid design reference domain: {}",
                instance.id
            );
            ensure!(
                [
                    &instance.title,
                    &instance.language,
                    &instance.summary,
                    &instance.use_when,
                    &instance.avoid_when,
                ]
                .into_iter()
                .all(|value| nonempty(value)),
                "incomplete design reference: {}",
                instance.id
            );
            ensure!(
                !instance.aliases.is_empty()
                    && !instance.tags.is_empty()
                    && instance
                        .aliases
                        .iter()
                        .chain(&instance.tags)
                        .all(|value| nonempty(value)),
                "missing design reference routing: {}",
                instance.id
            );
            ensure!(
                instance.card == format!("references/design-languages/{}.md", instance.id)
                    && instance.example
                        == format!("assets/design-languages/{}/index.html", instance.id),
                "design reference routes must belong to its ID: {}",
                instance.id
            );
            resource(files, &instance.card)?;
            resource(files, &instance.example)?;
            ensure!(
                instance.original_example,
                "external assets require a separate admission review: {}",
                instance.id
            );
            ensure!(
                !instance.sources.is_empty(),
                "missing design reference source: {}",
                instance.id
            );
            let mut urls = BTreeSet::new();
            for source in &instance.sources {
                ensure!(
                    https_url(&source.url) && urls.insert(&source.url),
                    "invalid or duplicate design reference source: {}",
                    instance.id
                );
                ensure!(
                    source.license_url.as_deref().is_none_or(https_url),
                    "invalid reference license URL: {}",
                    instance.id
                );
                ensure!(
                    [
                        &source.title,
                        &source.edition,
                        &source.rights,
                        &source.capture_note
                    ]
                    .into_iter()
                    .all(|value| nonempty(value)),
                    "missing design source scope: {}",
                    instance.id
                );
                let date = source.checked_on.as_bytes();
                ensure!(
                    date.len() == 10
                        && date[4] == b'-'
                        && date[7] == b'-'
                        && date
                            .iter()
                            .enumerate()
                            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit()),
                    "invalid design source check date: {}",
                    instance.id
                );
                ensure!(
                    source.sha256.as_deref().is_some_and(valid_digest)
                        || (source.sha256.is_none()
                            && source.capture_note.starts_with("Unavailable:")),
                    "source needs a valid capture digest or explicit unavailable record: {}",
                    instance.id
                );
            }
            instances.push(instance);
        }
        ensure!(
            manifest.coverage == coverage::PATH,
            "invalid design coverage route"
        );
        let coverage = coverage::Coverage::load(files, &instances)?;
        Ok(Self {
            index: manifest.index,
            instances,
            coverage,
        })
    }

    pub(super) fn len(&self) -> usize {
        self.instances.len()
    }

    pub(super) fn discovery(&self) -> Value {
        json!({"instances":self.len(),"index":route(&self.index),"manifest":route(MANIFEST),
            "record_pattern":"references/design-languages/instances/ID.json",
            "coverage":route(coverage::PATH),"coverage_guide":route("references/design-languages/coverage.md"),
            "recovery_practice":{"guide":route(RECOVERY_GUIDE),"example":route(RECOVERY_EXAMPLE)},
            "comparison_practice":comparison_practice(),
            "source_policy":"External sources are link-only. Original lessons and examples are MIT; inspect each source's rights and capture limits before reuse."})
    }

    pub(super) fn coverage_for(&self, context: &DesignContext) -> Result<Value> {
        self.coverage.lookup(context, &self.instances)
    }

    pub(super) fn partial_candidates(&self, query: &str) -> Result<Option<Value>> {
        selection::partial_candidates(&self.instances, query)
    }

    pub(super) fn matching(&self, terms: &[String]) -> Vec<&Instance> {
        if terms.is_empty() {
            return Vec::new();
        }
        let phrase = terms.join(" ");
        let mut matches: Vec<_> = self
            .instances
            .iter()
            .filter(|instance| {
                let text = format!(
                    "{} {} {} {} {} {} {} {}",
                    instance.id,
                    instance.title,
                    instance.language,
                    instance.domain,
                    instance.aliases.join(" "),
                    instance.tags.join(" "),
                    instance.summary,
                    instance.use_when
                )
                .to_lowercase();
                terms.iter().all(|term| text.contains(term))
            })
            .collect();
        matches.sort_by(|a, b| {
            b.exact(&phrase)
                .cmp(&a.exact(&phrase))
                .then_with(|| a.id.cmp(&b.id))
        });
        matches
    }

    pub(super) fn summaries(matches: &[&Instance]) -> Value {
        json!({"total":matches.len(),"shown":matches.len().min(MATCH_LIMIT),
            "items":matches.iter().take(MATCH_LIMIT).map(|instance| json!({
                "id":instance.id,"title":instance.title,"domain":instance.domain,
                "language":instance.language,"summary":instance.summary,
                "record":route(&record_path(&instance.id)),
                "lesson":route(&instance.card),"example":route(&instance.example),
                "contrast_query":"?variant=misleading"
            })).collect::<Vec<_>>()})
    }
}

fn record_path(id: &str) -> String {
    format!("references/design-languages/instances/{id}.json")
}

impl Instance {
    pub(super) fn exact(&self, phrase: &str) -> bool {
        self.id.eq_ignore_ascii_case(phrase)
            || self.title.eq_ignore_ascii_case(phrase)
            || self
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(phrase))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_contract_rejects_missing_unsafe_or_unaccounted_material() {
        let catalog = super::super::Catalog::embedded().unwrap();
        let payload = catalog.payload(OWNER).unwrap();
        let original: Value = serde_json::from_slice(&payload.files[MANIFEST]).unwrap();
        let path = original["instances"][0]["record"].as_str().unwrap();
        let first: Value = serde_json::from_slice(&payload.files[path]).unwrap();
        for mutation in [
            "duplicate",
            "record-traversal",
            "identity",
            "traversal",
            "rights",
            "digest",
            "source",
            "external",
            "license",
            "date",
        ] {
            let mut value = original.clone();
            let mut record = first.clone();
            let mut changed = payload.clone();
            match mutation {
                "duplicate" => {
                    let first = value["instances"][0].clone();
                    value["instances"].as_array_mut().unwrap().push(first);
                }
                "record-traversal" => value["instances"][0]["record"] = json!("../SKILL.md"),
                "identity" => record["id"] = json!("another-instance"),
                "traversal" => record["card"] = json!("../SKILL.md"),
                "rights" => record["sources"][0]["rights"] = json!(""),
                "digest" => record["sources"][0]["sha256"] = json!("bogus"),
                "source" => record["sources"] = json!([]),
                "external" => record["original_example"] = json!(false),
                "license" => value["license"]["path"] = json!("missing.txt"),
                "date" => record["sources"][0]["checked_on"] = json!("yesterday"),
                _ => unreachable!(),
            }
            changed
                .files
                .insert(MANIFEST.into(), serde_json::to_vec(&value).unwrap());
            changed
                .files
                .insert(path.into(), serde_json::to_vec(&record).unwrap());
            assert!(Library::load(&changed).is_err(), "{mutation}");
        }
        let mut incomplete = payload.clone();
        incomplete.files.remove(first["example"].as_str().unwrap());
        assert!(Library::load(&incomplete).is_err());
        incomplete.files = payload.files.clone();
        incomplete.files.insert(
            first["example"].as_str().unwrap().into(),
            vec![b'x'; READ_LIMIT + 1],
        );
        assert!(Library::load(&incomplete).is_err());
    }

    #[test]
    fn a_poor_fit_is_not_a_positive_discovery_match() {
        let mut payload = super::super::Catalog::embedded()
            .unwrap()
            .payload(OWNER)
            .unwrap();
        let mut value: Value =
            serde_json::from_slice(&payload.files[&record_path("govuk-procedural")]).unwrap();
        value["avoid_when"] = json!("exclusive-poor-fit-token");
        payload.files.insert(
            record_path("govuk-procedural"),
            serde_json::to_vec(&value).unwrap(),
        );
        let library = Library::load(&payload).unwrap();
        assert!(
            library
                .matching(&["exclusive-poor-fit-token".into()])
                .is_empty()
        );
    }
}
