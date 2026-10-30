# Mirket implementation contract

This is the current implementation contract. See [qualification](qualification.md)
for the tested candidate and platform limits.

## Product boundary

Pi is the sole supported agent host. The qualification target is macOS 26.5.2
ARM64. Pi owns models, authentication, sessions, agent loop, compaction, native
read/edit/write/shell tools, package resources and the MCP client. Mirket supplies
scoped canonical memory, selected navigation, finite creative operations and
bounded workflows. It does not implement another provider client or agent loop.
Project toolchains own application builds, tests, SQL and format-specific exports.
Browser QA uses an explicitly configured Pi MCP peer when selected; research
uses Pi's provider-native search when configured.

Mirket-owned processes are not an OS sandbox. Private rendering Chrome is
reserved for video. Preserve live installations and unrelated settings; use
private disposable HOME, agent/session directories and project ancestry for tests.

## Concrete capability and evidence contracts

| Capability | Implementation contract | Outcome required |
| --- | --- | --- |
| Memory | Canonical SQLite facts/FTS; lazy local multilingual MiniLM/ONNX index and bounded Rust cosine scoring. Explicit project/shared scope, source/revision, applicability, expiry, correction, supersession and forgetting. No passive transcript harvesting or remote vector store. | Independent retrieval and downstream answer oracle; cross-project distractors, concurrent corrections, stale vectors, incomplete indexes, absent model, restart and forgetting across managed backup restore. Report cold/warm and idle resource measurements at their actual observation times. |
| Navigation | TypeScript 5.9.3 inspection/definitions/references over caller-selected JS/TS/JSX/TSX and declaration snapshots with explicit imports and revisions. No project/config/package discovery or execution. | Original UTF-8/CRLF spans, same-name scope distinctions, stale/missing/unsupported refusals and bounded output. Syntax inspection and semantic navigation have separate evidence. |
| Design | Style Dictionary tokens, SVG/SVGO/resvg, D2 to SVG or direct bounded PNG, and separately configured editable OpenPencil subset. Raster/Sharp transformations remain excluded. The pi author and capable reviewer own source and creative judgment. | Reopened editable source, changed-source export, consumed tokens, actual glyph/alpha/layout checks and saved-pixel inspection. UI handoff also needs live responsive/accessibility/interaction checks. |
| Video | HyperFrames is the sole engine. Editable local HTML/CSS/JS and selected assets; supplied audio and captions. Exact private render-only Chrome and verified FFmpeg/ffprobe. | Requested MP4/WebM/GIF/PNG output, decoded frames/audio/captions, audio stream-end checks, changed-source/changed-pixels, timing/playback, cancellation, remote-asset refusal and reconciled finalization. Success receipts do not establish visual or audible quality. |
| Native skills | Evidence-led selected pi package resources; descriptive IDs without a `mirket-` prefix. Existing inventory is an audit input, not a retention target. | Correct scope, required/conditional pairing, hash-bound bodies/resources, native pi resource filtering, collision diagnosis and truthful prerequisites. All 305 catalog cases remain planned development cases until actually executed. |
| Coordination | Optional bounded pi CLI children; useful independent work, disjoint ownership and lead integration. Explicit user model/reasoning choices prevail. | Capable creative lead/comparable reviewer, actual artifact integration, denial/interruption handling and one shared review/repair budget. Observed compliance is not hard enforcement. |

Optional browser QA is a separate Pi-owned Playwright MCP peer. Mirket ships
only pins and first-party onboarding/control code; it does not redistribute
Playwright or Chrome. Setup chooses a verified publisher download, an exact
installed path or Skip for the peer and its separate browser executable. A
confirmed Pi TUI command grants at most 300 seconds and 64 calls against one
literal `http://127.0.0.1:PORT` origin. The seven exposed tools cover navigation,
snapshot, PNG screenshot, resize, limited keys, observed-text wait and close.
The application proxy forwards bounded GET/HEAD requests to that exact loopback
port and refuses redirects; it is not an OS network sandbox. Cancellation
revokes the grant and unregisters the peer before a fresh authorization.

Known forgetting exclusions take precedence over restored older
managed data. External copies and forensic erasure remain outside that guarantee.
The 60-second semantic idle policy does not establish low resource use or
observed retirement; see the [measured limits](qualification.md).

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

The package bundles its manageable private runtimes and selected source/notice
materials. Chrome is acquired directly from its publisher during an explicit
onboarding choice or bound to a selected existing executable after disclosure
and verification. It is never bundled in Mirket on the current rights evidence.
The optional Playwright peer and its separately selected browser also stay
external. Their bindings are rechecked before each affected operation; neither
is inferred from the video Chrome binding or searched on PATH.
New production dependencies still require user authorization. Current byte/source
pins live under `packaging/runtime/`; they establish identity, not legal clearance
or platform qualification. See [packaging](../packaging/README.md) for exact scopes
and unresolved optional-component rights.

State belongs under `<pi-agent-dir>/mirket/`, respecting `PI_CODING_AGENT_DIR`.
Session storage does not define canonical memory scope. Native setup accepts an
explicit verified payload and fresh private home. Existing installations are not
imported. Pi owns package resources; profiles modify only Mirket's package object
in pi settings with conflict detection. Do not copy skills into host directories
or create a second selection database. Updates are explicit; busy installations
remain staged. Preserve canonical memory unless deletion is explicitly authorized.
Managed backup policy retains seven daily, four weekly and protected migration
points within a 5 GiB aggregate cap.

Explicit `/mirket-project init` initializes the selected project through its Pi
package settings and a managed `AGENTS.md` section. It opts that project into
section updates based on Mirket skills and tools actually used there. These
observations neither select additional skills nor prove readiness, successful
outcomes or whole-project coverage. Preserve other guidance and unrelated
settings; refuse conflicting managed content or ownership. Project guidance is
not canonical memory identity, source discovery or a second selection store.

## Skill quality and creative production

Keep guidance only when a concrete workflow, quality safeguard or measured
performance benefit justifies its context and maintenance cost. Remove redundant
or non-contributing guidance; a tied small trial does not prove universal
uselessness. The host owns workflows that do not need a selected Mirket skill.

Hallmark, Taste and Impeccable are selected design references: structure,
brief-led direction and rendered anti-pattern/product-truth review respectively.
Consult the relevant pinned portion for the task; their methods do not impose a
single aesthetic or substitute for inspected output. Reference methods, source code,
fonts, artwork and runtime acquisition have separate provenance. The accepted
[Daybreak reference](../specimens/daybreak/README.md) does not confer acceptance
on changed exports. HyperFrames is the video engine; Remotion is reference only.

The OpenPencil authoring route uses exact file/revision/target scope and
container-owned layout within the admitted native subset. Video review records
absolute beat and boundary frames, including move midpoints and the actual last
frame. FFmpeg verifies encoded audio extent; hearing the content and checking
gain/sync remain part of saved-output review. These are focused method adoptions,
not new runtimes or permission to use upstream installers.

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
never weaken a gate to call work complete. Other platforms require separate native qualification.

Publication, pushes, new dependencies and live installation changes require
their applicable authorization. Preserve current byte/source pins and provenance inputs.
