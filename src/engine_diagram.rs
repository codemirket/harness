//! Revision-bound D2 rendering uses the shared artifact, receipt and worker owners.
use super::*;
use crate::diagram::{self, D2Layout};

#[derive(Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagramPrepareArgs {
    pub expected_release_id: String,
    pub project_root: PathBuf,
    pub source_path: PathBuf,
    pub expected_revision: String,
    pub destination_path: PathBuf,
    pub layout: D2Layout,
    #[serde(default)]
    pub output_format: diagram::D2OutputFormat,
}

impl Engine {
    pub fn prepare_diagram(&self, args: DiagramPrepareArgs) -> Result<Value, String> {
        self.guard(&args.expected_release_id)?;
        if self.payload.diagram.is_none() {
            return Err("private D2 runtime is not configured".into());
        }
        let registry = self.running.lock().map_err(err)?;
        if registry.closed {
            return Err("connection is shutting down; no new intents may be prepared".into());
        }
        if args.source_path.extension().and_then(|part| part.to_str()) != Some("d2")
            || args
                .destination_path
                .extension()
                .and_then(|part| part.to_str())
                != Some(args.output_format.extension())
        {
            return Err("D2 input must end in .d2 and new output extension must match output_format (svg or png)".into());
        }
        let scope = ProjectScope::open(&args.project_root).map_err(err)?;
        let source = scope
            .resolve_existing_file(&args.source_path)
            .map_err(err)?;
        let destination = if args.destination_path.is_absolute() {
            args.destination_path
                .strip_prefix(&args.project_root)
                .map(|path| scope.root().join(path))
                .unwrap_or(args.destination_path.clone())
        } else {
            args.destination_path
        };
        let plan = ArtifactPlan::prepare(
            scope.root(),
            Some(&source),
            &destination,
            PublicationMode::NewCopy,
            diagram::MAX_OUTPUT_BYTES,
        )
        .map_err(err)?;
        let selected = plan.source_bytes().ok_or("D2 source selection missing")?;
        if selected.len() > diagram::MAX_SOURCE_BYTES {
            return Err("D2 source exceeds 1 MiB".into());
        }
        let text = std::str::from_utf8(selected).map_err(|_| "D2 source must be UTF-8")?;
        diagram::validate_source(text).map_err(err)?;
        let source_revision = revision(selected);
        if source_revision != args.expected_revision {
            return Err("D2 source revision changed; inspect current source".into());
        }
        let created_at = now()?;
        let receipt = Receipt {
            schema: 2,
            kind: OperationKind::Diagram,
            release_id: self.release.as_str().into(),
            owner_id: self.owner_id.clone(),
            project_root: scope.root().into(),
            source_path: source,
            destination_path: plan.destination().into(),
            source_revision,
            intent_revision: revision(
                &serde_json::to_vec(
                    &json!({"layout":args.layout,"output_format":args.output_format,"destination":plan.destination()}),
                )
                .map_err(err)?,
            ),
            created_at,
            expires_at: created_at.checked_add(300).ok_or("invalid system clock")?,
            state: JobState::default(),
            artifact_revision: None,
            error: None,
            scratch_cleanup: None,
            queue_wait_ms: None,
        };
        self.issue_preparation(
            plan,
            PendingOperation::Diagram(args.layout, args.output_format),
            receipt,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagram_prepare_defaults_to_svg_and_rejects_unknown_format() {
        let mut value = serde_json::json!({
            "expected_release_id":"release", "project_root":"/private/project",
            "source_path":"source.d2", "expected_revision":"revision",
            "destination_path":"result.svg", "layout":"elk"
        });
        let args: DiagramPrepareArgs = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(args.output_format, diagram::D2OutputFormat::Svg);
        value["output_format"] = serde_json::json!("pdf");
        assert!(serde_json::from_value::<DiagramPrepareArgs>(value).is_err());
    }
}
