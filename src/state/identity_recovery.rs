//! Explicit project identity recovery. A page is one SQLite transaction;
//! reviewed legacy device transitions require per-project evidence and backup.
use super::*;
use anyhow::ensure;
use serde_json::Map;

type ProjectIdentityRow = (String, String, String, i64, String, String, String);

fn rows(conn: &Connection) -> Result<Vec<ProjectIdentityRow>> {
    let mut stmt = conn.prepare("SELECT p.id,p.root,p.identity,p.created,COALESCE(s.scope,''),COALESCE(s.root,''),COALESCE(s.identity,'') FROM projects p LEFT JOIN project_scopes s ON s.project_id=p.id ORDER BY p.root")?;
    Ok(stmt
        .query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?)
}

fn entry(row: &ProjectIdentityRow) -> Value {
    let (id, root, saved, registered, scope, scoped_root, scoped_identity) = row;
    let mut item = json!({"id":id,"root":root,"saved_identity":saved,"registered":registered,"scope":scope,"status":"invalid","current_identity":null,"current_legacy_identity":null});
    if scoped_root != root
        || scoped_identity != saved
        || scope.len() != 64
        || scope.bytes().any(|b| !b.is_ascii_hexdigit())
    {
        item["status"] = json!("inconsistent_state");
        return item;
    }
    let path = Path::new(root);
    if !path.exists() {
        item["status"] = json!("missing_or_moved");
        return item;
    }
    let observation = (|| -> Result<(String, String)> {
        util::reject_symlink_components(path)?;
        ensure!(
            path.canonicalize()? == path,
            "project root is not canonical"
        );
        Ok((identity::current(path)?, identity::legacy(path)?))
    })();
    let Ok((current, legacy)) = observation else {
        item["status"] = json!("unsafe_path_or_unknown_identity");
        return item;
    };
    item["current_identity"] = json!(current);
    item["current_legacy_identity"] = json!(legacy);
    let birth = current
        .rsplit(':')
        .next()
        .and_then(|s| s.parse::<u128>().ok());
    let created_before_registration =
        *registered >= 0 && birth.is_some_and(|n| n <= (*registered as u128 + 1) * 1_000_000_000);
    if saved.starts_with("v2:") {
        item["status"] = json!(if saved.as_str() == current {
            "valid"
        } else {
            "replaced_or_changed"
        });
    } else if saved.as_str() == legacy {
        item["status"] = json!(if created_before_registration {
            "valid_legacy"
        } else {
            "replaced_or_ambiguous"
        });
    } else {
        let old: Vec<_> = saved.split(':').collect();
        let new: Vec<_> = legacy.split(':').collect();
        item["status"] = json!(if old.len() == 2
            && new.len() == 2
            && old[0] != new[0]
            && old[1] == new[1]
            && created_before_registration
        {
            "legacy_device_transition_review_required"
        } else {
            "replaced_or_ambiguous"
        });
    }
    item
}

fn preview_connection(conn: &Connection, offset: usize, limit: usize) -> Result<Value> {
    ensure!(
        (1..=10).contains(&limit) && offset <= 1_000_000,
        "project page size must be 1..=10 and offset at most 1000000"
    );
    let all = rows(conn)?;
    let mut seen = std::collections::BTreeMap::new();
    let mut items: Vec<_> = all.iter().map(entry).collect();
    for item in &mut items {
        if let Some(current) = item["current_identity"].as_str() {
            let count = seen.entry(current.to_owned()).or_insert(0usize);
            *count += 1;
        }
    }
    for item in &mut items {
        if item["current_identity"]
            .as_str()
            .is_some_and(|s| seen[s] > 1)
        {
            item["status"] = json!("identity_collision");
        }
    }
    let total = items.len();
    let page: Vec<_> = items.into_iter().skip(offset).take(limit).collect();
    let mut plan = Map::new();
    plan.insert("schema".into(), json!(1));
    plan.insert("offset".into(), json!(offset));
    plan.insert("limit".into(), json!(limit));
    plan.insert("total".into(), json!(total));
    plan.insert(
        "next_offset".into(),
        if offset + limit < total {
            json!(offset + limit)
        } else {
            Value::Null
        },
    );
    plan.insert("projects".into(), json!(page));
    let digest = util::sha256(&serde_json::to_vec(&plan)?);
    plan.insert("plan_sha256".into(), json!(digest));
    Ok(Value::Object(plan))
}

