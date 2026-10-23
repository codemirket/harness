//! Revision-bound logical exports and explicitly selected retention, under the project operation lease.
use super::{journal::Journal, native::model_identity};
use crate::{memory::store::FactStore, util};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::path::Path;

fn digest(v: &Value) -> Result<String> {
    Ok(util::sha256(&serde_json::to_vec(v)?))
}
fn policy(v: &Value, now: u64) -> Result<(u64, u64)> {
    ensure!(
        v.as_object().is_some_and(|o| o.len() == 2),
        "invalid retention policy"
    );
    let expired = v["expired_before"]
        .as_u64()
        .context("expiry cutoff required")?;
    let completed = v["completed_before"]
        .as_u64()
        .context("completion cutoff required")?;
    ensure!(
        expired <= now && completed <= now,
        "retention cutoffs must be in the past"
    );
    Ok((expired, completed))
}
fn expired(v: &Value, cutoff: u64) -> bool {
    v["metadata"]["expires_at"]
        .as_u64()
        .is_some_and(|t| t <= cutoff)
}
fn completed(v: &Value, cutoff: u64) -> bool {
    v["phase"] == "completed"
        && v["state"]["last_clock"]
            .as_u64()
            .is_some_and(|t| t <= cutoff)
}

pub fn execute(
    directory: &Path,
    scope: &str,
    operation: &str,
    args: &Value,
    now: u64,
) -> Result<Value> {
    ensure!(
        matches!(operation, "data_export" | "data_plan" | "data_apply"),
        "unknown lifecycle operation"
    );
    let mut facts = FactStore::open(&directory.join("facts.sqlite"), scope, &model_identity())?;
    let mut journal = Journal::open(&directory.join("execution.sqlite"), scope)?;
    let fingerprint =
        digest(&json!({"scope":scope,"facts":facts.revision()?,"journal":journal.revision()?}))?;
    if operation == "data_apply" {
        let plan = &args["plan"];
        ensure!(
            plan["scope"] == scope && args["expected_sha256"] == digest(plan)?,
            "retention plan identity mismatch"
        );
        let (expiry, completion) = policy(&plan["policy"], now)?;
        let memories = plan["memories"]
            .as_array()
            .context("memory selection required")?;
        let workflows = plan["workflows"]
            .as_array()
            .context("workflow selection required")?;
        ensure!(
            memories.len() <= 100 && workflows.len() <= 100,
            "retention batch exceeds bound"
        );
        let mut seen = std::collections::BTreeSet::new();
        for item in memories {
            let id = item["id"].as_str().context("fact ID required")?;
            ensure!(seen.insert(id), "duplicate fact selection");
            if let Some(value) = facts.get_any(id)? {
                ensure!(
                    value["sha256"] == item["sha256"] && expired(&value, expiry),
                    "retention fact changed or remains active"
                );
            }
        }
        seen.clear();
        for item in workflows {
            let id = item["run_id"].as_str().context("run ID required")?;
            ensure!(seen.insert(id), "duplicate workflow selection");
            if let Some(value) = journal.get(id)? {
                ensure!(
                    value["checkpoint_id"] == item["checkpoint_id"]
                        && completed(&value, completion),
                    "retention workflow changed or remains unresolved"
                );
            }
        }
        // Separate databases intentionally have separate deletion boundaries.
        // Each removal commits atomically; exact retries skip already-removed IDs.
        for item in memories {
            facts.forget(
                item["id"].as_str().unwrap(),
                item["sha256"].as_str().context("fact digest required")?,
            )?;
        }
        for item in workflows {
            journal.execute(
                "workflow_forget",
                &json!({"run_id":item["run_id"],"expected_checkpoint_id":item["checkpoint_id"]}),
                now,
                Value::Null,
            )?;
        }
        return Ok(
            json!({"applied":true,"sha256":digest(plan)?,"memories":memories.len(),"workflows":workflows.len(),"traces_retained":true}),
        );
    }
    ensure!(
        operation == "data_export" || operation == "data_plan",
        "unknown lifecycle operation"
    );
    let exporting = operation == "data_export";
    let cursor = &args["cursor"];
    let mut phase = if cursor.is_null() {
        "memory"
    } else {
        cursor["phase"].as_str().context("cursor phase required")?
    }
    .to_owned();
    ensure!(
        ["memory", "workflow"].contains(&phase.as_str()),
        "invalid cursor phase"
    );
    if exporting && !cursor.is_null() {
        ensure!(
            cursor["sha256"] == fingerprint,
            "export changed; restart traversal"
        );
    }
    let mut after = cursor["after"].as_str().unwrap_or("").to_owned();
    ensure!(after.len() <= 64, "invalid lifecycle cursor");
    let (expiry, completion) = if exporting {
        (0, 0)
    } else {
        policy(&args["policy"], now)?
    };
    let page_bytes = args["page_bytes"].as_u64().unwrap_or(3 * 1024 * 1024) as usize;
    ensure!(
        (4096..=3 * 1024 * 1024).contains(&page_bytes),
        "invalid export page budget"
    );
    let mut items = Vec::new();
    let mut memories = Vec::new();
    let mut workflows = Vec::new();
    let mut scanned = 0;
    let mut size = 0;
    let mut finished = false;
    while scanned < 256 && items.len() < 20 && memories.len() < 100 && workflows.len() < 100 {
        let (id, item) = if phase == "memory" {
            let rows = facts.all_after(&after, 1)?;
            if rows.is_empty() {
                phase = "workflow".into();
                after.clear();
                continue;
            }
            let value = rows[0].clone();
            (
                value["id"].as_str().context("export fact ID")?.to_owned(),
                json!({"kind":"memory","value":value}),
            )
        } else {
            let Some((id, value)) = journal.next(&after)? else {
                finished = true;
                break;
            };
            (
                id.clone(),
                json!({"kind":"workflow","run_id":id,"value":value}),
            )
        };
        if exporting {
            let bytes = serde_json::to_vec(&item)?.len();
            ensure!(
                bytes <= page_bytes,
                "record exceeds transport page budget; use CLI export"
            );
            if size + bytes > page_bytes {
                break;
            }
            size += bytes;
            items.push(item);
        } else if phase == "memory" && expired(&item["value"], expiry) {
            memories.push(json!({"id":id,"sha256":item["value"]["sha256"]}));
        } else if phase == "workflow" && completed(&item["value"], completion) {
            workflows.push(json!({"run_id":id,"checkpoint_id":item["value"]["checkpoint_id"]}));
        }
        after = id;
        scanned += 1;
    }
    let mut next = if finished {
        Value::Null
    } else {
        json!({"phase":phase,"after":if after.is_empty(){None}else{Some(after)}})
    };
    if exporting {
        if !next.is_null() {
            next["sha256"] = json!(fingerprint);
        }
        Ok(
            json!({"format":"mirket-data-export-v1","scope":scope,"sha256":fingerprint,"items":items,"total":facts.count()?+journal.count()?,"next_cursor":next,"coverage":"Logical facts and latest native workflow states; use consistent database backup for restoration."}),
        )
    } else {
        let plan = json!({"scope":scope,"policy":args["policy"],"memories":memories,"workflows":workflows});
        Ok(
            json!({"sha256":digest(&plan)?,"plan":plan,"has_more":!next.is_null(),"next_cursor":next,"scanned":scanned,"retained":"Unresolved work, nonexpired facts, run traces and backups."}),
        )
    }
}
