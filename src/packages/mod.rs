//! Reviewed runtimes, installed and administered through the shared setup owner.
pub mod agent;
pub(crate) mod audit;
mod machine;
mod oci;
mod services;

use crate::{package_mcp::LaunchSpec, paths::Paths};
use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

const TOOLS: [&str; 10] = [
    "style-dictionary",
    "satori",
    "playwright",
    "openpencil",
    "typst",
    "sharp",
    "svgo",
    "odiff",
    "axe-core",
    "d2",
];

pub struct Registry {
    design: Value,
    agent: Value,
}

impl Registry {
    pub(crate) fn request_timeout(&self, id: &str) -> Result<std::time::Duration> {
        self.get(id)?;
        if id == "agent-runtime" {
            return Ok(std::time::Duration::from_secs(150));
        }
        // Let the bridge return its own bounded error and finish cleanup first.
        Ok(services::tool_budgets()?.0 + std::time::Duration::from_secs(30))
    }
    pub fn embedded() -> Result<Self> {
        let design = services::definition()?;
        ensure!(
            design["schema_version"] == 1 && design["id"] == "design",
            "invalid design stack registry"
        );
        let tools = design["tools"]
            .as_array()
            .context("design registry requires its canonical tools")?;
        let ids = tools
            .iter()
            .map(|tool| tool["id"].as_str().context("design tool ID missing"))
            .collect::<Result<BTreeSet<_>>>()?;
        ensure!(
            tools.len() == TOOLS.len() && ids == TOOLS.into_iter().collect(),
            "design registry differs from the canonical tool stack"
        );
        Ok(Self {
            design,
            agent: agent::definition()?,
        })
    }

    pub fn get(&self, id: &str) -> Result<&Value> {
        if id == "agent-runtime" {
            return Ok(&self.agent);
        }
        ensure!(
            id == "design",
            "unknown package: {id}; use mirket package list"
        );
        Ok(&self.design)
    }

    pub fn list(&self) -> Value {
        json!({"packages":[self.design,self.agent],"execution":"Reviewed packages on the registered private Podman engine. Design shares /workspace; agent-runtime receives only private state and no network.","installation":"Only explicit setup builds or downloads runtimes. Serving never installs software or starts the VM."})
    }

    pub fn validate_selection(&self, ids: &[String]) -> Result<()> {
        ensure!(
            ids.len() <= 2 && ids.iter().collect::<BTreeSet<_>>().len() == ids.len(),
            "select each runtime once"
        );
        for id in ids {
            self.get(id)?;
        }
        Ok(())
    }
}

pub(crate) fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

