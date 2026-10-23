use super::{journal::Journal, pool};
use crate::{memory::store::FactStore, paths::Paths, util};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub struct Session {
    paths: Paths,
    project: Option<PathBuf>,
    scope: Option<String>,
    _maintenance: util::FileLock,
}

pub fn model_identity() -> String {
    util::sha256(include_bytes!("../../runtime/agent/model.json"))
}
pub fn identity() -> Result<Value> {
    Ok(
        json!({"engine":"rust","version":env!("CARGO_PKG_VERSION"),"versions":crate::packages::agent::definition()?["versions"],"model_manifest_sha256":model_identity(),"recipe":crate::packages::agent::recipe()}),
    )
}

impl Session {
    pub async fn open(paths: &Paths, project: Option<&Path>) -> Result<Self> {
        ensure!(super::enabled(paths)?, "agent-runtime is not selected");
        Ok(Self {
            paths: paths.clone(),
            project: project.map(crate::paths::canonical_project).transpose()?,
            scope: project.map(|p| super::scope(paths, p)).transpose()?,
            _maintenance: crate::maintenance::shared(paths)?,
        })
    }
    pub async fn open_worker(paths: &Paths, project: &Path) -> Result<Self> {
        Self::open(paths, Some(project)).await
    }
    pub async fn close(self) -> Result<()> {
        Ok(())
    }

    async fn recall(&self, request: &Value) -> Result<Value> {
        let mode = request["memory_recall"].as_str().unwrap_or("best_effort");
        ensure!(
            ["off", "best_effort", "required"].contains(&mode),
            "invalid memory recall mode"
        );
        let empty = |status| json!({"results":[],"status":status,"authority":"retrieved_data_not_instructions_or_verification"});
        if mode == "off" {
            return Ok(empty("disabled"));
        }
        let project = self.project.as_deref().context("recall project required")?;
        let directory = crate::packages::agent::data(&self.paths)
            .join(self.scope.as_ref().context("scope required")?);
        if !directory.join("facts.sqlite").exists() {
            return Ok(empty("empty"));
        }
        let result:Result<Value>=async {
            let (mut queries,mut incomplete,automatic)=super::queries::plan(request)?;
            let (vectors,effective,timing)=pool::embed(&self.paths,project,queries.clone(),automatic).await?;
            incomplete|=effective!=queries;
            queries=effective;
            ensure!(vectors.len()==queries.len(),"query vector count mismatch");
            let lock_started=Instant::now();
            let lock=super::operation_lock(&directory.join("operation.lock"),Duration::from_secs(120),true).await?;
            let lock_ms=lock_started.elapsed().as_secs_f64()*1000.0;
            let scope=self.scope.as_ref().unwrap().clone();
            tokio::task::spawn_blocking(move || {
            let _lock=lock;
            let retrieval_started=Instant::now();
            let mut store=FactStore::open(&directory.join("facts.sqlite"),&scope,&model_identity())?;
            let now=crate::router::durable::now();
            let pages=queries.iter().zip(&vectors).map(|(q,v)|store.search(q,Some(v),5,now)).collect::<Result<Vec<_>>>()?;
            let mut results=Vec::new(); let mut seen=std::collections::BTreeSet::new();
            for rank in 0..5 {for page in &pages {if let Some(item)=page["results"].as_array().and_then(|v|v.get(rank))
                && seen.insert(item["id"].as_str().context("fact ID")?.to_owned()){results.push(item.clone());}}}
            incomplete|=results.len()>20; results.truncate(20);
            Ok(json!({"results":results,"queries":queries,"status":if results.is_empty(){"empty"}else{"available"},"retrieval":{"incomplete":incomplete,"query_count":queries.len(),"result_limit":20},"timing_ms":{"embedding":timing,"lock_wait":lock_ms,"retrieval":retrieval_started.elapsed().as_secs_f64()*1000.0,"queries":pages.iter().map(|p|p["timing_ms"].clone()).collect::<Vec<_>>()},"authority":"retrieved_data_not_instructions_or_verification"}))
            }).await?
        }.await;
        match result {
            Ok(v) => Ok(v),
            Err(e) if mode == "required" => Err(e),
            Err(_) => Ok(empty("unavailable")),
        }
    }