impl State {
    pub fn identity_preview(&self, offset: usize, limit: usize) -> Result<Value> {
        let conn = self.connection()?;
        preview_connection(&conn, offset, limit)
    }

    /// Applies exactly one reviewed page. The external backup and each project
    /// observation are rechecked under the exclusive maintenance lease.
    pub fn identity_apply(
        &self,
        plan_path: &Path,
        backup_path: &Path,
        backup_sha256: &str,
        attestation_path: Option<&Path>,
    ) -> Result<Value> {
        let saved: Value = serde_json::from_slice(&util::read_bounded(plan_path, 64 * 1024)?)?;
        let offset = saved["offset"].as_u64().context("plan offset missing")? as usize;
        let limit = saved["limit"].as_u64().context("plan limit missing")? as usize;
        ensure!(
            saved["schema"] == 1 && saved["plan_sha256"].is_string(),
            "invalid project identity plan"
        );
        let mut unsigned = saved.clone();
        let digest = unsigned
            .as_object_mut()
            .context("invalid project identity plan")?
            .remove("plan_sha256")
            .context("plan digest missing")?;
        ensure!(
            digest == json!(util::sha256(&serde_json::to_vec(&unsigned)?)),
            "project identity plan digest mismatch"
        );
        let attestation_bytes = attestation_path
            .map(|path| util::read_bounded(path, 32 * 1024))
            .transpose()?;
        let attestations: Value = attestation_bytes
            .as_ref()
            .map(|bytes| serde_json::from_slice(bytes))
            .transpose()?
            .unwrap_or_else(|| json!({"schema":1,"projects":[]}));
        ensure!(
            attestations["schema"] == 1 && attestations["projects"].as_array().is_some(),
            "invalid project identity attestation"
        );
        let reviewed_transitions: Vec<_> = saved["projects"]
            .as_array()
            .context("invalid reviewed project page")?
            .iter()
            .filter(|p| p["status"] == "legacy_device_transition_review_required")
            .collect();
        let statements = attestations["projects"].as_array().unwrap();
        ensure!(
            statements.len() == reviewed_transitions.len(),
            "one identity attestation is required for each reviewed device transition"
        );
        for project in reviewed_transitions {
            let matches: Vec<_> = statements
                .iter()
                .filter(|statement| {
                    statement["id"] == project["id"]
                        && statement["root"] == project["root"]
                        && statement["saved_identity"] == project["saved_identity"]
                        && statement["current_identity"] == project["current_identity"]
                        && statement["evidence"]
                            .as_str()
                            .is_some_and(|s| (20..=1000).contains(&s.len()) && !s.contains('\0'))
                })
                .collect();
            ensure!(
                matches.len() == 1,
                "missing or duplicate identity attestation for {}",
                project["id"]
            );
        }
        let attestation_sha256 = attestation_bytes.as_ref().map(|bytes| util::sha256(bytes));
        let _lease = crate::maintenance::exclusive(&self.paths)?;
        ensure!(
            !self.paths.root.join("restore-pending.json").exists(),
            "durable restore is incomplete"
        );
        let backup = crate::backup::inspect_project_identity_preimages(
            &self.paths,
            backup_path,
            backup_sha256,
            &saved,
        )?;
        ensure!(
            backup["valid"] == true
                && backup["manifest"]["home"] == json!(self.paths.home)
                && backup["manifest"]["runtime_recipe"] == json!(crate::packages::agent::recipe()),
            "backup home or runtime recipe cannot restore this Mirket state"
        );
        let mut conn = self.unleased_connection()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let fresh = preview_connection(&tx, offset, limit)?;
        if saved != fresh {
            // The page commits in one transaction. An exact retry is harmless
            // only when every changed row has its audit record and unchanged
            // rows still match the reviewed preview.
            ensure!(
                saved["schema"] == fresh["schema"]
                    && saved["offset"] == fresh["offset"]
                    && saved["limit"] == fresh["limit"]
                    && saved["total"] == fresh["total"],
                "project identity preview is stale; preview again"
            );
            let before = saved["projects"]
                .as_array()
                .context("invalid reviewed plan")?;
            let after = fresh["projects"]
                .as_array()
                .context("invalid current page")?;
            ensure!(
                before.len() == after.len(),
                "project identity preview is stale; preview again"
            );
            let mut prior = Vec::new();
            for (old, now) in before.iter().zip(after) {
                if old == now {
                    continue;
                }
                let id = old["id"].as_str().context("invalid reviewed project ID")?;
                ensure!(
                    ["valid_legacy", "legacy_device_transition_review_required"]
                        .contains(&old["status"].as_str().unwrap_or(""))
                        && old["id"] == now["id"]
                        && old["root"] == now["root"]
                        && now["status"] == "valid"
                        && old["current_identity"] == now["saved_identity"]
                        && old["current_identity"] == now["current_identity"],
                    "project identity preview is stale; preview again"
                );
                let logged: String = tx
                    .query_row(
                        "SELECT response FROM operations WHERE scope=?1 AND key=?2",
                        params![
                            format!("identity-recovery:{id}"),
                            saved["plan_sha256"]
                                .as_str()
                                .context("invalid plan digest")?
                        ],
                        |r| r.get(0),
                    )
                    .optional()?
                    .context("project identity preview is stale; no matching recovery record")?;
                let record: Value = serde_json::from_str(&logged)?;
                ensure!(
                    record["old_identity"] == old["saved_identity"]
                        && record["new_identity"] == now["saved_identity"]
                        && record["backup_sha256"] == backup_sha256
                        && record["attestation_sha256"] == json!(attestation_sha256),
                    "recovery record does not match reviewed plan"
                );
                prior.push(record);
            }
            ensure!(
                !prior.is_empty(),
                "project identity preview is stale; preview again"
            );
            return Ok(
                json!({"ok":true,"replayed":true,"changed":prior,"atomic_boundary":"one reviewed project page; SQLite transaction"}),
            );
        }
        let mut changed = Vec::new();
        for item in fresh["projects"]
            .as_array()
            .context("plan projects missing")?
        {
            let status = item["status"].as_str().context("project status missing")?;
            if status == "valid" {
                continue;
            }
            ensure!(
                status == "valid_legacy" || status == "legacy_device_transition_review_required",
                "project {} requires additional identity evidence: {status}",
                item["id"]
            );
            let id = item["id"].as_str().context("project ID missing")?;
            let current = item["current_identity"]
                .as_str()
                .context("current identity missing")?;
            let previous = item["saved_identity"]
                .as_str()
                .context("saved identity missing")?;
            let review_evidence = statements
                .iter()
                .find(|statement| statement["id"] == item["id"])
                .and_then(|statement| statement["evidence"].as_str());
            ensure!(
                identity::current(Path::new(item["root"].as_str().context("root missing")?))?
                    == current,
                "project identity changed during recovery"
            );
            ensure!(
                tx.execute(
                    "UPDATE projects SET identity=?1 WHERE id=?2 AND identity=?3",
                    params![current, id, previous]
                )? == 1,
                "project identity changed during recovery"
            );
            ensure!(
                tx.execute(
                    "UPDATE project_scopes SET identity=?1 WHERE project_id=?2 AND identity=?3",
                    params![current, id, previous]
                )? == 1,
                "project namespace identity changed during recovery"
            );
            let response = json!({"id":id,"root":item["root"],"old_identity":previous,"new_identity":current,"status":status,"plan_sha256":fresh["plan_sha256"],"backup_sha256":backup_sha256,"attestation_sha256":attestation_sha256,"review_evidence":review_evidence});
            tx.execute(
                "INSERT INTO operations(scope,key,fingerprint,response) VALUES(?1,?2,?3,?4)",
                params![
                    format!("identity-recovery:{id}"),
                    fresh["plan_sha256"].as_str().unwrap(),
                    util::sha256(&serde_json::to_vec(&response)?),
                    response.to_string()
                ],
            )?;
            changed.push(response);
        }
        tx.commit()?;
        Ok(
            json!({"ok":true,"changed":changed,"atomic_boundary":"one reviewed project page; SQLite transaction","backup_sha256":backup_sha256}),
        )
    }
}
