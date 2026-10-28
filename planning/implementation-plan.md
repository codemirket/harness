# Mirket implementation contract

The consolidated plan was approved on 2026-10-25; simplification and HyperFrames
selection were accepted on 2026-10-27, and the evidence-led skill retention rule
and exact first-release platform were accepted on 2026-10-28. This file retains
the active requirements. Completed implementation sequences are not new work.

## Product boundary

Mirket v0.0.1 is a local companion to **Codex Desktop on macOS 26.5.2 ARM64**.
Other macOS versions and Windows are unqualified. Codex Desktop GUI activation
was waived and remains untested; backend/MCP evidence does not establish it.
The [current qualification](../docs/first-release-qualification-20261028.md)
binds the tested package, implementation revision and limits.

| Owner | Responsibility |
| --- | --- |
| Codex | Model/reasoning choice, native agents, permissions, shell, source editing, Git/worktrees, general browser research and live UI review, documents, spreadsheets, presentations and SQL. |
| Mirket | Scoped project memory, selected JS/TS navigation, finite design/video operations, native skill distribution, and its own process/install lifecycle. |
| Project toolchain | Application builds, tests, database operations, browser tests, format-specific authoring and live animation libraries selected by the project. |

Mirket has no general crawler, generic file editor, SQL executor, Office/PDF/OCR
engine, model router or second agent orchestrator. Its owned processes are not
an OS sandbox. Private Chrome is reserved for video rendering. Preserve the
user's live installation and unrelated settings; use disposable homes/projects
for qualification.

## Concrete capability and evidence contracts

| Capability | Implementation contract | Outcome required |
| --- | --- | --- |
| Memory | Canonical SQLite facts/FTS; lazy local multilingual MiniLM/ONNX index and bounded Rust cosine scoring. Explicit project/shared scope, source/revision, applicability, expiry, correction, supersession and forgetting. No passive transcript harvesting or remote vector store. | Independent retrieval and downstream answer oracle; cross-project distractors, concurrent corrections, stale vectors, incomplete indexes, absent model, restart and forgetting across managed backup restore. Report cold/warm and idle resource measurements at their actual observation times. |
| Navigation | TypeScript 5.9.3 inspection/definitions/references over caller-selected JS/TS/JSX/TSX and declaration snapshots with explicit imports and revisions. No project/config/package discovery or execution. | Original UTF-8/CRLF spans, same-name scope distinctions, stale/missing/unsupported refusals and bounded output. Syntax inspection and semantic navigation have separate evidence. |
| Design | Style Dictionary tokens, SVG/SVGO/resvg, Sharp transformations, D2, and separately configured editable OpenPencil subset. Codex authors source and owns creative judgment. | Reopened editable source, changed-source export, consumed tokens, actual glyph/alpha/layout checks and saved-pixel inspection. UI handoff also needs live responsive/accessibility/interaction checks. |
| Video | HyperFrames is the sole engine. Editable local HTML/CSS/JS and selected assets; supplied audio and captions. Exact private render-only Chrome and verified FFmpeg/ffprobe. | Requested MP4/WebM/GIF/PNG output, decoded frames/audio/captions, changed-source/changed-pixels, timing/playback, cancellation, remote-asset refusal and reconciled finalization. Success receipts do not establish visual quality. |
| Native skills | Nine global and 52 opt-in project skills, seven selection profiles; descriptive IDs without a `mirket-` prefix. | Correct scope, required/conditional pairing, hash-bound bodies/resources, collision/drift refusal and rollback. All 305 catalog cases remain planned development cases until actually executed. |
| Coordination | Codex native agents only; useful independent work, disjoint ownership and lead integration. Explicit user model/reasoning choices prevail. | Capable creative lead/comparable reviewer, actual artifact integration, denial/interruption handling and one shared review/repair budget. Observed compliance is not hard enforcement. |

Memory history and backup recovery are product behavior, not removable legacy
compatibility. Known forgetting exclusions take precedence over restored older
managed data. External copies and forensic erasure remain outside that guarantee.
The 60-second semantic idle policy does not itself prove low resource use or
observed retirement; retain the measured limits in the release record.

## Runtime and installation topology

Keep one Rust crate and one compiled TypeScript adapter package. Rust owns
project identity, canonical memory, operation/receipt state, finite jobs and
installation. Node and native tools use explicit verified paths. No PATH search,
silent substitute, first-use download, incidental upstream installer/hook or
project code execution is permitted by these bounded routes.

Use private executable payloads where manageable. For a burdensome native tool,
setup may bind an explicitly selected host executable after capability and
identity checks; every installed job must recheck that binding. The qualified
Mac video package currently uses the self-contained private FFmpeg/ffprobe pair.

The lean package may acquire exact publisher archives for Chrome, native Sharp
and semantic assets during first-party setup, after terms disclosure and hash
verification. This does not waive use or recipient obligations. Redistributed
components require their exact notices/source materials. New dependencies require
approval of the exact batch in `packaging/admission`; proposals and reference
libraries do not grant admission. Preserve those active verification inputs.

Setup owns only its manifest-bound installation, nine global skills, managed
Codex guidance and one MCP registration. Project opt-in pins selection in
`.mirket/project.json`; explicit activation copies selected bodies/resources to
`.agents/skills`. Passive status does not synchronize project files. Detect
collisions and drift, stage atomically and preserve unrelated configuration.
Updates are explicit; busy installations remain staged until activation is
requested again. Uninstall preserves canonical memory/recovery points unless
data deletion is separately explicit. Managed backup policy retains seven daily,
four weekly and protected migration points within a 5 GiB aggregate cap.

## Skill quality and creative production

Keep guidance only when a concrete workflow, quality safeguard or measured
performance benefit justifies its context and maintenance cost. Remove redundant
or non-contributing guidance; a tied small trial does not prove universal
uselessness. The failed six-skill product/marketing bundle and content-growth
profile are retired. Codex continues to own those workflows.

Hallmark/Taste remain selected design references. Reference methods, source code,
fonts, artwork and runtime admission have separate rights. Preserve the accepted
Daybreak direction; changes to an export cannot inherit its human acceptance.
Remotion is excluded from adoption. Do not restart closed model comparisons or
creative experiments without a new task or demonstrated production blocker.

For a new visual task, freeze the brief, real content, scoped outputs, rubric and
shared review/repair budget. Inspect static art before changing motion. Produce
a bounded specimen, inspect saved pixels, export/decode when required, integrate
independent findings and recheck affected states. Keep initial/final evidence and
remaining defects. A new specimen or delegate does not reset the task budget.

## Development checks and stop condition

Run `python3 scripts/check-build-inputs.py` before Rust compilation. Required
affected gates: `cargo test --locked --offline`,
`cargo clippy --locked --offline --all-targets -- -D warnings`, `cargo fmt --check`,
the locked adapter compilation/process tests, and MCP smoke using explicit
private Node and Mirket paths. See [native build instructions](../docs/native-build.md).
Use the actual tested executable in disposable integration environments.

Completion requires actual consumer outcomes and meaningful failure cases,
scoped to the tested platform/revision. Hashes, successful builds, tool lists and
model self-report cannot replace output checks. Preserve failed/unrun evidence;
never weaken a gate to call work complete. Windows gates remain in the
[platform status](../docs/windows-qualification-status.md).

The selected first-release package is locally qualified; its immutable identity
is in the [release record](../packaging/admission/first-release-macos-arm64.json).
Future source/metadata changes do not silently requalify that archive. Publication,
commits, pushes, new dependencies and live installation changes require their
own authorization. The [release checklist](release-readiness.md) covers a future
candidate without reopening completed work automatically.
