# Mirket implementation contract

The pi-only direction supersedes the earlier host contract. Execute the phases
and exit gates in [the migration plan](pi-migration.md). The scoped local technical candidate is validated; see the
[qualification record](../docs/pi-qualification-20261029.md). Public release, live
installation and production browser admission remain outside this authorization.
Earlier qualification records describe their original artifacts and must not be relabeled.

## Product boundary

Pi is the sole supported agent host. The qualification target is macOS 26.5.2
ARM64. Pi owns models, authentication, sessions, agent loop, compaction, native
read/edit/write/shell tools, package resources and the MCP client. Mirket supplies
scoped canonical memory, selected navigation, finite creative operations and
bounded workflows. It does not implement another provider client or agent loop.
Project toolchains own application builds, tests, SQL and format-specific exports.
Browser and research access need their own admitted concrete integrations.

Mirket-owned processes are not an OS sandbox. Private rendering Chrome is
reserved for video. Preserve live installations and unrelated settings; use
private disposable HOME, agent/session directories and project ancestry for tests.

## Concrete capability and evidence contracts

| Capability | Implementation contract | Outcome required |
| --- | --- | --- |
| Memory | Canonical SQLite facts/FTS; lazy local multilingual MiniLM/ONNX index and bounded Rust cosine scoring. Explicit project/shared scope, source/revision, applicability, expiry, correction, supersession and forgetting. No passive transcript harvesting or remote vector store. | Independent retrieval and downstream answer oracle; cross-project distractors, concurrent corrections, stale vectors, incomplete indexes, absent model, restart and forgetting across managed backup restore. Report cold/warm and idle resource measurements at their actual observation times. |
| Navigation | TypeScript 5.9.3 inspection/definitions/references over caller-selected JS/TS/JSX/TSX and declaration snapshots with explicit imports and revisions. No project/config/package discovery or execution. | Original UTF-8/CRLF spans, same-name scope distinctions, stale/missing/unsupported refusals and bounded output. Syntax inspection and semantic navigation have separate evidence. |
| Design | Style Dictionary tokens, SVG/SVGO/resvg, Sharp transformations, D2, and separately configured editable OpenPencil subset. The pi author and capable reviewer own source and creative judgment. | Reopened editable source, changed-source export, consumed tokens, actual glyph/alpha/layout checks and saved-pixel inspection. UI handoff also needs live responsive/accessibility/interaction checks. |
| Video | HyperFrames is the sole engine. Editable local HTML/CSS/JS and selected assets; supplied audio and captions. Exact private render-only Chrome and verified FFmpeg/ffprobe. | Requested MP4/WebM/GIF/PNG output, decoded frames/audio/captions, changed-source/changed-pixels, timing/playback, cancellation, remote-asset refusal and reconciled finalization. Success receipts do not establish visual quality. |
| Native skills | Evidence-led selected pi package resources; descriptive IDs without a `mirket-` prefix. Existing inventory is an audit input, not a retention target. | Correct scope, required/conditional pairing, hash-bound bodies/resources, native pi resource filtering, collision diagnosis and truthful prerequisites. All 305 catalog cases remain planned development cases until actually executed. |
| Coordination | Optional bounded pi CLI children; useful independent work, disjoint ownership and lead integration. Explicit user model/reasoning choices prevail. | Capable creative lead/comparable reviewer, actual artifact integration, denial/interruption handling and one shared review/repair budget. Observed compliance is not hard enforcement. |

Memory history and backup recovery are product behavior, not removable legacy
compatibility. Known forgetting exclusions take precedence over restored older
managed data. External copies and forensic erasure remain outside that guarantee.
The 60-second semantic idle policy does not itself prove low resource use or
observed retirement; retain the measured limits in the release record.

## Runtime and installation topology

Keep one Rust crate and one compiled TypeScript package under `pi/`, with extension and runtime modules. Rust owns
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

State belongs under `<pi-agent-dir>/mirket/`, respecting `PI_CODING_AGENT_DIR`.
Session storage does not define canonical memory scope. Native setup accepts an
explicit verified payload and fresh private home. Existing installations are not
imported. Pi owns package resources; profiles modify only Mirket's package object
in pi settings with conflict detection. Do not copy skills into host directories
or create a second selection database. Updates are explicit; busy installations
remain staged. Preserve canonical memory unless deletion is explicitly authorized.
Managed backup policy retains seven daily, four weekly and protected migration
points within a 5 GiB aggregate cap.

## Skill quality and creative production

Keep guidance only when a concrete workflow, quality safeguard or measured
performance benefit justifies its context and maintenance cost. Remove redundant
or non-contributing guidance; a tied small trial does not prove universal
uselessness. The failed six-skill product/marketing bundle and content-growth
profile are retired. The host can perform those workflows without restoring the retired bundle.

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

The earlier release record retains its original immutable identity. It does not
qualify the pi product. Publication, commits, pushes, new dependencies and live
installation changes require their own authorization. The migration plan defines
the fresh candidate gates, including actual installed outcomes and held-out trials.
