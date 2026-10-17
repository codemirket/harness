//! The canonical design stack, installed and administered through one owner.
mod machine;
mod services;

use crate::{package_mcp::LaunchSpec, paths::Paths};
use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

const TOOLS: [&str; 11] = [
    "style-dictionary",
    "satori",
    "browserless",
    "penpot",
    "kroki",
    "hyperframes",
    "typst",
    "imgproxy",
    "svgo",
    "comfyui",
    "odiff",
];

pub struct Registry {
    design: Value,
}

impl Registry {
    pub fn embedded() -> Result<Self> {
        let design = services::definition()?;
        ensure!(
            design["schema_version"] == 1 && design["id"] == "design",
            "invalid design stack registry"
        );
        let tools = design["tools"]
            .as_array()
            .context("design registry requires its eleven tools")?;
        let ids = tools
            .iter()
            .map(|tool| tool["id"].as_str().context("design tool ID missing"))
            .collect::<Result<BTreeSet<_>>>()?;
        ensure!(
            tools.len() == TOOLS.len() && ids == TOOLS.into_iter().collect(),
            "design registry differs from the canonical eleven-tool stack"
        );
        Ok(Self { design })
    }

    pub fn get(&self, id: &str) -> Result<&Value> {
        ensure!(
            id == "design",
            "unknown package: {id}; use mirket package list"
        );
        Ok(&self.design)
    }

    pub fn list(&self) -> Value {
        json!({"packages":[self.design],"execution":"Explicitly installed services on the registered private Podman engine; all tools exchange paths in one shared /workspace.","installation":"Only explicit setup builds or downloads runtimes. Serving never installs software or starts the VM."})
    }

    pub fn validate_selection(&self, ids: &[String]) -> Result<()> {
        ensure!(ids.len() <= 1, "select the design stack once");
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
    if !selected.is_empty() {
        services::prepare(paths)?;
    }
    Ok(())
}

pub fn verify_machine_image(path: &Path) -> Result<bool> {
    machine::verify(path)
}
pub fn machine_image(paths: &Paths) -> Result<PathBuf> {
    machine::download(paths)
}

pub fn doctor(paths: &Paths, selected: &[String]) -> Result<Value> {
    Registry::embedded()?.validate_selection(selected)?;
    let checks = if selected.is_empty() {
        Vec::new()
    } else {
        vec![match services::verify(paths) {
            Ok(receipt) => {
                let runtime = match services::status(paths) {
                    Ok(status) => status,
                    Err(error) => json!({"ok":false,"error":format!("{error:#}")}),
                };
                json!({"id":"design","ok":true,"installation":receipt,"runtime":runtime,"workspace":services::workspace(paths)?})
            }
            Err(error) => json!({"id":"design","ok":false,"error":format!("{error:#}")}),
        }]
    };
    Ok(
        json!({"ok":checks.iter().all(|check| check["ok"] == true),"checks":checks,"verification":"Local installation identity and live service health are separate. Use package probe and saved output inspection for invocation evidence."}),
    )
}

fn selected(paths: &Paths, id: &str) -> Result<()> {
    Registry::embedded()?.get(id)?;
    let options = crate::install::saved_options(paths)?.context("run mirket setup first")?;
    if !options.packages.iter().any(|package| package == id) {
        bail!("package {id} is not selected; use mirket package install design");
    }
    Ok(())
}

pub fn launch(paths: &Paths, id: &str) -> Result<LaunchSpec> {
    selected(paths, id)?;
    services::launch(paths)
}

pub fn service_start(paths: &Paths, id: &str) -> Result<Value> {
    selected(paths, id)?;
    services::start(paths)
}

pub fn service_stop(paths: &Paths, id: &str, purge_data: bool) -> Result<Value> {
    Registry::embedded()?.get(id)?;
    services::stop(paths, purge_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_exposes_exactly_the_canonical_stack() {
        let registry = Registry::embedded().unwrap();
        assert_eq!(registry.list()["packages"].as_array().unwrap().len(), 1);
        assert_eq!(
            registry.get("design").unwrap()["tools"]
                .as_array()
                .unwrap()
                .len(),
            11
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
    #[test]
    fn unselected_stack_cannot_start_or_serve() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
        assert!(launch(&paths, "design").is_err());
        assert!(service_start(&paths, "design").is_err());
        assert!(!paths.root.exists());
    }
}
