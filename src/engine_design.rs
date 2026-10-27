//! Revision-bound design operations use the shared artifact, receipt and worker owners.
//!
//! This surface deliberately exposes only finite preparation/start pairs. The
//! adapter receives a private scratch copy and cannot select paths, load project
//! configuration, or publish directly.

use super::*;
use crate::design::{DesignOperation, TokenFormat};

const MAX_DESIGN_OUTPUT_BYTES: usize = crate::design::MAX_OUTPUT_BYTES;

#[derive(Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DesignPrepareArgs {
    pub expected_release_id: String,
    pub project_root: PathBuf,
    pub source_path: PathBuf,
    pub expected_revision: String,
    pub destination_path: PathBuf,
    pub operation: DesignOperation,
}

fn contract(args: &DesignPrepareArgs) -> Result<(&'static str, &'static str), String> {
    match &args.operation {
        DesignOperation::TokensCompile {
            format: TokenFormat::Css,
        } => {
            if args.source_path.extension().and_then(|part| part.to_str()) != Some("json")
                || args
                    .destination_path
                    .extension()
                    .and_then(|part| part.to_str())
                    != Some("css")
            {
                return Err(
                    "token CSS compilation requires .json input and .css new output".into(),
                );
            }
            Ok(("source.json", "result.css"))
        }
        DesignOperation::TokensCompile {
            format: TokenFormat::Json,
        } => {
            if args.source_path.extension().and_then(|part| part.to_str()) != Some("json")
                || args
                    .destination_path
                    .extension()
                    .and_then(|part| part.to_str())
                    != Some("json")
            {
                return Err(
                    "token JSON compilation requires .json input and .json new output".into(),
                );
            }
            Ok(("source.json", "result.json"))
        }
        DesignOperation::SvgOptimize => {
            if args.source_path.extension().and_then(|part| part.to_str()) != Some("svg")
                || args
                    .destination_path
                    .extension()
                    .and_then(|part| part.to_str())
                    != Some("svg")
            {
                return Err("SVG optimization requires .svg input and .svg new output".into());
            }
            Ok(("source.svg", "result.svg"))
        }
        DesignOperation::SvgRenderPng { width, height } => {
            if *width == 0
                || *height == 0
                || *width > crate::design::MAX_DIMENSION
                || *height > crate::design::MAX_DIMENSION
            {
                return Err("SVG raster dimensions must be between 1 and 2048 pixels".into());
            }
            if args.source_path.extension().and_then(|part| part.to_str()) != Some("svg")
                || args
                    .destination_path
                    .extension()
                    .and_then(|part| part.to_str())
                    != Some("png")
            {
                return Err("SVG PNG rendering requires .svg input and .png new output".into());
            }
            Ok(("source.svg", "result.png"))
        }
        DesignOperation::RasterTransform { .. } => {
            let input = args.operation.input_extension();
            let output = args.operation.output_extension();
            if args.source_path.extension().and_then(|part| part.to_str()) != Some(input)
                || args
                    .destination_path
                    .extension()
                    .and_then(|part| part.to_str())
                    != Some(output)
            {
                return Err(format!(
                    "raster transform requires .{input} input and .{output} new output"
                ));
            }
            Ok((
                match input {
                    "png" => "source.png",
                    "jpg" => "source.jpg",
                    "webp" => "source.webp",
                    "avif" => "source.avif",
                    _ => unreachable!(),
                },
                match output {
                    "png" => "result.png",
                    "jpg" => "result.jpg",
                    "webp" => "result.webp",
                    "avif" => "result.avif",
                    _ => unreachable!(),
                },
            ))
        }
        DesignOperation::OpenPencil { operation } => {
            operation.validate()?;
            let input = operation.input_extension();
            let output = operation.output_extension();
            if args.source_path.extension().and_then(|part| part.to_str()) != Some(input)
                || args
                    .destination_path
                    .extension()
                    .and_then(|part| part.to_str())
                    != Some(output)
            {
                return Err(format!(
                    "OpenPencil {} requires .{input} input and .{output} new output",
                    operation.wire_name()
                ));
            }
            Ok((
                if input == "json" {
                    "source.json"
                } else {
                    "source.fig"
                },
                if output == "svg" {
                    "result.svg"
                } else {
                    "result.fig"
                },
            ))
        }
    }
}

