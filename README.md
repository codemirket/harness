# Mirket

Mirket is a local companion for **Codex Desktop**. The first release, v0.0.1,
is scoped to **macOS 26.5.2 on Apple Silicon (ARM64)**. Earlier and newer macOS
versions and Windows are not qualified. Codex Desktop GUI activation was waived
by the user and remains untested; native Codex backend and MCP checks are separate.
No public release has been published yet.

Codex owns models, agents, permissions, shell, editing, Git, documents, general
browsing and SQL. Mirket supplies scoped project memory, selected JavaScript and
TypeScript navigation, design operations, HyperFrames video production and native
skill distribution. It does not add another agent orchestrator or OS sandbox.

## What it provides

- **Memory:** canonical SQLite facts and FTS, corrections, supersession, expiry,
  forgetting and recovery, with lazy local MiniLM semantic retrieval.
- **JS/TS navigation:** inspection, definitions and references over explicitly
  supplied, revision-bound sources. Unsupported or incomplete analysis is explicit.
- **Design:** editable OpenPencil files, Style Dictionary tokens, SVG/SVGO/resvg,
  Sharp raster operations and D2 diagrams. Codex owns source authoring and live UI
  review; a successful export is not visual approval.
- **Video:** editable HyperFrames HTML/CSS/JS compositions, local assets, audio and
  captions, with checked MP4/WebM/GIF/PNG exports. Private Chrome is used only for
  video rendering. HyperFrames is the sole video engine.
- **Native guidance:** nine concise global skills and 52 opt-in project skills,
  organized into seven profiles. Exact IDs remain selectable without a profile.
  Names have no `mirket-` prefix. Project sync detects managed-copy drift and
  preserves unrelated user files.

The Hallmark/Taste design direction and user-accepted Daybreak specimen remain.
See the [design production record](docs/design-production-closure-20261028.md)
and [D6 review pack](specimens/daybreak/review.html) for their actual limits.

## Skill retention

Skills must justify their context and maintenance cost through a concrete routing
contract, a non-obvious quality safeguard or measured useful-output benefit.
Catalog size is not a goal. The six tested product/marketing skills and
`content-growth` profile were removed from v0.0.1 after two valid enabled bundles
timed out while both no-skill baselines produced independently correct artifacts.
Codex continues to handle those workflows. This is a bundle-level release choice,
not proof that each removed skill is harmful on every task.

The [evaluation record](docs/supplemental-model-evaluation-20261028.md) preserves
all attempts and costs. Research and reviewer controls passed in both conditions;
no general quality uplift is claimed. The retained 305 catalog development cases
are planned fixtures, not 305 completed acceptance trials.

## Installation and distribution

Use the [release installation prompt](packaging/install-prompt.md) and the exact
installer, binary and manifest hashes from the release record. The first-party
installer refuses unsupported host versions. Installation lives under `.mirket`;
source changes in this checkout do not change an existing installation. The former
installation was retired and replaced on 2026-10-27. Tests use disposable homes.

The lean package includes corresponding source and notices for its redistributed
Video/FFmpeg closure. Chrome for Testing, Sharp native packages and the semantic
model/ONNX runtime are acquired from their publishers during setup under exact
pins and their own terms; they are not mirrored in the lean archive. First use
does not download packages or search PATH for substitute executables.

Mirket-authored source is MIT licensed. Third-party components retain their own
licenses, notices and source obligations. Admission evidence lives under
`packaging/admission`; historical proposals alone do not grant redistribution.
See the [release plan](planning/release-readiness.md) and
[first-release qualification](docs/first-release-qualification-20261028.md) for
current package identities, recipient materials, tests and exclusions.

## Development

Run `python3 scripts/check-build-inputs.py` before Rust compilation. Required
checks include `cargo test --locked --offline`,
`cargo clippy --locked --offline --all-targets -- -D warnings`, `cargo fmt --check`,
the locked adapter build/tests and MCP smoke with explicit private Node and binary
paths. See [native build requirements](docs/native-build.md).

The [implementation plan](planning/implementation-plan.md) defines ownership and
release constraints. The [catalog plan](planning/skill-catalog-design.md) and
[provenance](skills/PROVENANCE.md) define current skill scope. Earlier qualification
records in `docs/` preserve their original revisions and failed attempts; they do
not independently qualify a newer package or unsupported platform.