pub(crate) fn prepare_with_registry(
    paths: &Paths,
    selected: &[String],
    registry: &Registry,
) -> Result<()> {
    registry.validate_selection(selected)?;
    for id in selected {
        match id.as_str() {
            "design" => services::prepare(paths, &[])?,
            "agent-runtime" => agent::prepare(paths)?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

pub fn verify_machine_image(path: &Path) -> Result<bool> {
    machine::verify(path)
}
pub fn machine_image(paths: &Paths) -> Result<PathBuf> {
    machine::download(paths)
}

/// Installation checks do not require selected long-lived services to be running.
pub fn doctor(paths: &Paths, selected: &[String]) -> Result<Value> {
    inspect(paths, selected, false)
}

/// Readiness includes live observations, without starting services or a VM.
pub fn status(paths: &Paths, selected: &[String]) -> Result<Value> {
    inspect(paths, selected, true)
}

fn package_check(
    id: &str,
    installation: Result<Value>,
    runtime: Option<Result<Value>>,
    readiness: bool,
) -> Value {
    match installation {
        Err(error) => json!({"id":id,"configured":true,"installed":null,"installation_ok":false,
            "running":null,"ready":false,"ok":false,"error":format!("{error:#}")}),
        Ok(installation) => {
            let runtime = runtime.map(|result| {
                result.unwrap_or_else(|error| json!({"ready":false,"error":format!("{error:#}")}))
            });
            let ready = if id == "agent-runtime" {
                Some(true)
            } else {
                runtime.as_ref().and_then(|value| value["ready"].as_bool())
            };
            let running = runtime
                .as_ref()
                .and_then(|value| value["services"].as_array())
                .map(|entries| {
                    entries
                        .iter()
                        .filter(|entry| entry["enabled"] == true)
                        .all(|entry| entry["running"] == true)
                });
            json!({"id":id,"configured":true,"installed":true,"installation_ok":true,
                "running":running,"ready":ready,"ok":!readiness || ready == Some(true),
                "installation":installation,"runtime":runtime,
                "lifecycle":if id == "agent-runtime" {"on_demand"} else {"services"}})
        }
    }
}

fn inspect(paths: &Paths, selected: &[String], readiness: bool) -> Result<Value> {
    Registry::embedded()?.validate_selection(selected)?;
    let checks: Vec<_> = selected
        .iter()
        .map(|id| {
            let installation = if id == "agent-runtime" {
                agent::verify(paths)
            } else {
                services::verify(paths)
            };
            let runtime = (readiness && id == "design" && installation.is_ok())
                .then(|| services::status(paths));
            package_check(id, installation, runtime, readiness)
        })
        .collect();
    Ok(
        json!({"scope":if readiness {"readiness"} else {"installation"},
        "ok":checks.iter().all(|check| check["ok"] == true),
        "installation_ok":checks.iter().all(|check| check["installation_ok"] == true),
        "ready":if readiness {Some(checks.iter().all(|check| check["ready"] == true))} else {None},
        "checks":checks,"verification":"Null means unobserved. On-demand runtimes need no persistent process. Readiness is not invocation or artifact evidence."}),
    )
}

fn selected(paths: &Paths, id: &str) -> Result<()> {
    Registry::embedded()?.get(id)?;
    let options = crate::install::saved_options(paths)?.context("run mirket setup first")?;
    if !options.packages.iter().any(|package| package == id) {
        bail!("package {id} is not selected; explicitly include it in mirket package install");
    }
    Ok(())
}

pub async fn launch(paths: &Paths, id: &str) -> Result<LaunchSpec> {
    selected(paths, id)?;
    ensure!(
        id == "design",
        "agent-runtime is internal; use memory and agent commands through Mirket"
    );
    let paths = paths.clone();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    tokio::time::timeout_at(
        deadline.into(),
        tokio::task::spawn_blocking(move || services::launch(&paths, deadline)),
    )
    .await
    .context("design launch validation deadline exceeded")?
    .context("design launch validation worker failed")?
}

/// Worker-only process scope; host tools retain their explicitly owned shared workspace.
pub async fn launch_scoped(paths: &Paths, scope: &std::path::Path) -> Result<LaunchSpec> {
    let root = paths.root.join("execution/podman/staging/design/workspace");
    let relative = scope
        .strip_prefix(&root)
        .context("design scope outside workspace")?;
    crate::util::safe_relative(relative)?;
    crate::util::ensure_directory(scope)?;
    let mut launch = launch(paths, "design").await?;
    let index = launch
        .args
        .iter()
        .position(|a| a == "--workdir")
        .context("design launch has no workdir")?;
    let container = format!("/workspace/{}", relative.to_string_lossy());
    launch.args[index + 1] = container.clone();
    launch.args.splice(
        index..index,
        [
            "--env".into(),
            format!("MIRKET_DESIGN_WORKSPACE={container}"),
            "--env".into(),
            "MIRKET_SCOPED_WORKER=1".into(),
        ],
    );
    launch.cwd = scope.to_owned();
    launch.allowed_tools = Some(
        [
            "openpencil_inspect",
            "openpencil_document",
            "openpencil_edit",
            "openpencil_export",
            "tokens_compile",
            "static_render",
            "browser_screenshot",
            "vector_render",
            "svg_optimize",
            "raster_transform",
            "d2_render",
            "typst_compile",
            "odiff_compare",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
    );
    Ok(launch)
}

pub fn service_start(paths: &Paths, id: &str, groups: &[String]) -> Result<Value> {
    selected(paths, id)?;
    if id == "agent-runtime" {
        ensure!(
            groups.is_empty(),
            "agent-runtime has no design service groups"
        );
        return agent::verify(paths);
    }
    services::start(paths, groups)
}

/// Stage through the installer-owned preparation path without replacing client
/// configuration, saved choices or the installed binary. Selection is explicit.
pub fn prepare_selected(paths: &Paths, id: &str, groups: &[String]) -> Result<Value> {
    selected(paths, id)?;
    let selected = vec![id.to_owned()];
    if id == "design" {
        services::prepare(paths, groups)?;
    } else {
        ensure!(
            groups.is_empty(),
            "agent-runtime has no design service groups"
        );
        agent::prepare(paths)?;
    }
    doctor(paths, &selected)
}

pub fn service_stop(paths: &Paths, id: &str, purge_data: bool) -> Result<Value> {
    Registry::embedded()?.get(id)?;
    if id == "agent-runtime" {
        return agent::stop(paths, purge_data);
    }
    services::stop(paths, purge_data)
}

pub(crate) fn activation_status(paths: &Paths) -> Result<Value> {
    selected(paths, "design")?;
    services::activation_status(paths)
}

/// Reviewed candidate image for explicit isolated acceptance fixtures; never activates services.
#[doc(hidden)]
pub fn design_candidate_image(paths: &Paths) -> Result<String> {
    services::candidate_image(paths, "design-tools")
}

pub fn test_design_tools(paths: &Paths, project: &Path) -> Result<Value> {
    services::test_tools(paths, project)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn readiness_never_hides_a_stopped_or_unobservable_runtime() {
        let installed = || Ok(json!({"receipt":"valid"}));
        let stopped = package_check(
            "design",
            installed(),
            Some(Ok(
                json!({"ready":false,"services":[{"enabled":true,"running":false}]}),
            )),
            true,
        );
        assert_eq!(stopped["installation_ok"], true);
        assert_eq!(stopped["ok"], false);
        assert_eq!(stopped["running"], false);
        let unavailable = package_check(
            "design",
            installed(),
            Some(Err(anyhow::anyhow!("offline"))),
            true,
        );
        assert_eq!(unavailable["ok"], false);
        assert!(unavailable["running"].is_null());
        let passive = package_check("design", installed(), None, false);
        assert_eq!(passive["ok"], true);
        assert!(passive["ready"].is_null());
        let healthy = package_check("design", installed(), Some(Ok(json!({"ready":true}))), true);
        assert_eq!(healthy["ok"], true);
        let demand = package_check("agent-runtime", installed(), None, true);
        assert_eq!(demand["ready"], true);
        assert!(demand["running"].is_null());
    }
    #[test]
    fn preparation_requires_existing_selection_before_any_effect() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
        assert!(prepare_selected(&paths, "agent-runtime", &[]).is_err());
        assert!(!paths.root.exists());
    }
    #[test]
    fn registry_exposes_exactly_the_canonical_stack() {
        let registry = Registry::embedded().unwrap();
        assert_eq!(registry.list()["packages"].as_array().unwrap().len(), 2);
        assert_eq!(
            registry.get("design").unwrap()["tools"]
                .as_array()
                .unwrap()
                .len(),
            TOOLS.len()
        );
        assert!(registry.get("invented-package").is_err());
        assert!(registry.validate_selection(&[]).is_ok());
        assert!(registry.validate_selection(&["design".into()]).is_ok());
        assert!(
            registry
                .validate_selection(&["design".into(), "design".into()])
                .is_err()
        );
        assert!(
            registry
                .validate_selection(&["invented-package".into()])
                .is_err()
        );
    }
    #[tokio::test]
    async fn unselected_stack_cannot_start_or_serve() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
        assert!(launch(&paths, "design").await.is_err());
        assert!(service_start(&paths, "design", &[]).is_err());
        assert!(!paths.root.exists());
    }
}
