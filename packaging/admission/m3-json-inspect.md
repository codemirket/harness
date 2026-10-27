# JSON inspection feature admission

The user's full authorization for further local implementation covers this
feature-only amendment to the admitted `serde_json =1.0.154`: enable
`raw_value` for borrowed original JSON value slices. It supports exact source
spans and preserves number lexemes without floating-point conversion.

The cached published manifest declares `raw_value = []`. Locked offline Cargo
metadata before and after the change shows identical packages and dependency
edges; only serde_json's selected feature set gains `raw_value`. No new crate,
runtime, download, upstream helper or build script is added. Existing serde_json
build behavior remains the originally admitted build script. The original M0
inventory stays unchanged as historical evidence; the exact feature delta and
unchanged lock hash are in `m3-json-inspect.features.json`. The pre-build guard
checks the selected feature set and the original archive/extracted source pins.

This admits parsing functionality, not arbitrary JSON schema execution, network
references, JSONC, YAML or TOML. Source and result size, nesting and node counts
remain application bounds. No publication or change to the user's installation
is included. Runtime qualification is recorded with the implementation evidence.
