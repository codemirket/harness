# Engineering workflow source review

Reviewed on 2026-10-10. The ten repositories below were freshly fetched into an
isolated temporary directory with Git hooks disabled. Review included actual skill
bodies, selected implementation paths, activation manifests, runtime requirements,
and license declarations. No upstream code, installers, hooks, browser, MCP server,
or evaluation runner was executed. This is a source review, not a security audit of
every line or evidence of runtime performance. The accompanying
[machine-readable review](workflows.json) records every inspected file and exact
revision; large implementation files were inspected in the sections identified
below, not exhaustively.

## Decisions that should shape this harness

Keep the installation and task-execution contracts separate. A registered skill,
matching hash, available executable, successful invocation, useful artifact, and
accepted user outcome are different claims. Give each a distinct evidence field.
Do not infer the last five from the first. Existing baseline capability validation
at `ab44f6b7ed9f1436bee506d8194a8a05da5be189` checked authored guidance and
registration, while its delivery contract remained descriptive. The useful
extension is an inspectable task route and outcome evidence, not another runtime
framework or a list of personas.

Use a small task router with a primary capability, a concrete deliverable and a
failure probe. Add specialists when their guidance changes an actual decision.
Keep stack-specific references behind meaningful triggers. Multiple sources
independently support short pointers, bounded subagent briefs, and source-linked
evidence; several undermine those benefits with session-wide injected workflows.
Negative routing cases matter as much as positive cases: a typo, translation,
status request, or narrow code lookup should not trigger planning, two reviewers,
a browser, new issues, or a cross-model debate.

Treat context as evidence with a retrieval path. Preserve exact errors, units,
negation, authorization, uncertainty, and consumer-visible contracts. Store large
outputs outside the active context when helpful, then retain their locator,
command/query, revision or content hash, completeness, and decisive result.
Compression and retrieval may omit the only relevant fact; they need recovery and
quality checks. Do not compress third-party tool contracts by deleting words, or
claim a token win against a deliberately naive full-repository baseline.

Reuse verification when the relevant inputs are unchanged. Bind checks to the
files/configuration/dependencies/environment they actually tested. A fresh command
on every conversational response wastes time; a cached green result after a
relevant change is wrong. Keep requirements review, implementation review,
integration behavior, and visual acceptance distinguishable. Use independent
review where a second perspective can expose a consequential blind spot, not as
fixed ceremony for every edit.

Measure cost per successful task, including retries, recovery retrieval, human
correction, latency, and failed outcomes. Structural checks, lexical routing
proxies, real-host activation traces, and behavior/artifact grading answer
different questions. Compare additions against the current harness and a simple
baseline on the same tasks; record model, client version, skill bytes and tool
configuration. Upstream percentages below are not measurements of this harness.

Keep optional runtimes opt-in. Many nominal skill packages also install binaries,
register servers, rewrite client settings, capture tool I/O, change model routing,
start update checks, or write persistent memory. An upstream permission directive
does not replace the user's authorization or host tool contract. Preserve each
source's license and notices if copying text or code; this review supports
original, selective adaptations rather than importing whole packages.

## Graphify

