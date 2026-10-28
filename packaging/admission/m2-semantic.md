# Local semantic dependency admission

The root [Cargo manifest](../../Cargo.toml) and [lock](../../Cargo.lock) select
FastEmbed **7.5.0** with dynamic ONNX loading and ort **2.0.0-rc.15** with the
admitted features. The [inventory](m2-semantic.inventory.json) remains an active
input to `scripts/check-build-inputs.py`: versions, source archives, features and
extracted native sources are checked before compilation. The explicit serde_json
`raw_value` amendment is checked by that guard.

The [model/runtime manifest](m2-semantic-minilm.runtime.json) pins the separately
acquired private MiniLM model and ONNX runtime. Setup verifies exact publisher
bytes; normal operation does not download a model or search ambient runtime
paths. Canonical facts and lexical recall remain available when semantic
retrieval is unavailable; report coverage and failure explicitly.

[Advisories](m2-semantic.advisories.json) and
[telemetry review](m2-semantic.telemetry.json) retain the exact admission review;
they are not a current assessment of all future versions. The original batch
and rationale are recorded at the
[immutable admission revision](https://github.com/codemirket/harness/blob/b3068a181a674f2aff751de4ff254f8120e7f96e/packaging/admission/m2-semantic.md).
No additional dependency or runtime is admitted by this summary.

Use [native build instructions](../../docs/native-build.md) for source/package
checks and the [release qualification](../../docs/first-release-qualification-20261028.md)
for actual retrieval, memory-use, idle and platform evidence. Model receipts do
not establish answer usefulness or low memory cost.