impl Engine {
    pub fn prepare_design(&self, args: DesignPrepareArgs) -> Result<Value, String> {
        self.guard(&args.expected_release_id)?;
        match &args.operation {
            DesignOperation::OpenPencil { .. } if self.payload.openpencil.is_none() => {
                return Err("private OpenPencil shape-file runtime is not configured".into());
            }
            DesignOperation::OpenPencil { .. } => {}
            DesignOperation::RasterTransform { .. }
                if self
                    .payload
                    .design
                    .as_ref()
                    .is_none_or(|payload| payload.raster_module.is_none()) =>
            {
                return Err("private Sharp raster runtime is not configured".into());
            }
            _ if self.payload.design.is_none() => {
                return Err("private design runtime is not configured".into());
            }
            _ => {}
        }
        let registry = self.running.lock().map_err(err)?;
        if registry.closed {
            return Err("connection is shutting down; no new intents may be prepared".into());
        }
        let (_source_name, _output_name) = contract(&args)?;
        let scope = ProjectScope::open(&args.project_root).map_err(err)?;
        let source = scope
            .resolve_existing_file(&args.source_path)
            .map_err(err)?;
        let metadata = fs::symlink_metadata(&source).map_err(err)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err("design source must be a regular non-symlink file".into());
        }
        if metadata.len() > args.operation.max_input_bytes() as u64 {
            return Err("design source exceeds its operation byte limit".into());
        }
        let destination = if args.destination_path.is_absolute() {
            args.destination_path
                .strip_prefix(&args.project_root)
                .map(|path| scope.root().join(path))
                .unwrap_or(args.destination_path)
        } else {
            args.destination_path
        };
        let plan = ArtifactPlan::prepare(
            scope.root(),
            Some(&source),
            &destination,
            PublicationMode::NewCopy,
            MAX_DESIGN_OUTPUT_BYTES,
        )
        .map_err(err)?;
        let selected = plan
            .source_bytes()
            .ok_or("design source selection missing")?;
        if selected.len() > args.operation.max_input_bytes() {
            return Err("design source exceeds its operation byte limit".into());
        }
        let source_revision = revision(selected);
        if source_revision != args.expected_revision {
            return Err("design source revision changed; inspect current source".into());
        }
        let created_at = now()?;
        let receipt = Receipt {
            schema: 2,
            kind: OperationKind::Design,
            release_id: self.release.as_str().into(),
            owner_id: self.owner_id.clone(),
            project_root: scope.root().into(),
            source_path: source,
            destination_path: plan.destination().into(),
            source_revision,
            intent_revision: revision(
                &serde_json::to_vec(
                    &json!({"operation": args.operation.clone(), "destination": plan.destination()}),
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
        self.issue_preparation(plan, PendingOperation::Design(args.operation), receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn design_contract_binds_operation_to_extensions_and_bounds() {
        let base = DesignPrepareArgs {
            expected_release_id: "test".into(),
            project_root: PathBuf::from("/tmp/project"),
            source_path: PathBuf::from("tokens.json"),
            expected_revision: "sha256:unused".into(),
            destination_path: PathBuf::from("tokens.css"),
            operation: DesignOperation::TokensCompile {
                format: TokenFormat::Css,
            },
        };
        assert_eq!(contract(&base).unwrap(), ("source.json", "result.css"));

        let mut bad = base.clone();
        bad.destination_path = PathBuf::from("tokens.svg");
        assert!(contract(&bad).is_err());

        let mut raster = base;
        raster.source_path = PathBuf::from("source.svg");
        raster.destination_path = PathBuf::from("image.png");
        raster.operation = DesignOperation::SvgRenderPng {
            width: 4097,
            height: 1,
        };
        assert!(contract(&raster).is_err());

        raster.operation = DesignOperation::OpenPencil {
            operation: crate::openpencil::OpenPencilOperation::Create,
        };
        raster.source_path = PathBuf::from("source.json");
        raster.destination_path = PathBuf::from("shape.fig");
        assert_eq!(contract(&raster).unwrap(), ("source.json", "result.fig"));
        raster.destination_path = PathBuf::from("shape.svg");
        assert!(contract(&raster).is_err());

        raster.operation = DesignOperation::RasterTransform {
            source_format: crate::design::RasterFormat::Png,
            output_format: crate::design::RasterFormat::Webp,
            width: 100,
            height: 100,
            fit: crate::design::RasterFit::Inside,
        };
        raster.source_path = PathBuf::from("source.png");
        raster.destination_path = PathBuf::from("image.webp");
        assert_eq!(contract(&raster).unwrap(), ("source.png", "result.webp"));
        raster.destination_path = PathBuf::from("image.jpg");
        assert!(contract(&raster).is_err());
    }
}