**Decision: adapt graph provenance and cache invalidation techniques; defer the
runtime until a representative repository task establishes its value.** Revision
[`2cb81c6`](https://github.com/Graphify-Labs/graphify/tree/2cb81c6fd5ec0758453606a3a76e516e893199b4).

The [architecture](https://github.com/Graphify-Labs/graphify/blob/2cb81c6fd5ec0758453606a3a76e516e893199b4/ARCHITECTURE.md)
describes deterministic extraction, graph construction, clustering and export.
Source locations and confidence classes distinguish extracted from inferred or
ambiguous edges. That distinction is valuable for impact analysis: graph edges
are hypotheses or indexed observations with origins, not proof that no other
callers exist. Its serialized undirected graph has conventions for preserving
directed edge meaning; a consumer should use the appropriate loader rather than
guessing direction from JSON.

The [Codex skill](https://github.com/Graphify-Labs/graphify/blob/2cb81c6fd5ec0758453606a3a76e516e893199b4/graphify/skill-codex.md)
routes broadly to an existing graph and can bypass rebuild detection. The inspected
`graph_stats` handler in `graphify/serve.py` compares graph build revision with Git
HEAD; matching HEAD does not establish freshness for dirty files. In
`graphify/detect.py`, the incremental path also relies on file metadata and cached
hash/schema data. The focused
[deleted-target regression](https://github.com/Graphify-Labs/graphify/blob/2cb81c6fd5ec0758453606a3a76e516e893199b4/tests/test_cache_stale_import_target.py)
is particularly useful: changing an imported file must invalidate an unchanged
importer's resolution. Adopt this warm-cache versus cold-extraction invariant for
any future index.

`graphify/benchmark.py` estimates context reduction against a naive corpus-sized
baseline, includes approximate token accounting, and excludes queries without
matches from some aggregation. It does not establish task correctness, superiority
over focused `rg`, or cost per successful change. The reviewed security helper
rejects selected internal URL destinations and limits input sizes; this is not
proof that executing the package is isolated.

`pyproject.toml` specifies Python 3.10+, NetworkX, NumPy, RapidFuzz, and numerous
Tree-sitter parsers; MCP and document formats add optional dependencies. Hooks
can rebuild after checkout/commit and bind to a Python runtime. The skill's
installation fallbacks include `--break-system-packages`, which should not enter
personal defaults. The current Apache-2.0 distribution retains earlier MIT
attribution in `LICENSE-MIT` and `NOTICE`.

**Application:** continue targeted source retrieval by default. If graph support
is selected, require repository identity, dirty-input fingerprint, parser/schema
version, extraction errors, edge confidence and refresh status with every result.
Exercise stale imports, deleted files and ignored/generated files before trusting
impact analysis. Preserve a direct source lookup fallback. Do not activate
post-commit hooks just to make a skill available.

## Superpowers

**Decision: adapt debugging, task briefs and evidence discipline; reject universal
bootstrap, automatic commits and model-selection prescriptions.** Revision
[`bb92a77`](https://github.com/obra/superpowers/tree/bb92a77741419a4ab5f06e711a283343f1ada0c3),
MIT.

The [debugging body](https://github.com/obra/superpowers/blob/bb92a77741419a4ab5f06e711a283343f1ada0c3/skills/systematic-debugging/SKILL.md)
and condition-waiting companion provide concrete value: reproduce the symptom,
trace a failed boundary, form a falsifiable hypothesis, and change one relevant
variable. Wait for a defined condition with a timeout instead of sleeping an
arbitrary duration. The implementer prompt scopes ownership, forbids cascading
delegation, preserves focused verification and gives a concise completion report.
Those ideas fit the current harness without a second planner.

The [verification skill](https://github.com/obra/superpowers/blob/bb92a77741419a4ab5f06e711a283343f1ada0c3/skills/verification-before-completion/SKILL.md)
correctly distinguishes test success, build success, requirements coverage and an
agent's report. However, its demand for a new verification command in the same
message, and categorical rejection of partial checks, prevents sensible reuse of
unchanged evidence and risk-based focused checks. A production mitigation can also
be necessary before root cause is fully understood; the debugging phase order
should not forbid restoring service under the user's incident scope.

`using-superpowers` mandates skill lookup at a one-percent relevance threshold
before any response or action. `hooks/session-start` injects that workflow for
supported hosts. The Codex plugin's `hooks: {}` differs from Claude's hook setup;
shipping a manifest does not establish the same activation in both clients.

The [Codex adapter](https://github.com/obra/superpowers/blob/bb92a77741419a4ab5f06e711a283343f1ada0c3/skills/using-superpowers/references/codex-tools.md)
contains a material compatibility error for this session: it permits explicit
model overrides on full-history subagent forks, whereas the exposed host tool
contract disallows that combination. It also prescribes explicit cheaper-model
selection on every spawn, long waits, and commit behavior. These instructions
cannot override the current host or user. The implementer prompt's directive to
commit and guidance that reviewers need not rerun reported checks also need
task-specific interpretation.

**Application:** retain hypothesis-driven debugging, explicit completion evidence,
and bounded ownership. Route to planning and independent review proportionally.
Use actual host schemas for delegation; inherit the user's model unless an
authorized override applies. Keep commits and publication separate from a
successful implementation. Replace blanket fresh-check rules with input-bound
evidence invalidation.

## Matt Pocock's skills

**Decision: adapt agent-facing writing, review separation and correct regression
test seams; decline mandatory ticket/commit ceremonies.** Revision
[`49dd158`](https://github.com/mattpocock/skills/tree/49dd158d1076134a641b33efb035946536778336),
MIT. Current skill locations are under `skills/engineering`, `productivity` and
`misc`; older flattened paths are stale.

[Writing for agents](https://github.com/mattpocock/skills/blob/49dd158d1076134a641b33efb035946536778336/skills/productivity/writing-for-agents/SKILL.md)
is the strongest general contribution: a pointer should say when to follow it,
each branch should expose its own next reference, and entrypoints should preserve
completion criteria rather than cache facts already discoverable from manifests.
Split material when it changes a branch or sequence, not merely to achieve a
preferred file length. The mechanics companion contains host-specific invocation
fields that should be validated against the actual target before use.

The [code-review skill](https://github.com/mattpocock/skills/blob/49dd158d1076134a641b33efb035946536778336/skills/engineering/code-review/SKILL.md)
separates a standards check from a specification check, reducing the chance that
clean implementation hides the wrong behavior. Its prescribed `base...HEAD` diff
does not cover uncommitted edits, despite discussing work in progress. A harness
review must explicitly choose branch, staged, unstaged and untracked scope. Two
parallel reviewers and a rigid short report are not always economical.

`codebase-design` treats interface complexity as including invariants, failures,
configuration and operational behavior, not just signatures. The deletion test
helps detect abstractions that merely relay calls. A boundary can still be useful
with one implementation when it isolates trust or ownership; do not make adapter
count a universal design rule. `diagnosing-bugs` emphasizes the real production
call pattern and removal of diagnostic instrumentation. A regression test should
exercise the failing seam instead of a convenient fake that cannot reproduce it.

`wayfinder` separates the destination, decision frontier and genuinely blocked
work; these fit an existing local task record. Its GitHub issues, one-ticket
session structure, research branches and push directions would otherwise expand
external work. The tiny `implement` skill also ends with an automatic commit.

The [Git guardrail script](https://github.com/mattpocock/skills/blob/49dd158d1076134a641b33efb035946536778336/skills/misc/git-guardrails-claude-code/scripts/block-dangerous-git.sh)
uses Bash, `jq` and command-string regular expressions. It is a useful accidental
operation tripwire, not a complete shell authorization mechanism: wrappers,
alternate spellings and commands embedded as data make string matching weaker
than parsed, host-enforced permissions. Prompt-only skills need no package
installation; repository package dependencies maintain releases.

**Application:** author concise descriptions and conditional references; preserve
real completion criteria. Review intended behavior and code quality separately
when useful, using the actual dirty/committed scope. Keep task state in the
project's owning record, and require no ticket creation or commit merely to use
an engineering skill.

## ECC

**Decision: adapt its harness contracts, selective context accounting and evaluation
taxonomy; defer plugin runtime, observers and broad hooks.** Revision
[`4eb71d9`](https://github.com/affaan-m/ECC/tree/4eb71d92a39cab44ad40ac9d8a6a5ccb4029d6c2),
MIT, package version 2.2.3.

[Agent harness construction](https://github.com/affaan-m/ECC/blob/4eb71d92a39cab44ad40ac9d8a6a5ccb4029d6c2/skills/agent-harness-construction/SKILL.md)
recommends narrow action spaces, validated inputs and deterministic structured
results. Macro tools are justified when round trips dominate; irreversible or
ambiguous actions benefit from smaller steps. Completion rate, retries and cost
per success are better measures than raw token reduction. Its example result
envelope is a starting point, not sufficient proof: include incomplete/truncated
states, origin, version and effects where relevant.

`context-budget` distinguishes always-loaded from conditional content and warns
that persisted transcript size is not the model's active window or billable token
count. Its word and per-tool estimates remain estimates. `eval-harness` separates
capability and regression tasks and code/model/human graders. A symbol lookup is
not a substitute for behavior, and a universal percentage target is not meaningful
without a representative denominator and repeatability.

The [adapter compliance implementation](https://github.com/affaan-m/ECC/blob/4eb71d92a39cab44ad40ac9d8a6a5ccb4029d6c2/scripts/lib/harness-adapter-compliance.js)
distinguishes native, adapter-backed, instruction-backed and reference-only
support. This is useful metadata, but its dated assertions do not prove current
client behavior. For example, the current Codex package supplies a SessionStart
hook even where a matrix entry characterizes older instruction-level support.

The activation surfaces are substantial. Claude `hooks/hooks.json` wires command
dispatch, capture, edit checks, compaction and lifecycle actions with profiles.
The Codex manifest includes skills, hooks and `.mcp.json`; that MCP config launches
a pinned Chrome DevTools package through `npx`. `.codex/config.toml` supplies
parallel-agent and role settings. Node 18+ is the package baseline; several hooks
also rely on Bash/Python or their runtime resolvers. These are configuration and
execution changes, not merely text availability.

[Continuous learning's capture hook](https://github.com/affaan-m/ECC/blob/4eb71d92a39cab44ad40ac9d8a6a5ccb4029d6c2/skills/continuous-learning-v2/hooks/observe.sh)
persists truncated tool inputs/outputs to project observations, rotates files and
scrubs common secret-shaped patterns. Regex scrubbing cannot establish that an
arbitrary output is safe to retain. `observer.enabled=false` in `config.json`
controls model observer startup; it does not by itself disable observation
capture when the standard/strict hooks are registered. Project-hashed instincts
and confidence scores do not authorize automatic promotion into personal policy.
The bounded-stdin hook runner is a useful implementation example: it explicitly
marks truncation and lets selected critical hooks fail closed.

**Application:** add precise support and evidence states to the personal harness,
select only relevant specialist content, and evaluate actual artifacts. Leave
memory authoring under user control; do not automatically record raw tool I/O or
silently install the full plugin. Preserve inherited model choice and existing
client settings. Maintain a separately reviewed record for each executable hook
or MCP integration.

## Caveman

**Decision: adapt evaluation controls and fidelity principles; reject default
terse persona and automatic compression of tool contracts; defer its runtime.**
Revision [`2e08b91`](https://github.com/JuliusBrussee/caveman/tree/2e08b9177c07bb7249a8a2d1a6758e5db281d002).

The [current voice skill](https://github.com/JuliusBrussee/caveman/blob/2e08b9177c07bb7249a8a2d1a6758e5db281d002/skills/caveman/SKILL.md)
contains sensible constraints: preserve negation, identifiers, numbers and the
user's language, and prefer clarity over fewer words. However, persistent terse
mode, twenty-word sentences and deleted hedging are not appropriate default
communication for research, translation, leadership and explanation. The compact
`investigate-first` and `lean-build` skills offer evidence-ranked hypotheses and
complete narrow outcomes; their unknown `Native Core` dependency and categorical
no-edit-before-cause directive should not be copied as independent authority.

The [evaluation design](https://github.com/JuliusBrussee/caveman/blob/2e08b9177c07bb7249a8a2d1a6758e5db281d002/evals/README.md)
is more valuable than the persona: compare the skill against a plain concision
instruction as well as an unconstrained baseline, isolate installed host settings,
retain actual provider usage, reject incomplete snapshot matrices and separately
check factual fidelity. The `llm_run.py` command construction corroborates its host
isolation and usage collection. Old unisolated and current isolated runs are not
directly comparable. Regex fidelity cases still cannot establish general semantic
equivalence or successful engineering outcomes.

The package is now much broader than a skill. The Claude manifest wires session,
subagent, prompt and session-end hooks. Activation maintains per-session state,
can refresh owned statusline files, and applies environment-selected agent model
overrides. The inspected Codex hook resolves the configured default on startup or
resume and explicitly lacks the Claude session mode tracker; therefore persistent
off/on behavior is not equivalent between hosts.

The [MCP prose compressor](https://github.com/JuliusBrussee/caveman/blob/2e08b9177c07bb7249a8a2d1a6758e5db281d002/src/mcp-servers/caveman-shrink/compress.js)
protects code-looking spans but removes words such as `might`, `maybe` and `it
appears`. Those words can convey material uncertainty in a tool description.
Protecting syntax cannot guarantee preserving a contract. The MCP wrapper also
documents that descendant processes launched by an upstream wrapper can survive
shutdown because it owns only the immediate child. File compression calls Claude,
rewrites the original and makes an out-of-tree backup; it is a separate paid,
mutating workflow. The gateway routes model traffic and changes base URLs; its
README correctly labels standalone savings as inferred.

`package.json` requires Node 18+ for its installer and pulls its CLI. The gateway
is Go-based. `LICENSING.md`, `LICENSE` and retained notices establish the current
Apache-2.0 distribution, with earlier MIT notices retained; do not reuse older
license assumptions.

**Application:** preserve compact plain language, exact technical payloads and
uncertainty. Evaluate proposed context savings against an equally concise
baseline, including input overhead and recovery cost. Do not rewrite MCP
descriptions, project instructions or stored memories solely to reduce tokens.

## Gstack

**Decision: adapt evidence identity, substantive review and test-value criteria;
decline the entire runtime/preamble and automatic publication or memory behavior.**
Revision [`20eb620`](https://github.com/garrytan/gstack/tree/20eb6202fa8ea83a882e7c0463b722cd8a31af1e).

The source is a large workflow platform rather than a small skill collection. The
root skill and individual skills carry extensive executable preambles and helper
protocols. `bin/gstack-skill-start` consolidates bootstrap work but still inspects
or writes session state and can run update checks. `gstack-skill-end` discovers
brain-sync, records local state and routes telemetry according to settings.
Defaulting remote telemetry off does not make every lifecycle action read-only.
The current package uses Bun 1.4.2+ and a pinned, patched Playwright dependency;
its multi-thousand-line setup script was inspected for scope, not run or fully
audited. Root code is MIT, with Apache-2.0 derived design material explicitly
mapped in `NOTICE.md`; that provenance mapping is worth emulating.

The [review procedure](https://github.com/garrytan/gstack/blob/20eb6202fa8ea83a882e7c0463b722cd8a31af1e/review/SKILL.md)
checks concrete defects, surrounding code and incomplete enum/dispatch changes,
and asks for evidence before assigning severity. This is useful senior-lead craft.
Its fix-first behavior changes the meaning of a read-only review unless the task
already authorizes repair. Numeric confidence is a prioritization heuristic, not
a calibrated probability. Fixed review ceremonies and repeated preambles would
increase cost for small changes.

The QA skill requires each proposed regression test to explain what it protects,
how it fails, why an existing check misses it and why its seam is realistic. It
distinguishes verified from best-effort or reverted changes. These are better
criteria than testing every changed line. Its automatic per-fix commits and
fixed iteration limits should not enter shared defaults. The exploratory QA
section's immutable observations and replayable reproduction artifacts are
valuable for genuine investigations, but its per-invocation evidence rules should
not force rerunning unchanged expensive journeys.

[Review evidence code](https://github.com/garrytan/gstack/blob/20eb6202fa8ea83a882e7c0463b722cd8a31af1e/lib/review-evidence.ts)
ties reusable findings to canonical paths, hashed file content and specific
evidence/branch records. This is substantially better than accepting a generic
"reviewed" marker. Engineering-plan preferences emphasize interface costs,
reversibility, operational ownership and avoiding decorative abstraction. Its
brain-context retrieval then treats absent cached material as a reason to ask;
the personal harness should first inspect the actual project and preserve
uncertainty rather than require a separate brain store.

**Application:** use concrete defect categories, actual consumers and test-value
criteria in technical leadership. Make evidence reuse depend on relevant bytes
and configuration. Keep one owning task record and one authoritative workflow;
do not layer gstack bootstrap, memory, deployment and communication conventions
over the existing host.

## Context Mode

**Decision: adapt bounded retrieval, source identity and stale-result checks; defer
the MCP runtime and hook layer.** Revision
[`0dfbe8d`](https://github.com/mksglu/context-mode/tree/0dfbe8de71abcb637a07dd6444bee5823c3186fc).

The [skill](https://github.com/mksglu/context-mode/blob/0dfbe8de71abcb637a07dd6444bee5823c3186fc/skills/context-mode/SKILL.md)
keeps large files and command output outside the active window, indexes them, and
returns relevant query slices. It encourages batched questions, source scoping and
avoiding reindexing material already in context. `src/fetch-cache.ts` combines
source label and URL to avoid same-label collisions; the stale-detection tests
cover refreshed file content. These are useful retrieval invariants, independent
of choosing its server.

The inspected [executor](https://github.com/mksglu/context-mode/blob/0dfbe8de71abcb637a07dd6444bee5823c3186fc/src/executor.ts)
spawns local processes in a temporary working area, applies output limits and
filters some dangerous environment variables. It forwards other environment
values and retains access to the host. This is an output/context execution
boundary, not evidence of OS-level filesystem or network isolation. An omitted
timeout does not create a timer in the inspected path. Path/policy checks in
`security.ts` cannot make arbitrary child code an isolated workload.

The skill claims independent agent-browser processes are safe to parallelize;
the current agent-browser instead shares a default daemon unless unique sessions
are selected. Routing all reading and tests through this wrapper can also conflict
with purpose-built native tools. Its command whitelist is guidance, not a grant
to install packages or mutate a project.

The current package requires Node 22.5+, an MCP SDK, SQLite support and other
libraries. `scripts/postinstall.mjs` includes native-binding and plugin/settings
repair logic, so installation has more effects than placing an executable.
Codex hooks cover pre/post-tool, session, compaction and stop events; their actual
availability needs version-specific host testing. The license is Elastic License
2.0, not MIT or Apache; preserve its restrictions and notices rather than
relicensing copied implementation.

**Application:** improve existing context-management guidance with source/query
identity, completeness, raw recovery and freshness. Measure retrieval omissions as
well as token savings. Prefer available native tools and `rg` until representative
long-output tasks justify a separately authorized server. Keep process sandboxing
and tool authorization as separate, explicitly verified concerns.

## Agent Browser

**Decision: adapt session isolation, lazy capability discovery and browser evidence;
defer this additional browser runtime where the host already provides one.**
Revision [`44af398`](https://github.com/vercel-labs/agent-browser/tree/44af39842650f0bb9c1afb7354df9a82921d4f09),
Apache-2.0, package version 0.39.0.

The current [core guide](https://github.com/vercel-labs/agent-browser/blob/44af39842650f0bb9c1afb7354df9a82921d4f09/skill-data/core/SKILL.md)
describes a Rust/CDP implementation, not the historical Playwright implementation.
Its tiny installed skill delegates to version-matched CLI documentation. MCP core
tools remain small and extra network/state/debug capabilities are discovered when
needed. WebMCP summaries expose metadata first, and full schemas are loaded for
the selected action; origin metadata remains untrusted. This is a useful way to
bound tool schema context without guessing tool arguments.

Browser sessions persist. Parallel workers must use unique task/worktree sessions,
not the default daemon. The dogfood guide's domain-derived session slug is weaker
than the core guide's collision-resistant task guidance. Refresh references after
navigation and use an actual snapshot, console/network evidence and reproducible
steps. A screenshot establishes rendered state, not successful business behavior.
The dogfood skill's screenshots/video/repro report are useful, but its default
write/delete exploration must remain inside the user's authorized test scope.
An exported auth-state file is sensitive and should not be mixed into deliverables.

Security controls are mostly opt-in; the
[security guide](https://github.com/vercel-labs/agent-browser/blob/44af39842650f0bb9c1afb7354df9a82921d4f09/docs/content/docs/security.mdx)
explains domain-filter limitations and unsupported combinations. Browser plugins
are executable code, not sandboxed by the browser policy. A consequential source
finding is in
[`ActionPolicy::load_if_exists`](https://github.com/vercel-labs/agent-browser/blob/44af39842650f0bb9c1afb7354df9a82921d4f09/cli/src/native/policy.rs):
loading errors become `None`. `actions.rs` initializes state with that result and
enforces the policy only when it is present. Thus a malformed configured policy
can reach the inspected no-policy path. This was established by source tracing,
not a runtime exploit test; do not represent the optional policy as verified
fail-closed enforcement.

The npm wrapper specifies Node 24+; its postinstall downloads a platform binary
and can alter global shims. Browser acquisition is an additional runtime step.
The skill's preference for itself over native browser tools does not supersede
the user's host/tool choice. No binary or browser was installed in this review.

**Application:** preserve native browser control by default. If this runtime is
chosen for a project, pin binary/browser versions, use unique sessions, keep auth
state out of artifacts, verify malformed-policy handling, and exercise actual
target navigation. Treat tool discovery, invocation and user-flow acceptance as
separate evidence.

## Vercel Agent Skills

**Decision: adapt only stack-relevant rules and measured priorities; reject blanket
dependency, animation and deployment prescriptions.** Revision
[`063bee9`](https://github.com/vercel-labs/agent-skills/tree/063bee94c3f4df8453406c830b0a7df0f2860278).

[React best practices](https://github.com/vercel-labs/agent-skills/blob/063bee94c3f4df8453406c830b0a7df0f2860278/skills/react-best-practices/SKILL.md)
prioritizes asynchronous waterfalls and bundle costs before micro-optimizations,
with targeted rule files. `async-dependencies` shows early starts and dependency
coordination, including a native Promise alternative to an added library. Treat
its numerical gains as examples until measured on the actual path. Composition
patterns separate ownership and compound components from proliferating boolean
modes, with version-specific React guidance kept conditional.

The shared-state and cache rules should be read together. Request state must not
live in shared module variables; an intentional cache needs correct keys and
invalidation. The LRU example keys by user ID without spelling out tenant,
authorization or lifecycle scope. Copying that example cannot establish a safe
cache for another application. The deployed execution model, not a blanket
serverless assertion, determines reuse and isolation.

React Native guidance is useful for list identity, native rendering, animation
and package/autolink boundaries, but it is not universal mobile engineering. It
assumes particular libraries. The custom gesture example renders an animated
View without an equivalent accessible button contract; a faster gesture path
must preserve semantics, focus and assistive behavior. Verify package/runtime
versions and physical-device behavior before adopting performance advice.

The view-transition skill asks for broad application-wide coverage and includes
canary installation and framework version assumptions. Animation scope and
dependency changes belong to the task and installed stack. Its reference does
cover meaningful state, navigation and reduced-motion concerns. The web-design
skill fetches a live `main` reference, so byte identity and instruction trust are
not implied by the local skill's pin; code inspection is also not visual
acceptance.

The [deployment skill](https://github.com/vercel-labs/agent-skills/blob/063bee94c3f4df8453406c830b0a7df0f2860278/skills/deploy-to-vercel/SKILL.md)
prefers previews but has a linked-repository path using broad staging, commit and
push. Pushing a production branch can contradict the preview intent; finding a
latest deployment can also select a concurrent deployment unless tied to the
requested revision. Do not use this as a generic deployment authority.

Inspected skill frontmatter declares MIT where present; the fetched tree has no
root `LICENSE`. Resolve licensing at the exact copied skill/rule scope and retain
attribution, rather than asserting every unmarked wrapper has identical terms.
Prompt guidance itself needs no runtime install; each suggested dependency or
provider CLI is a separate decision.

**Application:** keep focused React/React Native references behind detected stack
and affected behavior. Preserve native/mobile accessibility, exact deployed
version and real measurements. Deployment adapters must bind the result to the
authorized environment and source revision, without automatic commit/push.

## Addy Osmani's agent skills

**Decision: adapt source/version grounding and layered evaluations; decline a
duplicate lifecycle router and the current fetch-cache implementation.** Revision
[`1be8e34`](https://github.com/addyosmani/agent-skills/tree/1be8e34187e34647bb83adc3a1323b26ae6f6abe),
MIT, version 0.6.12.

`context-engineering` preserves restartable state: objective, decisions, actual
file/Git state, verification and unresolved authority. `source-driven-development`
checks the installed dependency and official source before relying on recalled
APIs, and treats retrieved instructions as untrusted. A manifest range is not an
installed version; inspect the resolved version where needed. Latest docs do not
automatically override a project's supported release. Mandatory questioning or a
citation for every minor framework decision would be disproportionate.

The lifecycle router overlaps the personal harness's own selection mechanism.
Significantly, [the optional SessionStart hook](https://github.com/addyosmani/agent-skills/blob/1be8e34187e34647bb83adc3a1323b26ae6f6abe/hooks/session-start.sh)
is deliberately not wired into the inspected native plugin manifests, avoiding
duplicate router injection. Preserve that restraint. The doubt-driven review
pattern of giving a reviewer the artifact and contract without the builder's
conclusion can reduce anchoring. Its repeated cross-model suggestions and
per-invocation confirmations should not replace the user's persistent
authorization or actual host capability.

The browser-testing skill encourages DOM, console, network and performance
evidence in an isolated profile. Its `npx ...@latest` setup is an unpinned
dependency action; do not execute it just because the skill was selected.
Accessibility-tree inspection is not a screen-reader test. Navigation and
JavaScript evaluation should be governed by the authorized QA task, not an
universal confirmation loop.

The [evaluation guide](https://github.com/addyosmani/agent-skills/blob/1be8e34187e34647bb83adc3a1323b26ae6f6abe/evals/README.md)
distinguishes structural checks, lexical retrieval/routing proxies, real host
trigger tests, and behavior or artifact grading. It includes negative routing,
model/CLI pins, repeated trials, and execute-versus-dialogue cases. A selected skill
is only one part of success. The rejected-impact ledger makes failed optimization
attempts inspectable. Adopt the method, not its upstream score as evidence here.

The optional
[fetch-cache post hook](https://github.com/addyosmani/agent-skills/blob/1be8e34187e34647bb83adc3a1323b26ae6f6abe/hooks/sdd-cache-post.sh)
caches the shaped fetch result under a URL hash, then obtains a validator with a
separate HEAD request. The pre-hook reuses that result after a conditional HEAD
returns 304. The original extraction prompt is not part of the cache key, so a
different question about the same page can receive the wrong cached summary.
The hit message exposes the earlier prompt for the agent to assess, but still
blocks the requested fetch; this leaves suitability to a later judgment.
The separate result/validator requests can also bind old content to a newer
validator. A URL plus a successful 304 is not sufficient evidence for reusing a
query-shaped answer. The scripts need Bash, jq, curl and a SHA-256 utility; this
cache was not installed or executed.

**Application:** extend evaluations with real outcomes, negative routes and pinned
host configuration. Keep one router and useful progressive references. If caching
is ever justified, store raw content identity and the validator from the same
response, include extraction query/version and relevant request variation, and
distinguish freshness of the source from suitability of a derived answer.

## Integration priorities and acceptance probes

1. **Routing:** a substantive mobile synchronization request selects mobile
   engineering and its lifecycle reference; a one-line wording correction does
   not. Cross-role tasks have one outcome owner and bounded specialists.
2. **Task evidence:** a passing build cannot satisfy a visual, financial or
   business-flow acceptance criterion. State the remaining evidence and continue
   authorized work; do not represent skill registration as task execution.
3. **Reuse:** a reviewed file with unchanged relevant inputs can reuse evidence;
   changing a called interface, auth scope, dependency or environment invalidates
   the affected proof even if the top-level file is unchanged.
4. **Context:** an abbreviated log marks truncation and retains exact retrieval
   information. A missing graph edge or search result is not proof of absence.
5. **Runtime:** source-reviewed prompt availability never silently enables capture
   hooks, model proxies, downloads, browser state export, remote telemetry or
   autonomous memory promotion.
6. **Evaluation:** compare a new workflow with the current harness on representative
   successful and failing tasks. Record correctness and human rework alongside
   context, calls, latency and cost; do not optimize the token count alone.

These are implementation and evaluation targets, not claims that a fresh hosted
agent run or live client activation has already passed. The source checkout and
current host still need their own scoped verification after integration.