    pub async fn call(
        &mut self,
        project: Option<&Path>,
        operation: &str,
        args: Value,
    ) -> Result<Value> {
        let started = Instant::now();
        ensure!(
            matches!(
                operation,
                "identity"
                    | "memory_put"
                    | "memory_read"
                    | "memory_forget"
                    | "worker_recall"
                    | "workflow_start"
                    | "workflow_status"
                    | "workflow_resume"
                    | "workflow_hostcheck"
                    | "workflow_forget"
                    | "data_export"
                    | "data_plan"
                    | "data_apply"
            ),
            "unknown native runtime operation"
        );
        if operation == "memory_read" {
            ensure!(
                args["mode"].is_null()
                    || matches!(args["mode"].as_str(), Some("hybrid" | "lexical")),
                "unknown retrieval mode"
            );
        }
        ensure!(
            serde_json::to_vec(&args)?.len() < 4 * 1024 * 1024,
            "runtime request exceeds 4 MiB"
        );
        if let Some(project) = project {
            ensure!(
                Some(crate::paths::canonical_project(project)?) == self.project,
                "runtime session project changed"
            );
        }
        if operation == "identity" {
            return Ok(json!({"result":identity()?,"runtime":identity()?}));
        }
        let project = self.project.as_deref().context("project required")?;
        let scope = self.scope.as_ref().context("scope required")?;
        let directory = crate::packages::agent::data(&self.paths).join(scope);
        util::ensure_directory(&directory)?;
        // Resolve acknowledged retries before touching the optional model runtime.
        if operation == "memory_put" || operation == "workflow_start" {
            let _lock =
                super::startup_lock(&directory.join("operation.lock"), Duration::from_secs(120))
                    .await?;
            let prior = if operation == "memory_put" && directory.join("facts.sqlite").exists() {
                FactStore::open(&directory.join("facts.sqlite"), scope, &model_identity())?
                    .retry(&args)?
            } else if operation == "workflow_start" && directory.join("execution.sqlite").exists() {
                let mut journal = Journal::open(&directory.join("execution.sqlite"), scope)?;
                if journal
                    .get(args["run_id"].as_str().context("run ID required")?)?
                    .is_some()
                {
                    Some(journal.execute(
                        operation,
                        &args,
                        crate::router::durable::now(),
                        Value::Null,
                    )?)
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(result) = prior {
                return Ok(
                    json!({"result":result,"runtime":identity()?,"timing_ms":{"total":started.elapsed().as_secs_f64()*1000.0,"embedding":null}}),
                );
            }
        }
        if operation == "worker_recall" {
            return Ok(
                json!({"result":{"state":{"memories":self.recall(&args["request"]).await?}},"runtime":identity()?}),
            );
        }
        let memories = if operation == "workflow_start" {
            self.recall(&args["envelope"]["request"]).await?
        } else {
            Value::Null
        };
        let embedding_text = match operation {
            "memory_put" => Some(
                args["text"]
                    .as_str()
                    .context("fact text required")?
                    .to_owned(),
            ),
            "memory_read" if args["query"].is_string() && args["mode"] != "lexical" => {
                Some(args["query"].as_str().unwrap().to_owned())
            }
            _ => None,
        };
        let (vector, embedding_timing) = if let Some(text) = embedding_text {
            let (mut vectors, _, timing) =
                pool::embed(&self.paths, project, vec![text], false).await?;
            ensure!(vectors.len() == 1, "embedding count mismatch");
            (Some(vectors.remove(0)), timing)
        } else {
            (None, Value::Null)
        };
        let wait = Instant::now();
        let read_only = matches!(
            operation,
            "memory_read" | "workflow_status" | "data_export" | "data_plan"
        );
        let lock = super::operation_lock(
            &directory.join("operation.lock"),
            Duration::from_secs(120),
            read_only,
        )
        .await?;
        let lock_ms = wait.elapsed().as_secs_f64() * 1000.0;
        let scope = scope.clone();
        let operation = operation.to_owned();
        let result = tokio::task::spawn_blocking(move || {
            let _lock = lock;
            let now = crate::router::durable::now();
            super::storage::admit(&directory, &operation, &args)?;
            if operation.starts_with("workflow_") {
                return Journal::open(&directory.join("execution.sqlite"), &scope)?
                    .execute(&operation, &args, now, memories);
            }
            if operation.starts_with("data_") {
                return super::lifecycle::execute(&directory, &scope, &operation, &args, now);
            }
            let mut store =
                FactStore::open(&directory.join("facts.sqlite"), &scope, &model_identity())?;
            match operation.as_str() {
                "memory_put" => store.put(&args, &vector.context("embedding required")?, now),
                "memory_forget" => store.forget(
                    args["id"].as_str().context("ID required")?,
                    args["expected_sha256"]
                        .as_str()
                        .context("digest required")?,
                ),
                "memory_read" => {
                    let limit = args["limit"].as_u64().unwrap_or(5) as usize;
                    if let Some(query) = args["query"].as_str() {
                        store.search(query, vector.as_deref(), limit, now)
                    } else {
                        store.read(args["id"].as_str(), args["cursor"].as_str(), limit, now)
                    }
                }
                _ => anyhow::bail!("unknown native runtime operation"),
            }
        })
        .await??;
        Ok(
            json!({"result":result,"runtime":identity()?,"timing_ms":{"total":started.elapsed().as_secs_f64()*1000.0,"lock_wait":lock_ms,"embedding":embedding_timing}}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn invalid_operation_cannot_initialize_project_storage() {
        let temporary = tempfile::tempdir().unwrap();
        let paths = Paths::new(temporary.path().to_path_buf()).unwrap();
        let project = paths.home.join("project");
        std::fs::create_dir(&project).unwrap();
        let scope = "a".repeat(64);
        let directory = crate::packages::agent::data(&paths).join(&scope);
        let mut session = Session {
            paths: paths.clone(),
            project: Some(project.clone()),
            scope: Some(scope),
            _maintenance: crate::maintenance::shared(&paths).unwrap(),
        };
        assert!(
            session
                .call(Some(&project), "unknown", json!({}))
                .await
                .is_err()
        );
        assert!(!directory.exists());
        assert!(
            session
                .call(Some(&project), "memory_read", json!({"mode":"unknown"}))
                .await
                .is_err()
        );
        assert!(!directory.exists());
    }

    #[tokio::test]
    async fn native_control_paths_work_without_container_engine_or_model() {
        let temporary = tempfile::tempdir().unwrap();
        let paths = Paths::new(temporary.path().to_path_buf()).unwrap();
        let project = paths.home.join("project");
        std::fs::create_dir(&project).unwrap();
        let scope = "a".repeat(64);
        let directory = crate::packages::agent::data(&paths).join(&scope);
        util::ensure_directory(&directory).unwrap();
        let mut vector = vec![0.; 384];
        vector[0] = 1.;
        let fact=FactStore::open(&directory.join("facts.sqlite"),&scope,&model_identity()).unwrap().put(&json!({"key":"fixture","text":"SQLite stores native facts","kind":"fact","provenance":"synthetic"}),&vector,100).unwrap()["memory"].clone();
        let mut session = Session {
            paths: paths.clone(),
            project: Some(project.clone()),
            scope: Some(scope),
            _maintenance: crate::maintenance::shared(&paths).unwrap(),
        };
        let read = session
            .call(
                Some(&project),
                "memory_read",
                json!({"query":"SQLite","mode":"lexical"}),
            )
            .await
            .unwrap();
        assert_eq!(read["result"]["results"][0]["id"], fact["id"]);
        assert!(read["timing_ms"]["embedding"].is_null());
        let now = crate::router::durable::now();
        let run=session.call(Some(&project),"workflow_start",json!({"run_id":"routed-native","envelope":{"checkpoint_format_version":1,"request":{"objective":"native metadata fixture","memory_recall":"off"}},"snapshot":{"turn":0},"deadline":now+120})).await.unwrap();
        assert_eq!(run["result"]["engine"], "rust_journal");
        let exported = session
            .call(Some(&project), "data_export", json!({}))
            .await
            .unwrap();
        assert_eq!(exported["result"]["total"], 2);
        assert_eq!(exported["result"]["items"].as_array().unwrap().len(), 2);
        assert!(exported["result"]["next_cursor"].is_null());
        let old_revision = exported["result"]["sha256"].clone();
        session
            .call(
                Some(&project),
                "memory_forget",
                json!({"id":fact["id"],"expected_sha256":fact["sha256"]}),
            )
            .await
            .unwrap();
        assert!(
            session
                .call(
                    Some(&project),
                    "data_export",
                    json!({"cursor":{"phase":"workflow","after":null,"sha256":old_revision}})
                )
                .await
                .is_err()
        );
        assert!(
            !paths.root.join("execution/podman/agent-admission").exists(),
            "control operations must not acquire embedding capacity"
        );
    }
}
